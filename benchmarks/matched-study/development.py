#!/usr/bin/env python3
"""Registered new-skill/control comparison; frozen product and old harnesses stay unchanged."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
PRIVATE=ROOT/'benchmarks/private-evidence/development-study-20261124'
def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
L=load('dev_luna',HERE/'luna.py');B=L.configure('high');C=load('dev_corpus',HERE/'development_corpus.py')
MODEL=L.MODEL;SKILL='.agents/skills/lobo-development/SKILL.md'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def hashed(files):return {k:hashlib.sha256(v.encode()).hexdigest() for k,v in files.items()}
def save_sources(folder,files):
    folder.mkdir(parents=True,exist_ok=True)
    for name,text in files.items():
        path=folder/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(text)
B.save_sources=save_sources

def materials():
    paths=set()
    for folder in ('spec','libraries','.agents/skills/lobo-programming/references'):
        paths.update(p for p in (ROOT/folder).rglob('*') if p.is_file() and p.suffix in ('.md','.lobo','.json'))
    paths.update(ROOT/'docs/agent-reference/compact-pilot-2026-11-24'/name for name in ('ranges.txt','nominal.txt','propagation.txt','contracts.txt'))
    paths.add(ROOT/'research/compact-pilot-2026-11-24.md')
    return {str(p.relative_to(ROOT)):p.read_text() for p in sorted(paths)}

def fixtures(row):
    files=C.files(row['family'])|materials()
    files['VISIBLE.json']=json.dumps([dict(input=a,expected=C.oracle(row['family'],a)) for a in C.cases(row['family'])],indent=2)+'\n'
    if row['arm']=='skill':files[SKILL]=(ROOT/SKILL).read_text()
    elif row['arm']!='control':raise ValueError(row['arm'])
    return files

def guide(_):
    text='''Development task: edit logic.lobo and logic.lobo.ids.json; optional scratch tests/probes belong under scratch/. Other supplied files are read-only. Do not inspect other trials, reference implementations, the compiler binary/its parent directory, the real repository or external skill homes. No network, dependencies or IO/native calls in application functions. The compiler may be executed for authoring/checking, but not inspected. Use python3 check.py for visible native behavior checks, at most6 submissions. All work counts in the120-second/16-command budget. Read typed results and diagnostics; no self-reported success substitutes for checks. Model/util and test client are trusted fixed fixtures. Existing declarations/IDs must be preserved. Adding helpers is allowed.\n'''
    body=B.LOBO_GUIDE[B.LOBO_GUIDE.index('Each block returns'):B.LOBO_GUIDE.index('New functions/records/variants')]
    return text+body+'\nCommon reference files: spec/, libraries/, .agents/skills/lobo-programming/references/ and docs/agent-reference/compact-pilot-2026-11-24/. Read relevant files as needed; no older SKILL.md is installed.\nAfter adding declarations, sync identities with:\n'+str(B.COMPILER)+' identities sync logic.lobo --source model.lobo --source util.lobo\n'

def task_text(row):
    return C.REQUIREMENTS[row['family']]+'\n\nEditable algorithm files: logic.lobo and logic.lobo.ids.json. Optional scratch/ files do not alter acceptance. Preserve model/util and existing declarations/IDs. VISIBLE.json lists public checks. Hidden inputs use the same stated bounds. Run python3 check.py for native checks.\n'

def schedule():
    rows=[]
    for repeat in range(2):
        for i,task in enumerate(C.TASKS):
            arms=('control','skill') if (repeat+i)%2==0 else ('skill','control')
            for arm in arms:rows.append(dict(index=len(rows),repeat=repeat,family=task,arm=arm,language='lobo',reasoning='high',phase='change' if task=='dispatch' else 'construction'))
    return rows

def integrity(folder,fixed):
    issues=[]
    for name,text in fixed.items():
        p=folder/name
        if p.is_symlink():issues.append('symlink:'+name)
        elif not p.is_file():issues.append('missing:'+name)
        elif name not in ('logic.lobo','logic.lobo.ids.json') and p.read_text()!=text:issues.append('modified:'+name)
        elif name in ('logic.lobo','logic.lobo.ids.json') and p.stat().st_size>65536:issues.append('oversize:'+name)
    try:
        before=json.loads(fixed['logic.lobo.ids.json'])['identities'];after=json.loads((folder/'logic.lobo.ids.json').read_text())['identities']
        issues.extend('changed_identity:'+k for k,v in before.items() if after.get(k)!=v)
    except (OSError,ValueError,KeyError,TypeError):issues.append('invalid_identity_file')
    for p in folder.rglob('*'):
        if p.is_symlink():issues.append('symlink:'+str(p.relative_to(folder)))
        elif p.is_file():
            name=str(p.relative_to(folder))
            if name not in fixed and not name.startswith(('.check-io/','scratch/')):issues.append('unexpected:'+name)
    return sorted(set(issues))

def normalize(v):
    if v['type']=='Variant':return dict(case=v['value']['case'],payload=normalize(v['value']['payload']))
    if v['type']=='Record':return {k:normalize(x) for k,x in v['value']['fields'].items()}
    if v['type']=='List':return [normalize(x) for x in v['value']['elements']]
    if v['type']=='I64':return int(v['value'])
    return v['value']

def assess(folder,fixed,row,out,hidden=False):
    out.mkdir(parents=True,exist_ok=False);issues=integrity(folder,fixed);files=C.files(row['family'])
    for n in ('logic.lobo','logic.lobo.ids.json'):
        p=folder/n
        if p.is_file() and not p.is_symlink() and p.stat().st_size<=65536:files[n]=p.read_text()
    B.save_sources(out/'source',files);result=dict(issues=issues,build=None,cases=[],accepted=False)
    if not issues:
        command=[str(B.COMPILER),'compile',str(out/'source/logic.lobo'),'--source',str(out/'source/model.lobo'),'--source',str(out/'source/util.lobo'),'-o',str(out/'program')]
        try:
            p=subprocess.run(command,capture_output=True,text=True,timeout=35);result['build']=dict(ok=p.returncode==0,command=command,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr)
            if p.returncode==0:
                for a in C.cases(row['family'],hidden):
                    cmd=[str(out/'program'),'--entry','skilldev.'+row['family']+'.run','--fuel','10000000','--heap-cells','1000000']
                    for arg in C.arguments(row['family'],a):cmd+=['--arg',arg]
                    try:
                        q=subprocess.run(cmd,capture_output=True,text=True,timeout=5);value=json.loads(q.stdout);actual=normalize(value['value']) if q.returncode==0 and value.get('kind')=='value' else None;error=None if actual is not None else value
                    except Exception as e:actual=None;error=repr(e)
                    expected=C.oracle(row['family'],a);result['cases'].append(dict(input=a,expected=expected,actual=actual,error=error,ok=B.equal(expected,actual)))
                result['accepted']=all(c['ok'] for c in result['cases'])
        except Exception as e:result['infrastructure_error']=repr(e)
    result.update(passed=sum(c['ok'] for c in result['cases']),total=len(C.cases(row['family'],hidden)));B.write_json(out/'result.json',result);return result

def validate(out):
    out.mkdir(parents=True,exist_ok=False);results=[]
    for task in C.TASKS:
        for kind in ('starter','reference','mutant'):
            folder=out/(task+'-'+kind);files=C.files(task,kind);B.save_sources(folder,files)
            result=assess(folder,files,dict(family=task),out/(task+'-'+kind+'-check'),True)
            record=dict(task=task,kind=kind,result=result);results.append(record);B.write_json(out/'results.json',results)
            assert result['build'] and result['build']['ok'],record
            assert result['accepted']==(kind=='reference'),record
    return results

def isolated_home(parent):
    home=parent/'agent-home';home.mkdir(mode=0o700);(home/'skills').mkdir()
    for name in ('auth.json','models_cache.json'):
        source=Path.home()/'.codex'/name
        if source.is_file():shutil.copyfile(source,home/name);(home/name).chmod(0o600)
    return home

_ORIGINAL_COMMAND=B.command
_ORIGINAL_TRIAL=B.run_trial
_PROMPT_COMMON='Complete TASK.txt following GUIDE.txt. The limits are 120 seconds, 16 command-execution tool items and 6 visible check submissions. All build/check work counts in wall time. Do not delegate.\n'
def command(folder,row,last):return _ORIGINAL_COMMAND(folder,row,last)
def configure():
    B.fixtures=fixtures;B.guide=guide;B.task_text=task_text;B.assess=assess;B.command=command
    # Reuse the frozen process, timeout, mailbox, snapshot and terminal-usage implementation unchanged.


def register(out,validation):
    results=json.loads(validation.read_text());assert len(results)==9
    assert all(x['result']['build']['ok'] and x['result']['accepted']==(x['kind']=='reference') for x in results)
    old=json.loads((ROOT/'research/snapshots/2026-11-24-compact-pilot/registration/manifest.json').read_text())
    assert all(sha(ROOT/p)==h for p,h in old['code'].items()) and sha(B.COMPILER)==old['compiler_sha256']
    out.mkdir(parents=True,exist_ok=False);inputs={}
    for row in schedule():
        key=row['family']+'/'+row['arm'];files=fixtures(row)|dict(**{'TASK.txt':task_text(row),'GUIDE.txt':guide('lobo'),'check.py':B.CHECK_CLIENT,'AGENTS.md':'This is an isolated matched experiment. Follow TASK.txt and GUIDE.txt. Only the declared algorithm files are editable. Do not inspect other directories.\n'})
        text=_PROMPT_COMMON+('Use $lobo-development at .agents/skills/lobo-development/SKILL.md.\n' if row['arm']=='skill' else '')
        B.save_sources(out/'inputs'/row['family']/row['arm'],files);(out/('prompt-%02d.txt'%row['index'])).write_text(text);inputs[key]=dict(files=hashed(files),prompt_sha256=hashlib.sha256(text.encode()).hexdigest())
    paths=[HERE/n for n in ('development.py','development_corpus.py','test_development.py','development_report.py','test_development_report.py','study.py','luna.py','context_report.py')]+[ROOT/SKILL,ROOT/'.agents/skills/lobo-development/agents/openai.yaml']
    manifest=dict(format='lobo-development-skill-study/1',date='2026-11-24',model=MODEL,reasoning='high',scheduled=12,schedule=schedule(),inputs=inputs,code={str(p.relative_to(ROOT)):sha(p) for p in paths},material_hashes=hashed(materials()),compiler_sha256=sha(B.COMPILER),validation_sha256=sha(validation),environment=L.environment(),limits=dict(wall_s=120,command_tool_items=16,visible_checks=6,hard_token_cap=None),primary='Final complete hidden native behavior acceptance AND frozen-fixture/identity/budget integrity; report all failures and unattempted assignments.',secondary=['Held-out cases passed','Visible compiler/behavior failure counts and first edited accepted check','Command and visible-check use','Matched reported tokens/cache and wall ratios including failures','Skill exposure/read traces and control contamination'],failure_policy='One excluded source-free availability check, no retries/replacements/fallback. Stop after3 consecutive infrastructure failures and record unattempted rows.',treatment='Same references, task, scaffolds and budgets. Skill arm adds only frozen SKILL.md and explicit invocation. Global skill homes are isolated. Measures availability/invocation/workflow/salience together, not pure packaging.',compiler_regression='Unchanged SHA and prior312-pass receipt reused; new harness/calibration/TDD/research checks run. Full compiler/editor suite not rerun.',limitations=['Exploratory3 task instances x2 repeats: six matched pairs, not powered or independently blinded','New requirements were authored after reviewing prior errors; different tasks from older studies, not comparable absolute scores','Explicit skill activation and selective retrieval together; no equal-text placebo or isolated workflow/fact effect','Global skill files excluded using temporary CODEX_HOME; host/provider instructions and immutable weights remain unknown','Provider caching, session overhead and host load confound latency; reported tokens are not billing or parent development costs','Direct compiler/probe commands share16 command budget; visible mailbox checks capped6; no hard token cap','Common references are a selected normative/spec/library snapshot, not the full repository'])
    B.write_json(out/'manifest.json',manifest);return manifest

def run(out,registration):
    reg=json.loads(registration.read_text());assert all(sha(ROOT/p)==h for p,h in reg['code'].items());assert hashed(materials())==reg['material_hashes'];assert sha(B.COMPILER)==reg['compiler_sha256']
    out.mkdir(parents=True,exist_ok=False);B.write_json(out/'environment-before.json',L.environment());configure()
    # The old runner uses its fixed common prompt. Deliver only the registered invocation delta via TASK.txt
    # is not equivalent; instead replace its private prompt constant through a temporary function clone.
    import types
    code=B.run_trial.__code__
    # Create a separate code object per arm, replacing only its exact prompt string constant.
    original_home=os.environ.get('CODEX_HOME');rows=[]
    with tempfile.TemporaryDirectory(prefix='lobo-dev-private-home-') as parent:
        home=isolated_home(Path(parent));os.environ['CODEX_HOME']=str(home)
        try:
            canary=L.canary(B,'high',out/'availability');print('CANARY',canary['available'],flush=True)
            plan=reg['schedule'] if canary['available'] else [];streak=0
            for row in plan:
                text=_PROMPT_COMMON+('Use $lobo-development at .agents/skills/lobo-development/SKILL.md.\n' if row['arm']=='skill' else '')
                assert code.co_consts.count(_PROMPT_COMMON)==1
                cloned=types.FunctionType(code.replace(co_consts=tuple(text if x==_PROMPT_COMMON else x for x in code.co_consts)),B.__dict__)
                print('START',row,flush=True);result=cloned(row,out);rows.append(result);B.write_json(out/'results.json',rows);print('END',row['index'],result['status'],result['accepted'],result['grading']['passed'],result['usage'],flush=True)
                streak=streak+1 if L.infrastructure_failure(result) else 0
                if streak>=3:break
            attempted={r['assignment']['index'] for r in rows};B.write_json(out/'unattempted.json',[r for r in reg['schedule'] if r['index'] not in attempted]);B.write_json(out/'results.json',rows)
        finally:
            if original_home is None:os.environ.pop('CODEX_HOME',None)
            else:os.environ['CODEX_HOME']=original_home
    B.write_json(out/'environment-after.json',L.environment());print('FINISHED',len(rows),flush=True)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('operation',choices=['validate','register','run']);p.add_argument('--out',type=Path,required=True);p.add_argument('--validation',type=Path,default=PRIVATE/'validation/results.json');p.add_argument('--registration',type=Path,default=PRIVATE/'registration/manifest.json');a=p.parse_args()
    if a.operation=='validate':validate(a.out.resolve())
    elif a.operation=='register':register(a.out.resolve(),a.validation.resolve())
    else:run(a.out.resolve(),a.registration.resolve())
