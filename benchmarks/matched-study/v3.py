#!/usr/bin/env python3
"""Single-factor v2 additions and combined v3 on matched harder native applications."""
import argparse,hashlib,importlib.util,json,os,sqlite3,subprocess,tempfile,types
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];PRIVATE=ROOT/'benchmarks/private-evidence/v3-study-20261126'
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
H=load('v3_isolated_previous',HERE/'realistic.py');D=H.D;B=H.B;C=load('v3_fresh_corpus',HERE/'v3_corpus.py');H.C=C;sha=D.sha;hashed=D.hashed
PARTS=json.loads((HERE/'v3_parts.json').read_text());ARMS=('v2','requirements','resources','finish','v3');SKILL='.agents/skills/lobo-development-study/SKILL.md'
V2=ROOT/'.agents/skills/lobo-development-v2/SKILL.md';V3=ROOT/'.agents/skills/lobo-development-v3/SKILL.md'

def skill(arm):
 assert arm in ARMS
 baseline=V2.read_text().replace('name: lobo-development-v2','name: lobo-development-study',1)
 return baseline+(''.join(PARTS.values()) if arm=='v3' else PARTS.get(arm,''))
def fixtures(row):
 files=C.files(row['family'])|D.materials();files['author.py']=H.author();files['VISIBLE.json']=json.dumps([dict(input=c,expected=C.oracle(row['family'],c)) for c in C.cases(row['family'])],ensure_ascii=True,indent=2)+'\n';files[SKILL]=skill(row['arm']);return files

def prompt(arm):
 assert arm in ARMS
 return D._PROMPT_COMMON+'Use $lobo-development-study at '+SKILL+'.\n'
def guide(_):return H.guide('lobo')
def task_text(row):return H.task_text(row)
def schedule():
 rows=[]
 for repeat in range(2):
  for task_index,task in enumerate(C.TASKS):
   offset=(2*repeat+task_index)%len(ARMS);order=ARMS[offset:]+ARMS[:offset]
   if repeat:order=order[::-1]
   for arm in order:rows.append(dict(index=len(rows),repeat=repeat,family=task,arm=arm,language='lobo',reasoning='high',phase='application-change'))
 return rows

def strict(a,b):
 if type(a) is not type(b):return False
 if isinstance(a,dict):return a.keys()==b.keys() and all(strict(v,b[k]) for k,v in a.items())
 if isinstance(a,list):return len(a)==len(b) and all(strict(x,y) for x,y in zip(a,b))
 return a==b
# Reuse exact native HTTP request/response implementation, replacing only the task entry identity.
http_code=H.http_cases.__code__;assert http_code.co_consts.count('realistic.catalog.app.main')==1
http_cases=types.FunctionType(http_code.replace(co_consts=tuple('realistic.feed.app.main' if x=='realistic.catalog.app.main' else x for x in http_code.co_consts)),H.__dict__)

def sql_cases(program,cases,out):
 results=[]
 for index,c in enumerate(cases):
  path=out/('database-%02d.sqlite'%index)
  with sqlite3.connect(path) as db:
   db.executescript("CREATE TABLE inventory(sku INTEGER PRIMARY KEY,available INTEGER NOT NULL CHECK(available>=0 AND available<=1000000)); CREATE TABLE requests(key TEXT PRIMARY KEY CHECK(key!='reject'),sku1 INTEGER,qty1 INTEGER,sku2 INTEGER,qty2 INTEGER); PRAGMA ignore_check_constraints=ON;");db.executemany('INSERT INTO inventory VALUES(?,?)',c['inventory'])
  expected=C.sql_oracle(c);actual=[];calls=[];error=None
  for operation in c['operations']:
   sku1,qty1,sku2,qty2,key=operation;cmd=[program,'--entry','realistic.reservation.app.main']
   for atom in (f'I64:{sku1}',f'I64:{qty1}',f'I64:{sku2}',f'I64:{qty2}','Text:'+key.encode().hex()):cmd+=['--arg',atom]
   cmd+=['--sqlite-file',path,'--sqlite-mode','write','--sqlite-calls','1024','--sqlite-steps','1000000','--sqlite-rows','128','--sqlite-handles','3','--fuel','100000000','--heap-cells','16000000']
   try:
    call=H.invoke(cmd);value=json.loads(call['stdout']);normal=H.R.normalize(value['value']) if call['exit_code']==0 and value.get('kind')=='value' else None
    with sqlite3.connect(path) as db:inventory=[list(x) for x in db.execute('SELECT sku,available FROM inventory ORDER BY sku')];requests=[list(x) for x in db.execute('SELECT key,sku1,qty1,sku2,qty2 FROM requests ORDER BY key')]
    actual.append(dict(reply=normal['payload']['outcome'],clean=normal['payload']['clean'],inventory=inventory,requests=requests) if normal and normal['case']=='Ok' else dict(outer=normal,inventory=inventory,requests=requests));calls.append(call)
   except Exception as e:error=repr(e);break
  results.append(dict(input=c,expected=expected,actual=actual,error=error,ok=strict(actual,expected),executed=len(calls)==len(c['operations'])));B.write_json(out/('operations-%02d.json'%index),calls)
 return results

