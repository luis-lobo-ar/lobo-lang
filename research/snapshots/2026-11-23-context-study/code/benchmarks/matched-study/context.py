#!/usr/bin/env python3
"""Versioned 2x2 source-only context experiment; never edits the frozen driver."""
import argparse
import copy
import hashlib
import importlib.util
import itertools
import json
import os
from pathlib import Path
import random
import re
import signal
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).parent
PRIVATE = ROOT/'benchmarks/private-evidence/context-study-20261123'
spec = importlib.util.spec_from_file_location('frozen_luna', HERE/'luna.py')
LUNA = importlib.util.module_from_spec(spec); spec.loader.exec_module(LUNA)
BASE = LUNA.load_base()
ARMS = ('original','dedup','schema','both')
RENAMES = {
    'ledger': {'valid_amount':'amount_allowed','step':'apply_event'},
    'scheduler': {'ready':'deps_complete','fits':'budget_allows','choose':'next_job'},
    'frames': {'read_header':'read_size','fits':'size_allowed','checksum':'payload_checksum'},
}
ISOLATION = 'This is an isolated matched experiment. Follow TASK.txt and GUIDE.txt. Only the declared algorithm files are editable. Do not inspect other directories.\n'


def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def hash_text(text): return hashlib.sha256(text.encode()).hexdigest()
def renamed(text, family):
    return re.sub(r'\b('+ '|'.join(RENAMES[family])+r')\b', lambda m: RENAMES[family][m[0]], text)


def logic(family, language, mutant=False):
    text = renamed(BASE.reference(family,language),family)
    if family == 'ledger':
        text = text.replace('<=1000000','<=97')
        if mutant:
            text = text.replace('value>=0','value>=1').replace('0<=value','1<=value')
    elif family == 'scheduler':
        text = text.replace('item.id<old.id','item.id>old.id').replace('item.id<best.id','item.id>best.id')
        if mutant: text = text.replace('cost<=remaining','cost+2<=remaining')
    else:
        text = text.replace('total:I64=0','total:I64=7').replace('fold(0u8,','fold(7u8,').replace('sum(data)%256','(7+sum(data))%256')
        if mutant:
            text = text.replace('length<=limit','length<=limit && length!=2' if language!='python' else 'length<=limit and length!=2')
    return text


def bundle(family,language):
    files = BASE.sources(family,language)
    suffix = {'lobo':'lobo','rust':'rs','python':'py'}[language]
    files['logic.'+suffix] = logic(family,language)
    if language == 'lobo':
        sidecar = json.loads(files['logic.lobo.ids.json'])
        sidecar['identities'] = {RENAMES[family].get(k,k):v for k,v in sidecar['identities'].items()}
        files['logic.lobo.ids.json'] = json.dumps(sidecar,indent=2)+'\n'
    files.update({'TASK.txt':task(family,language),'GUIDE.txt':BASE.guide(language),'check.py':BASE.CHECK_CLIENT,'AGENTS.md':ISOLATION})
    return files


def input_schema(family):
    common = ('INPUT DOCUMENTATION v1 (identical across languages). Inputs are JSON objects. Integers below are JSON integers, not strings, floats or booleans. Arrays preserve order. The harness converts this wire representation to the run signature; this is not language syntax.\n')
    schemas = {
      'ledger': 'Fields: initial: integer in [0,1000000]; events: array of at most32 pairs [kind,amount]. kind is integer0=Credit(amount),1=Debit(amount),2=Lock,3=Unlock. amount is an integer in [-1000000,1000001]; for Lock/Unlock it is ignored. The pair encoding maps to Event values before run; it is not a tuple parameter passed directly to run.',
      'scheduler': 'Fields: budget: integer in [0,512]; jobs: array of at most24 objects, each with exactly id,cost,deps. id: integer, all job IDs together are exactly0..n-1 in any array order. cost: integer in [0,64]. deps: array of at most24 valid job IDs; repeats, self-references and cycles are allowed. deps are IDs, not array positions.',
      'frames': 'Fields: limit: integer in [0,64]; data: even-length hexadecimal string encoding at most512 raw bytes, two hexadecimal digits per byte. Input hex accepts either letter case; output bytes use lowercase hex. Decode data before calling run: the string is not UTF-8 payload text, a decimal number, or one byte per digit. An empty string encodes zero bytes. All offsets and lengths in the program count decoded bytes.',
    }
    return common+schemas[family]+'\n'


