#!/usr/bin/env python3
"""Ordered high/low reasoning extension of the frozen matched agent pilot."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time

ROOT=Path(__file__).resolve().parents[2]
PRIVATE=ROOT/'benchmarks/private-evidence/luna-study-20261123'
BASE=Path(__file__).with_name('study.py')
BASE_REGISTRATION=ROOT/'benchmarks/results/2026-11-23-matched-study-registration-amendment-1.json'
MODEL='gpt-5.6-luna'
EFFORTS=('high','low')

def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def baseline_registration():return json.loads(BASE_REGISTRATION.read_text())
def load_base():
    registration=baseline_registration()
    if sha(BASE)!=registration['harness_sha256']:raise ValueError('frozen base harness changed')
    spec=importlib.util.spec_from_file_location('frozen_matched',BASE)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def configure(effort):
    if effort not in EFFORTS:raise ValueError('unregistered reasoning level')
    module=load_base();original=module.command
    def command(folder,row,last):
        args=original(folder,row,last)
        args[args.index('-m')+1]=MODEL
        args=[('model_reasoning_effort="'+effort+'"') if x=='model_reasoning_effort="high"' else x for x in args]
        return args
    module.MODEL=MODEL;module.command=command
    return module

def fixture_hashes(module):
    files={}
    for row in module.schedule():
        values=module.fixtures(row)|{'TASK.txt':module.task_text(row),'GUIDE.txt':module.guide(row['language'])}
        for name,text in values.items():files['%02d/'%row['index']+name]={'sha256':hashlib.sha256(text.encode()).hexdigest(),'bytes':len(text.encode())}
    return files

def infrastructure_failure(result):
    return bool(result['status'] in {'infrastructure_error','agent_error'} or result['grading'].get('infrastructure_error'))

def environment():
    versions=[]
    for command in [['codex','--version'],['rustc','--version'],['clang','--version'],['python3','--version']]:
        process=subprocess.run(command,capture_output=True,text=True,check=True);path=Path(shutil.which(command[0])).resolve()
        versions.append({'command':command,'stdout':process.stdout,'stderr':process.stderr,'executable':str(path),'sha256':sha(path)})
    return {'versions':versions,'logical_cpus':os.cpu_count(),'load':os.getloadavg(),'recorded_unix':time.time()}

def register(out):
    module=load_base();prior=baseline_registration();files=fixture_hashes(module)
    if files!=prior['files']:raise ValueError('task/source/guide content drift')
    freeze=json.loads((module.PRIVATE/'freeze/manifest.json').read_text())
    if any(sha(module.PRIVATE/'freeze'/name)!=value['sha256'] for name,value in freeze['files'].items()):raise ValueError('compiler/source freeze changed')
    catalog_path=Path.home()/'.codex/models_cache.json';catalog=json.loads(catalog_path.read_text());model=next(m for m in catalog['models'] if m['slug']==MODEL)
    if not set(EFFORTS)<={r['effort'] for r in model['supported_reasoning_levels']}:raise ValueError('model does not advertise both reasoning levels')
    manifest={'date':'2026-11-23','format':'lobo-luna-reasoning-study/1','model':MODEL,'ordered_cohorts':list(EFFORTS),'scheduled_per_cohort':54,'scheduled_total':108,'source_commit':prior['source_commit'],'frozen_version':'1.11.2','baseline_registration_sha256':sha(BASE_REGISTRATION),'base_harness_sha256':sha(BASE),'wrapper_sha256':sha(__file__),'input_files':files,'schedule':module.schedule(),'guides':prior['guides'],'limits':{k:prior[k] for k in ['deadline_s','command_tool_items','visible_checks','hard_token_cap']},'environment':environment(),'catalog':{'fetched_at':catalog.get('fetched_at'),'client_version':catalog.get('client_version'),'model':{k:model.get(k) for k in ['slug','description','supported_reasoning_levels','supported_in_api']}},'failure_policy':'Retain every attempt and unknown usage. No retries or replacements. Three consecutive infrastructure failures stop that cohort; proceed to availability check for the next effort. Candidate compile/test/limit failures are ordinary outcomes.','availability_canaries':'One source-free, shell-disabled CLI invocation immediately before each cohort; exclude from scored trials, retain usage. No fallback model.','scope':'Participant CLI invocations only; parent/harness-development usage and billing unknown; no hard token cap','limitations':['Model alias is not immutable weights; older description is not a measured capability ranking','High-before-low order was requested; time/cache/provider drift confounds a causal reasoning comparison','Baseline was measured earlier; equal frozen task bytes and CLI flags do not guarantee identical host instructions or backend behavior','Same three small task families and cued comprehension items as baseline; no task tuning after outcomes']}
    out.mkdir(parents=True,exist_ok=False);module.write_json(out/'manifest.json',manifest)
    return manifest

def canary(module,effort,out):
    out.mkdir(parents=True,exist_ok=False);folder=Path(tempfile.mkdtemp(prefix='luna-availability-'));(folder/'AGENTS.md').write_text('This is a source-free model availability check. Follow the user prompt; do not inspect other directories.\n')
    row={'phase':'comprehension','language':'python'};command=module.command(folder,row,out/'last-message.txt');module.write_json(out/'command.json',command)
    prompt='Model availability check. Use no tools. Reply only READY.';started=time.monotonic();result={'model':MODEL,'reasoning':effort,'excluded_from_scored_trials':True,'command':command}
    env=dict(os.environ);env.pop('CODEX_THREAD_ID',None);env.pop('CODEX_INTERNAL_ORIGINATOR_OVERRIDE',None)
    process=subprocess.Popen(command,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=folder,env=env,text=True,start_new_session=True)
    timed_out=False
    try:stdout,stderr=process.communicate(prompt,timeout=90)
    except subprocess.TimeoutExpired:
        timed_out=True;os.killpg(process.pid,signal.SIGTERM)
        try:stdout,stderr=process.communicate(timeout=3)
        except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);stdout,stderr=process.communicate()
    (out/'events.jsonl').write_text(stdout);(out/'stderr.txt').write_text(stderr);events=[]
    for line in stdout.splitlines():
        try:events.append(json.loads(line))
        except ValueError:pass
    result.update(exit_code=process.returncode,usage=module.usage(events),available=not timed_out and process.returncode==0 and any(e.get('type')=='turn.completed' for e in events),last_message=(out/'last-message.txt').read_text() if (out/'last-message.txt').exists() else None)
    if timed_out:result['error']='availability timeout; retain reported usage or unknown'
    result['wall_s']=time.monotonic()-started;module.write_json(out/'result.json',result);return result

def run(out,registration_path):
    registration=json.loads(registration_path.read_text());module=load_base()
    if sha(__file__)!=registration['wrapper_sha256'] or fixture_hashes(module)!=registration['input_files']:raise ValueError('registered harness or inputs changed')
    out.mkdir(parents=True,exist_ok=False);overall=[];module.write_json(out/'environment-before.json',environment())
    for effort in registration['ordered_cohorts']:
        module=configure(effort);cohort=out/effort;cohort.mkdir();module.write_json(cohort/'assignment-plan.json',registration['schedule']);results=[]
        print('CANARY',MODEL,effort,flush=True);availability=canary(module,effort,cohort/'availability');module.write_json(cohort/'availability.json',availability)
        if not availability['available']:
            module.write_json(cohort/'results.json',results);module.write_json(cohort/'stopped.json',{'reason':'model/reasoning unavailable; no fallback','remaining':registration['schedule']});overall.append({'reasoning':effort,'attempted':0,'availability':False});module.write_json(out/'progress.json',overall);continue
        streak=0
        for row in registration['schedule']:
            print('START',effort,row,flush=True);result=module.run_trial(row,cohort)
            result['configuration']={'model':MODEL,'reasoning':effort,'base_harness_sha256':registration['base_harness_sha256'],'wrapper_sha256':registration['wrapper_sha256']};module.write_json(cohort/('trial-%02d'%row['index'])/'result.json',result);results.append(result);module.write_json(cohort/'results.json',results)
            print('END',effort,row['index'],result['status'],result['accepted'],result['usage'],flush=True)
            streak=streak+1 if infrastructure_failure(result) else 0
            if streak>=3:
                module.write_json(cohort/'stopped.json',{'reason':'three consecutive infrastructure failures','remaining':registration['schedule'][len(results):]});break
        overall.append({'reasoning':effort,'attempted':len(results),'accepted':sum(r['accepted'] for r in results),'availability':True});module.write_json(out/'progress.json',overall)
    module.write_json(out/'environment-after.json',environment());print('FINISHED',overall,flush=True)

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('operation',choices=['register','run']);parser.add_argument('--out',type=Path,required=True);parser.add_argument('--registration',type=Path,default=PRIVATE/'registration/manifest.json');args=parser.parse_args()
    if args.operation=='register':register(args.out.resolve())
    else:run(args.out.resolve(),args.registration.resolve())