def assess(folder,fixed,row,out,hidden=False):
 out.mkdir(parents=True,exist_ok=False);issues=H.integrity(folder,fixed);files=C.files(row['family'])
 for name in H.EDITABLE:
  p=folder/name
  if p.is_file() and not p.is_symlink() and p.stat().st_size<=65536:files[name]=p.read_text()
 B.save_sources(out/'source',files);result=dict(issues=issues,build=None,cases=[],accepted=False);cases=C.cases(row['family'],hidden)
 if not issues:
  sources=C.sources(row['family']);cmd=[B.COMPILER,'compile',out/'source'/sources[0]]
  for source in sources[1:]:cmd+=['--source',out/'source'/source]
  cmd+=['-o',out/'program']
  try:
   build=H.invoke(cmd,35);result['build']=build|dict(ok=build['exit_code']==0)
   if result['build']['ok']:
    result['cases']=(http_cases if row['family']=='feed' else sql_cases)(out/'program',cases,out)
    for case in result['cases']:case['ok']=strict(case['expected'],case['actual'])
    result['accepted']=len(result['cases'])==len(cases) and all(c['ok'] for c in result['cases'])
  except Exception as e:result['infrastructure_error']=repr(e)
 result.update(passed=sum(c['ok'] for c in result['cases']),total=len(cases));B.write_json(out/'result.json',result);return result

def configure():B.fixtures=fixtures;B.guide=guide;B.task_text=task_text;B.assess=assess;B.command=D.command

def validate(out):
 out.mkdir(parents=True,exist_ok=False);rows=[]
 for task,kinds in (('feed',('reference','starter','duplicate-mutant','cursor-mutant')),('reservation',('reference','starter','aggregate-mutant','cleanup-mutant','second-mutant'))):
  for kind in kinds:
   folder=out/(task+'-'+kind);files=C.files(task,kind);B.save_sources(folder,files);result=assess(folder,files,dict(family=task),out/(task+'-'+kind+'-check'),True);rows.append(dict(task=task,kind=kind,result=result));B.write_json(out/'results.json',rows)
   assert result['build'] and result['build']['ok'] and result['accepted']==(kind=='reference'),rows[-1]
 return rows