def prompt(files,arm,family):
    if arm not in ARMS: raise ValueError(arm)
    # Canonical source order reproduces the old dict insertion order even for JSON snapshots.
    names = [n for stem in ('model','util','logic') for n in files if n.split('.')[0]==stem and '.ids.' not in n]
    names += ['TASK.txt','GUIDE.txt','check.py','AGENTS.md']
    if arm in ('dedup','both'): names = [n for n in names if n not in ('TASK.txt','GUIDE.txt')]
    text = 'Source-only comprehension. Do not use tools.\n'+files['TASK.txt']+'\n'+files['GUIDE.txt']+'\n'+'\n'.join('FILE '+n+'\n'+files[n] for n in names)
    if arm in ('schema','both'): text += '\n'+input_schema(family)
    return text


def inputs(family):
    return {
      'ledger': [
        {'initial':8,'events':[[0,4],[1,5],[2,0],[0,3],[1,0],[3,0],[1,2]]},
        {'initial':3,'events':[[0,97],[0,1],[1,98],[1,97]]},
        {'initial':12,'events':[[2,0],[2,9],[1,4],[0,2],[3,0],[1,14]]},
        {'initial':6,'events':[[0,-2],[1,8],[1,0],[0,0],[1,6]]},
        {'initial':0,'events':[[0,98],[0,96],[1,96],[1,1]]},
        {'initial':4,'events':[[3,7],[1,4],[0,97],[0,97],[1,97]]}],
      'scheduler': [
        {'jobs':[{'id':0,'cost':2,'deps':[]},{'id':2,'cost':1,'deps':[]},{'id':1,'cost':1,'deps':[]}],'budget':2},
        {'jobs':[{'id':2,'cost':0,'deps':[1]},{'id':1,'cost':2,'deps':[0,0]},{'id':0,'cost':1,'deps':[]}],'budget':3},
        {'jobs':[{'id':0,'cost':0,'deps':[0]},{'id':2,'cost':0,'deps':[]},{'id':1,'cost':0,'deps':[2]}],'budget':0},
        {'jobs':[{'id':1,'cost':5,'deps':[]},{'id':0,'cost':2,'deps':[]},{'id':2,'cost':6,'deps':[]}],'budget':4},
        {'jobs':[{'id':0,'cost':1,'deps':[1]},{'id':1,'cost':0,'deps':[0]},{'id':2,'cost':1,'deps':[]}],'budget':6},
        {'jobs':[{'id':2,'cost':2,'deps':[]},{'id':0,'cost':1,'deps':[2]},{'id':1,'cost':3,'deps':[]}],'budget':5}],
      'frames': [
        {'data':'0007','limit':1}, {'data':'02040510','limit':3},
        {'data':'0109100203040e','limit':3}, {'data':'0109100203040f','limit':3},
        {'data':'0401','limit':3}, {'data':'0109100203','limit':3}],
    }[family]


