#!/usr/bin/env python3
"""Three-arm fresh-task comparison of a smaller, grammar-first Lobo development skill."""
import argparse,hashlib,importlib.util,json,os,re,subprocess,tempfile,types
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];PRIVATE=ROOT/'benchmarks/private-evidence/skill-revision-20261126'
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
D=load('isolated_original_development',HERE/'development.py');B=D.B;C=load('revision_corpus',HERE/'skill_revision_corpus.py');D.C=C
ARMS=('control','original','revised');SKILLS={'original':'.agents/skills/lobo-development/SKILL.md','revised':'.agents/skills/lobo-development-v2/SKILL.md'}
sha=D.sha;hashed=D.hashed

def normalize(v):
 if v is None:return None
 if v['type']=='Variant':return dict(case=v['value']['case'],payload=normalize(v['value'].get('payload')))
 if v['type']=='Record':return {k:normalize(x) for k,x in v['value']['fields'].items()}
 if v['type']=='List':return [normalize(x) for x in v['value']['elements']]
 if v['type']=='I64':return int(v['value'])
 return v['value']
D.normalize=normalize

def fixtures(row):
 task=row['family'];files=C.files(task)|D.materials()
 files['VISIBLE.json']=json.dumps([dict(input=a,expected=C.oracle(task,a)) for a in C.cases(task)],indent=2)+'\n'
 if row['arm'] in SKILLS:files[SKILLS[row['arm']]]=(ROOT/SKILLS[row['arm']]).read_text()
 elif row['arm']!='control':raise ValueError(row['arm'])
 return files

def guide(language):return D.guide(language).replace('no older SKILL.md is installed','the broad lobo-programming SKILL.md is not installed')
def prompt(arm):
 text=D._PROMPT_COMMON
 if arm in SKILLS:text+='Use $'+('lobo-development' if arm=='original' else 'lobo-development-v2')+' at '+SKILLS[arm]+'.\n'
 elif arm!='control':raise ValueError(arm)
 return text

def schedule():
 rows=[]
 for repeat in range(2):
  for i,task in enumerate(C.TASKS):
   order=ARMS[i:]+ARMS[:i]
   if repeat:order=order[::-1]
   for arm in order:rows.append(dict(index=len(rows),repeat=repeat,family=task,arm=arm,language='lobo',reasoning='high',phase='construction' if task=='locate' else 'change'))
 return rows

def configure():
 B.fixtures=fixtures;B.guide=guide;B.task_text=D.task_text;B.assess=D.assess;B.command=D.command