def register(out,validation):
 values=json.loads(validation.read_text());assert len(values)==9 and all(v['result']['build']['ok'] and v['result']['accepted']==(v['kind']=='reference') for v in values)
 old=json.loads((ROOT/'research/snapshots/2026-11-26-realistic-study/registration/manifest.json').read_text());assert all(sha(ROOT/p)==h for p,h in old['code'].items());assert sha(B.COMPILER)==old['compiler_sha256'];assert V3.read_text().replace('name: lobo-development-v3','name: lobo-development-study',1)==skill('v3')
 out.mkdir(parents=True,exist_ok=False);inputs={}
 for row in schedule():
  files=fixtures(row)|H.EXTRAS|{'TASK.txt':task_text(row),'GUIDE.txt':guide('lobo')};B.save_sources(out/'inputs'/row['family']/row['arm'],files);(out/('prompt-%02d.txt'%row['index'])).write_text(prompt(row['arm']));inputs[row['family']+'/'+row['arm']]=dict(files=hashed(files),prompt_sha256=hashlib.sha256(prompt(row['arm']).encode()).hexdigest())
 paths=[HERE/n for n in ('v3.py','v3_corpus.py','v3_parts.json','test_v3.py','v3_report.py','test_v3_report.py')]+[V3,V3.parent/'agents/openai.yaml']
 manifest=dict(format='lobo-v3-ablation-study/1',date='2026-11-26',model=D.MODEL,reasoning='high',scheduled=20,schedule=schedule(),inputs=inputs,code=old['code']|{str(p.relative_to(ROOT)):sha(p) for p in paths},material_hashes=hashed(D.materials()),compiler_sha256=sha(B.COMPILER),validation_sha256=sha(validation),v2_sha256=sha(V2),v3_sha256=sha(V3),skill_bytes={arm:len(skill(arm).encode()) for arm in ARMS},limits=old['limits'],environment=D.L.environment(),compiler_regression=old['compiler_regression'],failure_policy=old['failure_policy'],contrasts=[['v2',arm] for arm in ARMS if arm!='v2'],primary='Timely completion AND final native acceptance AND fixture/ID integrity. Candidate correctness and software requirement coverage separate.5public+35private feed;5public+25private reservation; repeat per assignment, not700 independent tasks.',treatment='All arms same experimental skill name/path/frontmatter/prompt and v2 body; only selected appended requirement/resource/finish block differs. Full v3 has all3, in fixed order. Frozen original v2 remains unchanged; alias rename common to all arms.',limitations=['Exploratory2 new authored harder application tasks x2 repeats;4 matched assignments per contrast, not powered/blinded','Four blocks cannot balance all5 order positions; fixed rotation/reversal before calls. Host load/cache confound timing','Task hardness is defined by additional requirements/mutations, not preselected model failures; no post-outcome simplification','5 of8 possible factors: individual additions and combined tested, interactions not fully identified','No no-skill arm; baseline is frozen v2 body at common experimental alias; name/salience held equal, extra text length part of treatment','Same Luna/high120s/16commands/6checks, no hard token cap; SQLite full engine build inside window,35s compile/40s client ceiling, no cached builds','Pending public controller work may drain after agent termination without granting more usable time; final grading after window','78common reference files have known incomplete transitive links; direct skill links present. Provider weights/hidden instructions unknown','No retries/replacements/fallback/post-outcome edits; unknown token usage not imputed; no billing/parent development cost estimate','Independent strict-type Python oracles inspect real HTTP/SQLite; finite cases are not formal proof or production security/concurrency coverage'])
 B.write_json(out/'manifest.json',manifest);return manifest

def run(out,registration):
 reg=json.loads(registration.read_text());assert all(sha(ROOT/p)==h for p,h in reg['code'].items());assert hashed(D.materials())==reg['material_hashes'] and sha(B.COMPILER)==reg['compiler_sha256']
 out.mkdir(parents=True,exist_ok=False);B.write_json(out/'environment-before.json',D.L.environment());configure();rows=[];code=B.run_trial.__code__;assert code.co_consts.count(D._PROMPT_COMMON)==1
 previous=os.environ.get('CODEX_HOME')
 with tempfile.TemporaryDirectory(prefix='lobo-v3-home-') as parent:
  os.environ['CODEX_HOME']=str(D.isolated_home(Path(parent)))
  try:
   availability=D.L.canary(B,'high',out/'availability');print('CANARY',availability['available'],flush=True);streak=0
   for row in reg['schedule'] if availability['available'] else []:
    text=prompt(row['arm']);assert hashlib.sha256(text.encode()).hexdigest()==reg['inputs'][row['family']+'/'+row['arm']]['prompt_sha256'];trial=types.FunctionType(code.replace(co_consts=tuple(text if x==D._PROMPT_COMMON else x for x in code.co_consts)),B.__dict__)
    print('START',row,flush=True);value=trial(row,out);rows.append(value);B.write_json(out/'results.json',rows);print('END',row['index'],value['status'],value['accepted'],value['grading']['passed'],value['usage'],flush=True);streak=streak+1 if D.L.infrastructure_failure(value) else 0
    if streak>=3:break
   attempted={r['assignment']['index'] for r in rows};B.write_json(out/'results.json',rows);B.write_json(out/'unattempted.json',[r for r in reg['schedule'] if r['index'] not in attempted])
  finally:
   if previous is None:os.environ.pop('CODEX_HOME',None)
   else:os.environ['CODEX_HOME']=previous
 B.write_json(out/'environment-after.json',D.L.environment());print('FINISHED',len(rows),flush=True)

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('operation',choices=['validate','register','run']);p.add_argument('--out',type=Path,required=True);p.add_argument('--validation',type=Path,default=PRIVATE/'validation/results.json');p.add_argument('--registration',type=Path,default=PRIVATE/'registration/manifest.json');a=p.parse_args()
 if a.operation=='validate':validate(a.out.resolve())
 elif a.operation=='register':register(a.out.resolve(),a.validation.resolve())
 else:run(a.out.resolve(),a.registration.resolve())