def oracle(family,args,mutant=False):
    if family=='ledger':
        balance=args['initial'];peak=balance;rejected=0;locked=False;audit=[]
        for kind,amount in args['events']:
            if kind==2: locked=True
            elif kind==3: locked=False
            elif not (int(mutant)<=amount<=97) or (kind==1 and (locked or amount>balance)): rejected+=1
            else: balance += amount if kind==0 else -amount
            peak=max(peak,balance);audit.append(balance)
        return dict(balance=balance,rejected=rejected,peak=peak,audit=audit)
    if family=='scheduler':
        done=[];remaining=args['budget']
        while True:
            eligible=[j for j in args['jobs'] if j['id'] not in done and all(d in done for d in j['deps']) and j['cost']+(2 if mutant else 0)<=remaining]
            if not eligible: break
            job=max(eligible,key=lambda j:j['id']);done.append(job['id']);remaining-=job['cost']
        return dict(order=done,spent=args['budget']-remaining,blocked=len(args['jobs'])-len(done))
    data=bytes.fromhex(args['data']);cursor=0;frames=[];error=0
    while cursor<len(data):
        n=data[cursor];end=cursor+n+2
        if n>args['limit'] or (mutant and n==2):error=3
        elif end>len(data):error=1
        elif (7+sum(data[cursor+1:end-1]))%256!=data[end-1]:error=2
        else:frames.append(data[cursor+1:end-1].hex());cursor=end;continue
        break
    return dict(frames=frames,consumed=cursor,error=error)


def target(family):
    return {'ledger':dict(balance=10,rejected=2,peak=13),'scheduler':dict(order=[2,0],spent=4,blocked=1),'frames':dict(frames=['0a','0b0c'],consumed=7,error=0)}[family]


def question(family):
    change={'ledger':'amount_allowed changes the inclusive lower amount bound from0 to1 (zero amounts become invalid)','scheduler':'budget_allows changes cost<=remaining to cost+2<=remaining (retain at least2 units)','frames':'size_allowed additionally rejects length==2, retaining the inclusive upper limit'}[family]
    return ('Analyze the supplied program without executing it. Return ONE JSON object with keys: predictions (the SIX full normalized Summary objects for the inputs below, in order); witness (any valid input yielding '+json.dumps(target(family))+'; unspecified Summary fields can have any value); direct_calls (names of application functions directly invoked by run, ignoring constructors/builtins/standard-library methods and util.reverse); impacted (names of functions in logic whose return values can change for identical direct arguments if '+change+'; include the edited predicate itself, ignore reachability alone); fault_location (the predicate function responsible for that policy); counterexample (any valid input distinguishing the original program from that mutant). Bytes in answers are lowercase hexadecimal strings. Use plain function names without module prefixes. No tools/target execution. Inputs:\n'+json.dumps(inputs(family)))


def task(family,language):
    requirement=BASE.BASE_REQUIREMENTS[family]
    if family=='ledger':requirement=requirement.replace('0<=amount<=1000000','0<=amount<=97')
    elif family=='scheduler':requirement=requirement.replace('LOWEST ID','HIGHEST ID')
    else:requirement=requirement.replace('sum(payload)%256','(7+sum(payload))%256').replace('needs checksum0','needs checksum7')
    return 'Task family: '+family+'. Phase: comprehension.\n'+requirement+'\n\nRequired signature: run('+BASE.signature(family,language)+').\n\n'+question(family)+'\nUse NO tools or execution; reason from the supplied source. End with ONLY the requested JSON object, no Markdown.\n'


def correct(family):
    witness={
      'ledger':{'initial':13,'events':[[1,3],[1,11],[0,98]]},
      'scheduler':{'jobs':[{'id':2,'cost':1,'deps':[]},{'id':0,'cost':3,'deps':[2]},{'id':1,'cost':8,'deps':[]}],'budget':4},
      'frames':{'data':'010a11020b0c1e','limit':3}}[family]
    counter={'ledger':{'initial':2,'events':[[0,0]]},'scheduler':{'jobs':[{'id':0,'cost':2,'deps':[]},{'id':1,'cost':2,'deps':[]}],'budget':4},'frames':{'data':'0201020a','limit':3}}[family]
    direct={'ledger':['apply_event'],'scheduler':['next_job'],'frames':['read_size','size_allowed','payload_checksum']}[family]
    impacted={'ledger':['amount_allowed','apply_event','run'],'scheduler':['budget_allows','next_job','run'],'frames':['size_allowed','run']}[family]
    return dict(predictions=[oracle(family,x) for x in inputs(family)],witness=witness,direct_calls=direct,impacted=impacted,fault_location=impacted[0],counterexample=counter)