def probes(out):
 out.mkdir(parents=True,exist_ok=False);rows=[]
 programs={
  'else':('fn check(n:I64)->I64 {if n<0 {1} else if n==0 {2} else {3}}','fn check(n:I64)->I64 {if n<0 {1} else {if n==0 {2} else {3}}}',[['I64:-1'],['I64:0'],['I64:1']],[1,2,3]),
  'reserved':('fn check()->I64 {let result=6;result}','fn check()->I64 {let answer=6;answer}',[[]],[6]),
  'match':('variant Choice {Keep(I64),Skip} record Envelope {choice:Choice} fn choose(n:I64)->Choice {if n==0 {Skip} else {Keep(n)}} fn check(n:I64)->I64 {let state=Envelope {choice:choose(n)};match state.choice {Keep(x)=>x,Skip=>0}}','variant Choice {Keep(I64),Skip} record Envelope {choice:Choice} fn choose(n:I64)->Choice {if n==0 {Skip} else {Keep(n)}} fn check(n:I64)->I64 {let state=Envelope {choice:choose(n)};match (state.choice) {Keep(x)=>x,Skip=>0}}',[['I64:0'],['I64:3']],[0,3])}
 def command(cmd):
  p=subprocess.run([str(x) for x in cmd],capture_output=True,text=True,timeout=35);return dict(command=[str(x) for x in cmd],exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr)
 for name,(bad,good,args,values) in programs.items():
  for valid,body in ((False,bad),(True,good)):
   folder=out/(name+('-good' if valid else '-bad'));folder.mkdir();source=folder/'program.lobo';text='module probe;\n'+body;source.write_text(text);Path(str(source)+'.ids.json').write_text(C.identities(text,'revision.probe'))
   record=dict(probe=name,valid=valid,check=command([B.COMPILER,'check',source]));rows.append(record);B.write_json(out/'results.json',rows);assert (record['check']['exit_code']==0)==valid,record
   if valid:
    record['build']=command([B.COMPILER,'compile',source,'-o',folder/'program']);assert record['build']['exit_code']==0;record['values']=[]
    for a,wanted in zip(args,values):
     cmd=[folder/'program','--entry','revision.probe.check']
     for atom in a:cmd+=['--arg',atom]
     observed=command(cmd);record['values'].append(observed);assert observed['exit_code']==0 and json.loads(observed['stdout'])['value']==dict(type='I64',value=str(wanted))
   B.write_json(out/'results.json',rows)
 # Sync must fail without changing existing metadata, then allocate the missing declaration after repair.
 folder=out/'sync';folder.mkdir();source=folder/'program.lobo';source.write_text('module probe; record Box {value:I64} fn check()->I64 {let result=6;result}')
 meta=Path(str(source)+'.ids.json');meta.write_text(json.dumps(dict(format='lobo-identities/0.1',identities={'module':'revision.sync','check':'revision.sync.check'}))+'\n');before=meta.read_bytes();bad=command([B.COMPILER,'identities','sync',source]);assert bad['exit_code']!=0 and meta.read_bytes()==before
 (folder/'invalid-source.txt').write_text(source.read_text());source.write_text(source.read_text().replace('let result=6;result','let answer=6;answer'));good=command([B.COMPILER,'identities','sync',source]);assert good['exit_code']==0
 ids=json.loads(meta.read_text())['identities'];assert ids['module']=='revision.sync' and ids['check']=='revision.sync.check' and 'Box' in ids
 B.write_json(out/'sync-results.json',dict(failed=bad,repaired=good,old_ids_preserved=True,new_id_allocated=True));return rows

def register(out,validation):
 values=json.loads(validation.read_text());assert len(values)==6 and all(x['result']['build']['ok'] and x['result']['accepted']==(x['kind']=='reference') for x in values)
 prior=json.loads((ROOT/'research/snapshots/2026-11-24-development-study/registration/manifest.json').read_text());assert all(sha(ROOT/f)==h for f,h in prior['code'].items());assert sha(B.COMPILER)==prior['compiler_sha256']
 out.mkdir(parents=True,exist_ok=False);inputs={}
 for row in schedule():
  files=fixtures(row)|{'TASK.txt':D.task_text(row),'GUIDE.txt':guide('lobo'),'check.py':B.CHECK_CLIENT,'AGENTS.md':'This is an isolated matched experiment. Follow TASK.txt and GUIDE.txt. Only the declared algorithm files are editable. Do not inspect other directories.\n'}
  D.save_sources(out/'inputs'/row['family']/row['arm'],files);(out/('prompt-%02d.txt'%row['index'])).write_text(prompt(row['arm']));inputs[row['family']+'/'+row['arm']]=dict(files=hashed(files),prompt_sha256=hashlib.sha256(prompt(row['arm']).encode()).hexdigest())
 own=[HERE/n for n in ('skill_revision.py','skill_revision_corpus.py','test_skill_revision.py','skill_revision_report.py','test_skill_revision_report.py')]+[ROOT/SKILLS['revised'],ROOT/'.agents/skills/lobo-development-v2/agents/openai.yaml']
 code=prior['code']|{str(p.relative_to(ROOT)):sha(p) for p in own}
 manifest=dict(format='lobo-skill-revision-study/1',date='2026-11-26',model=D.MODEL,reasoning='high',scheduled=12,schedule=schedule(),inputs=inputs,code=code,material_hashes=hashed(D.materials()),compiler_sha256=sha(B.COMPILER),validation_sha256=sha(validation),probe_results_sha256=sha(PRIVATE/'probes/results.json'),sync_probe_sha256=sha(PRIVATE/'probes/sync-results.json'),skill_bytes={arm:(ROOT/p).stat().st_size for arm,p in SKILLS.items()},limits=prior['limits'],contrasts=[['control','original'],['control','revised'],['original','revised']],primary='Final native suite acceptance plus fixture, identity and budget integrity; 5 public plus65 private cases per task.',failure_policy=prior['failure_policy'],environment=D.L.environment(),compiler_regression=prior['compiler_regression'],treatment='All share same78 reference files, basic guide, tasks and grader. Original/revised add exact corresponding skill and explicit invocation; name/path/text/workflow change together. No independently identified sentence effect.',limitations=['Exploratory2 development-targeted task instances x2 repeats;4 matched assignments per arm, not powered or blinded','Fresh tasks selected after failure review; no absolute score comparison with prior tasks/dates','Configured model/reasoning/budgets held equal; weights, provider requests and hidden host instructions unknown','Provider caching and host load confound latency; no reliable speed ranking or billing/parent-token estimate','Same selected78-file packet in all arms; transitive links remain incomplete although direct skill links exist','Nullable payload normalization supports new payload-free variant; old driver and scores unchanged','No retry/replacement or fallback; original/revised skills freeze for all attempts'])
 B.write_json(out/'manifest.json',manifest);return manifest