def grade(family,answer):
    answer=answer if isinstance(answer,dict) else {};pred=answer.get('predictions');checks={}
    for i,x in enumerate(inputs(family)):
        checks['prediction_'+str(i)]=isinstance(pred,list) and len(pred)==6 and BASE.equal(pred[i],oracle(family,x))
    witness=answer.get('witness');checks['witness']=BASE.valid_input(family,witness) and all(BASE.equal(oracle(family,witness)[k],v) for k,v in target(family).items())
    wanted=correct(family)
    for key in ('direct_calls','impacted'):
        value=answer.get(key);checks[key]=isinstance(value,list) and all(isinstance(x,str) for x in value) and len(value)==len(wanted[key]) and set(value)==set(wanted[key])
    checks['fault_location']=answer.get('fault_location')==wanted['fault_location']
    counter=answer.get('counterexample');checks['counterexample']=BASE.valid_input(family,counter) and oracle(family,counter)!=oracle(family,counter,True)
    return dict(accepted=all(checks.values()),passed=sum(checks.values()),total=11,checks=checks)


def schedule():
    rows=[]
    for effort in ('high','low'):
        blocks=list(itertools.product(BASE.FAMILIES,BASE.LANGUAGES));random.Random(20261124).shuffle(blocks)
        for i,(family,language) in enumerate(blocks):
            # Four balanced orders: each arm appears once in each position per cycle.
            order=tuple(ARMS[j] for j in ((0,1,3,2),(1,2,0,3),(2,3,1,0),(3,0,2,1))[i%4])
            for arm in order:rows.append(dict(index=len(rows),reasoning=effort,family=family,language=language,arm=arm,phase='comprehension'))
    return rows


def command(folder,effort,last):
    return LUNA.configure(effort).command(folder,{'phase':'comprehension'},last)


def validation(out):
    out.mkdir(parents=True,exist_ok=False);records=[]
    for family,language,mutant in itertools.product(BASE.FAMILIES,BASE.LANGUAGES,(False,True)):
        folder=out/(family+'-'+language+('-mutant' if mutant else ''))
        files=bundle(family,language);suffix={'lobo':'lobo','rust':'rs','python':'py'}[language]
        files['logic.'+suffix]=logic(family,language,mutant)
        BASE.save_sources(folder,files);compiled=BASE.build(folder,language,folder/'program')
        record=dict(family=family,language=language,mutant=mutant,build=compiled,cases=[]);records.append(record);BASE.write_json(out/'results.json',records)
        if not compiled['ok']:raise RuntimeError(compiled)
        probes=inputs(family)+[correct(family)['witness'],correct(family)['counterexample']]+BASE.cases(family,hidden=True)
        # Original random frames seldom contain valid seeded frames; add deterministic valid ones.
        if family=='frames':
            for n in range(10):
                data=bytes(range(n));probes.append(dict(data=(bytes([n])+data+bytes([(7+sum(data))%256])).hex(),limit=n))
        for args in probes:
            actual=BASE.execute(folder,language,family,args,folder/'program');expected=oracle(family,args,mutant)
            record['cases'].append(dict(input=args,actual=actual,expected=expected,ok=BASE.equal(actual,expected)))
            if not BASE.equal(actual,expected):BASE.write_json(out/'results.json',records);raise ValueError(record['cases'][-1])
        record['mutant_killed_by_behavior']=None if not mutant else oracle(family,correct(family)['counterexample'])!=BASE.execute(folder,language,family,correct(family)['counterexample'],folder/'program')
        if mutant and not record['mutant_killed_by_behavior']:raise ValueError('surviving mutant')
        BASE.write_json(out/'results.json',records)
    return records


def register(out,validation_path):
    checks=json.loads(validation_path.read_text())
    if len(checks)!=18 or not all(r['build']['ok'] and all(x['ok'] for x in r['cases']) and (not r['mutant'] or r['mutant_killed_by_behavior']) for r in checks):raise ValueError('reference/mutation gate failed')
    freeze=json.loads((BASE.PRIVATE/'freeze/manifest.json').read_text())
    if any(sha(BASE.PRIVATE/'freeze'/name)!=value['sha256'] for name,value in freeze['files'].items()):raise ValueError('freeze drift')
    out.mkdir(parents=True,exist_ok=False)
    inputs_manifest={}
    for family,language in itertools.product(BASE.FAMILIES,BASE.LANGUAGES):
        files=bundle(family,language)
        for arm in ARMS:
            key='/'.join([family,language,arm]);text=prompt(files,arm,family)
            BASE.save_sources(out/'inputs'/family/language,files)
            path=out/'prompts'/(key.replace('/','-')+'.txt');path.parent.mkdir(exist_ok=True);path.write_text(text)
            inputs_manifest[key]=dict(prompt_sha256=hash_text(text),prompt_bytes=len(text.encode()),files={k:hash_text(v) for k,v in files.items()})
    record=dict(format='lobo-context-factorial/1',date='2026-11-23',model=LUNA.MODEL,ordered_cohorts=['high','low'],schedule=schedule(),scheduled_total=72,arms=list(ARMS),driver_sha256=sha(__file__),tests_sha256=sha(HERE/'test_context.py'),base_driver_sha256=sha(HERE/'study.py'),luna_driver_sha256=sha(HERE/'luna.py'),compiler_sha256=sha(BASE.COMPILER),freeze_version='1.11.2',inputs=inputs_manifest,validation_sha256=sha(validation_path),environment=LUNA.environment(),limits=dict(deadline_s=120,tools=0,hard_token_cap=None),scope='Source-only comprehension, fresh contexts and fresh program/question variants. No reference graph exposure, no product or frozen-fixture edits.',interventions={'dedup':'Remove only FILE TASK.txt and FILE GUIDE.txt duplicates, retain other noise.','schema':'Append one language-neutral input schema; identical bytes for all languages in each family.'},analysis='Report all attempts/items by arm, language and effort; matched within-question discordances, usage including cache, wall time and unknowns. Pilot is descriptive: one program form/family, no powered generalization or posthoc task tuning.',failure_policy='No retries/replacements. After three consecutive infrastructure failures stop that cohort, record remaining assignments. One excluded source-free availability canary per effort. Missing terminal usage is unknown.',limitations=['Four-arm matched questions repeat across stateless independent invocations; provider caching and time drift remain possible.','High then low order prevents a causal reasoning comparison.','Fresh programs change constants/helper names/order policy relative to old studies; do not interpret old-vs-new score differences as treatment effects.','Parent/harness-development tokens and actual billing unavailable.','Graph usefulness not measured in this experiment.'])
    BASE.write_json(out/'manifest.json',record);return record