def run(out,registration):
 reg=json.loads(registration.read_text());assert all(sha(ROOT/f)==h for f,h in reg['code'].items());assert hashed(D.materials())==reg['material_hashes'];assert sha(B.COMPILER)==reg['compiler_sha256']
 out.mkdir(parents=True,exist_ok=False);B.write_json(out/'environment-before.json',D.L.environment());configure();rows=[];code=B.run_trial.__code__;assert code.co_consts.count(D._PROMPT_COMMON)==1
 original_home=os.environ.get('CODEX_HOME')
 with tempfile.TemporaryDirectory(prefix='lobo-revision-home-') as parent:
  os.environ['CODEX_HOME']=str(D.isolated_home(Path(parent)))
  try:
   available=D.L.canary(B,'high',out/'availability');print('CANARY',available['available'],flush=True);streak=0
   for row in reg['schedule'] if available['available'] else []:
    text=prompt(row['arm']);assert hashlib.sha256(text.encode()).hexdigest()==reg['inputs'][row['family']+'/'+row['arm']]['prompt_sha256']
    trial=types.FunctionType(code.replace(co_consts=tuple(text if x==D._PROMPT_COMMON else x for x in code.co_consts)),B.__dict__)
    print('START',row,flush=True);value=trial(row,out);rows.append(value);B.write_json(out/'results.json',rows);print('END',row['index'],value['status'],value['accepted'],value['grading']['passed'],value['usage'],flush=True)
    streak=streak+1 if D.L.infrastructure_failure(value) else 0
    if streak>=3:break
   attempted={r['assignment']['index'] for r in rows};B.write_json(out/'results.json',rows);B.write_json(out/'unattempted.json',[r for r in reg['schedule'] if r['index'] not in attempted])
  finally:
   if original_home is None:os.environ.pop('CODEX_HOME',None)
   else:os.environ['CODEX_HOME']=original_home
 B.write_json(out/'environment-after.json',D.L.environment());print('FINISHED',len(rows),flush=True)

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('operation',choices=['probes','validate','register','run']);p.add_argument('--out',type=Path,required=True);p.add_argument('--validation',type=Path,default=PRIVATE/'validation/results.json');p.add_argument('--registration',type=Path,default=PRIVATE/'registration/manifest.json');a=p.parse_args()
 if a.operation=='probes':probes(a.out.resolve())
 elif a.operation=='validate':D.validate(a.out.resolve())
 elif a.operation=='register':register(a.out.resolve(),a.validation.resolve())
 else:run(a.out.resolve(),a.registration.resolve())