def trial(row,out):
    out.mkdir(parents=True,exist_ok=False);files=bundle(row['family'],row['language']);BASE.save_sources(out/'initial',files)
    folder=Path(tempfile.mkdtemp(prefix='lobo-context-'));BASE.save_sources(folder,files)
    text=prompt(files,row['arm'],row['family']);(out/'prompt.txt').write_text(text)
    args=command(folder,row['reasoning'],out/'last-message.txt');BASE.write_json(out/'command.json',args);BASE.write_json(out/'assignment.json',row)
    env=dict(os.environ);env.pop('CODEX_THREAD_ID',None);env.pop('CODEX_INTERNAL_ORIGINATOR_OVERRIDE',None)
    started=time.monotonic();result=dict(assignment=row,command=args,started_unix=time.time(),load_before=os.getloadavg(),workspace=str(folder));stdout='';stderr=''
    try:
        p=subprocess.Popen(args,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=folder,env=env,text=True,start_new_session=True)
        try:stdout,stderr=p.communicate(text,timeout=120);status='completed' if p.returncode==0 else 'agent_error'
        except subprocess.TimeoutExpired:
            status='wall_limit';os.killpg(p.pid,signal.SIGTERM)
            try:stdout,stderr=p.communicate(timeout=3)
            except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);stdout,stderr=p.communicate()
        result.update(status=status,exit_code=p.returncode)
    except Exception as exc:result.update(status='infrastructure_error',error=repr(exc))
    result.update(agent_wall_s=time.monotonic()-started,load_after=os.getloadavg())
    (out/'events.jsonl').write_text(stdout);(out/'stderr.txt').write_text(stderr);events=[];items={}
    for line in stdout.splitlines():
        try:events.append(json.loads(line))
        except ValueError:pass
    for event in events:
        item=event.get('item',{})
        if item.get('id'):items[item['id']]=item
    last=(out/'last-message.txt').read_text() if (out/'last-message.txt').exists() else ''
    result.update(usage=BASE.usage(events),item_types=[i.get('type') for i in items.values()],grading=grade(row['family'],BASE.parse_answer(last)))
    result['accepted']=result['status']=='completed' and result['grading']['accepted'] and all(i.get('type') in {'agent_message','reasoning'} for i in items.values())
    BASE.write_json(out/'result.json',result);return result


def run(out,registration_path):
    registration=json.loads(registration_path.read_text())
    if sha(__file__)!=registration['driver_sha256'] or sha(HERE/'study.py')!=registration['base_driver_sha256'] or sha(HERE/'luna.py')!=registration['luna_driver_sha256'] or sha(BASE.COMPILER)!=registration['compiler_sha256']:raise ValueError('registered code changed')
    for family,language,arm in itertools.product(BASE.FAMILIES,BASE.LANGUAGES,ARMS):
        key='/'.join([family,language,arm])
        if hash_text(prompt(bundle(family,language),arm,family))!=registration['inputs'][key]['prompt_sha256']:raise ValueError('prompt drift')
    out.mkdir(parents=True,exist_ok=False);BASE.write_json(out/'environment-before.json',LUNA.environment());results=[]
    for effort in ('high','low'):
        module=LUNA.configure(effort);canary=LUNA.canary(module,effort,out/('availability-'+effort));print('CANARY',effort,canary['available'],flush=True)
        plan=[r for r in registration['schedule'] if r['reasoning']==effort];streak=0
        if not canary['available']:
            BASE.write_json(out/('unattempted-'+effort+'.json'),dict(reason='unavailable',assignments=plan));continue
        for i,row in enumerate(plan):
            print('START',row,flush=True);result=trial(row,out/('trial-%02d'%row['index']));results.append(result);BASE.write_json(out/'results.json',results)
            print('END',row['index'],result['status'],result['accepted'],result['grading']['passed'],result['usage'],flush=True)
            streak=streak+1 if result['status'] in ('agent_error','infrastructure_error') else 0
            if streak>=3:BASE.write_json(out/('unattempted-'+effort+'.json'),dict(reason='three consecutive infrastructure failures',assignments=plan[i+1:]));break
    BASE.write_json(out/'environment-after.json',LUNA.environment());print('FINISHED',len(results),flush=True)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('operation',choices=['validate','register','run']);p.add_argument('--out',type=Path,required=True);p.add_argument('--validation',type=Path,default=PRIVATE/'validation/results.json');p.add_argument('--registration',type=Path,default=PRIVATE/'registration/manifest.json');a=p.parse_args()
    if a.operation=='validate':validation(a.out.resolve())
    elif a.operation=='register':register(a.out.resolve(),a.validation.resolve())
    else:run(a.out.resolve(),a.registration.resolve())
