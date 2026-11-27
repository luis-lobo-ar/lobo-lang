#!/usr/bin/env python3
"""Registered application-level skill/control experiment, unchanged process and budgets."""
import argparse,hashlib,importlib.util,json,os,socket,sqlite3,subprocess,tempfile,time,types
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];PRIVATE=ROOT/'benchmarks/private-evidence/realistic-study-20261126'
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
R=load('realistic_old_revision',HERE/'skill_revision.py');D=R.D;B=R.B;C=load('realistic_corpus',HERE/'realistic_corpus.py');sha=D.sha;hashed=D.hashed
SKILL='.agents/skills/lobo-development-v2/SKILL.md';ARMS=('control','v2');EDITABLE=('logic.lobo','logic.lobo.ids.json','policy.lobo','policy.lobo.ids.json')
EXTRAS={'AGENTS.md':'This is an isolated matched experiment. Follow TASK.txt and GUIDE.txt. Only the declared algorithm files are editable. Do not inspect other directories.\n','check.py':B.CHECK_CLIENT}
def author():return '''import json,subprocess
from pathlib import Path
compiler='''+repr(str(B.COMPILER))+'''
sources=json.loads(Path('BUNDLE.json').read_text())
for name in ('policy.lobo','logic.lobo'):
 command=[compiler,'identities','sync',name]
 for source in sources:
  if source not in ('app.lobo',name) and not(name=='policy.lobo' and source=='logic.lobo'):command+=['--source',source]
 p=subprocess.run(command)
 if p.returncode:raise SystemExit(p.returncode)
'''
def fixtures(row):
 files=C.files(row['family'])|D.materials();files['author.py']=author();files['VISIBLE.json']=json.dumps([dict(input=c,expected=C.oracle(row['family'],c)) for c in C.cases(row['family'])],ensure_ascii=True,indent=2)+'\n'
 if row['arm']=='v2':files[SKILL]=(ROOT/SKILL).read_text()
 elif row['arm']!='control':raise ValueError(row['arm'])
 return files

def guide(_):
 body=D.guide('lobo');start=body.index('Each block returns');end=body.index('Common reference files:')
 return '''Edit only logic.lobo, policy.lobo and their adjacent .ids.json files. Preserve existing declaration IDs. Optional probes go under scratch/. app/model/lib/BUNDLE/author/client/reference files are read-only. Do not inspect other trials, solutions, compiler binary contents/parent directories, the actual repository or external skill homes. No network/dependencies from agent commands. Only the trusted checker launches native application IO with explicit loopback/SQLite grants; application functions may use the supplied declared actions. Use python3 author.py to synchronize both editable identity files after adding declarations (counts as a command). BUNDLE.json explicitly lists compilation sources; normative libraries/ are reference copies, not additional sources. Use python3 check.py for up to6 public native submissions. All compilation, retrieval and testing count inside120 seconds and16 command-execution items. SQLite builds include the engine and take about26 seconds; compile ceiling35 seconds, checker response ceiling40 seconds. Final private grading is outside the agent window. All limits apply equally.\n'''+body[start:end].replace('the supplied util.reverse<T> restores order and may infer T','a local reverse helper can restore order')+'''\nReferences: spec/, libraries/, .agents/skills/lobo-programming/references/ and docs/agent-reference/compact-pilot-2026-11-24/. The broad programming skill is not installed. Use relevant reference files; no external docs.\n'''
def task_text(row):return C.REQUIREMENTS[row['family']]+'\n\nRead GUIDE.txt. Public native scenarios and independent expected results are in VISIBLE.json. Final grading includes these and private scenarios within the same requirements. Multiple calls in a SQLite scenario share a DB; HEAD is checked against a paired real GET. Existing scaffold is executable but deliberately incomplete. Only the four files declared in GUIDE are editable.\n'
def prompt(arm):
 assert arm in ARMS
 return D._PROMPT_COMMON+('Use $lobo-development-v2 at '+SKILL+'.\n' if arm=='v2' else '')
def schedule():
 rows=[]
 for repeat in range(3):
  for i,task in enumerate(C.TASKS):
   for arm in ARMS if (repeat+i)%2==0 else ARMS[::-1]:rows.append(dict(index=len(rows),repeat=repeat,family=task,arm=arm,language='lobo',reasoning='high',phase='application-change'))
 return rows

def integrity(folder,fixed):
 issues=[]
 for name,text in fixed.items():
  p=folder/name
  if p.is_symlink():issues.append('symlink:'+name)
  elif not p.is_file():issues.append('missing:'+name)
  elif name not in EDITABLE and p.read_text()!=text:issues.append('modified:'+name)
  elif name in EDITABLE and p.stat().st_size>65536:issues.append('oversize:'+name)
 for name in ('logic.lobo.ids.json','policy.lobo.ids.json'):
  try:
   before=json.loads(fixed[name])['identities'];after=json.loads((folder/name).read_text())['identities'];issues.extend('changed_identity:'+name+':'+k for k,v in before.items() if after.get(k)!=v)
  except (OSError,ValueError,KeyError,TypeError):issues.append('invalid_identity:'+name)
 for p in folder.rglob('*'):
  name=str(p.relative_to(folder))
  if p.is_symlink():issues.append('symlink:'+name)
  elif p.is_file() and name not in fixed and not name.startswith(('.check-io/','scratch/')):issues.append('unexpected:'+name)
 return sorted(set(issues))

def invoke(cmd,timeout=5):
 started=time.monotonic();p=subprocess.run([str(x) for x in cmd],capture_output=True,text=True,timeout=timeout)
 return dict(command=[str(x) for x in cmd],exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr,wall_s=time.monotonic()-started)
def request(port,method,path):
 with socket.create_connection(('127.0.0.1',port),timeout=2) as connection:
  connection.sendall((method+' '+path+' HTTP/1.1\r\nHost: localhost\r\n\r\n').encode());connection.shutdown(socket.SHUT_WR);chunks=[]
  while True:
   data=connection.recv(65536)
   if not data:break
   chunks.append(data)
 raw=b''.join(chunks);headers,body=raw.split(b'\r\n\r\n',1);lines=headers.decode('ascii').split('\r\n');fields={k.lower():v.strip() for k,v in (line.split(':',1) for line in lines[1:])}
 return dict(status=int(lines[0].split()[1]),headers=fields,body_hex=body.hex(),wire_hex=raw.hex())
def http_cases(program,cases,out):
 with socket.socket() as reserve:reserve.bind(('127.0.0.1',0));port=reserve.getsockname()[1]
 count=1+len(cases)+sum(c['method']=='HEAD' for c in cases)
 cmd=[str(program),'--entry','realistic.catalog.app.main','--arg',f'I64:{port}','--arg',f'I64:{count}','--native-clock','true','--native-listen',f'127.0.0.1:{port}','--native-calls','65536','--native-bytes','1048576','--fuel','100000000','--heap-cells','16000000']
 proc=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True);results=[];observations=[];ready=False;start=time.monotonic()
 try:
  while time.monotonic()-start<3 and proc.poll() is None:
   try:
    probe=request(port,'GET','/health');ready=True;break
   except ConnectionRefusedError:time.sleep(.01)
  fatal=None if ready else 'server_not_ready'
  for c in cases:
   expected=C.http_oracle(c);wire=None;shadow=None;actual=None;error=fatal
   if fatal is None:
    try:
     if c['method']=='HEAD':shadow=request(port,'GET',c['path'])
     wire=request(port,c['method'],c['path']);data=bytes.fromhex((shadow or wire)['body_hex']);head=wire['headers'];actual=dict(status=wire['status'],body=json.loads(data),head_empty=wire['body_hex']=='' if shadow else True,length_ok=int(head['content-length'])==len(data),content_type=head.get('content-type'),allow=head.get('allow'))
     if shadow and (shadow['status']!=wire['status'] or shadow['headers']!=wire['headers']):actual['length_ok']=False
    except Exception as e:error=repr(e);fatal='earlier_transport_failure'
   results.append(dict(input=c,expected=expected,actual=actual,error=error,ok=actual==expected,executed=wire is not None));observations.append(dict(wire=wire,shadow=shadow))
  stdout,stderr=proc.communicate(timeout=3)
 except Exception as e:
  proc.kill();stdout,stderr=proc.communicate();fatal=repr(e)
 finally:
  if proc.poll() is None:proc.kill();proc.wait()
 B.write_json(out/'transport.json',dict(command=cmd,port=port,planned_connections=count,startup_probe=locals().get('probe'),observations=observations,exit_code=proc.returncode,stdout=stdout,stderr=stderr,fatal=fatal))
 return results

def sql_cases(program,cases,out):
 results=[]
 for index,c in enumerate(cases):
  path=out/('database-%02d.sqlite'%index)
  with sqlite3.connect(path) as db:
   db.executescript('CREATE TABLE accounts(id INTEGER PRIMARY KEY,balance INTEGER NOT NULL CHECK(balance>=0 AND balance<=1000000)); CREATE TABLE requests(key TEXT PRIMARY KEY,source_id INTEGER,target_id INTEGER,amount INTEGER); PRAGMA ignore_check_constraints=ON;');db.executemany('INSERT INTO accounts VALUES(?,?)',c['accounts'])
  expected=C.sql_oracle(c);actual=[];calls=[];error=None
  for operation in c['operations']:
   source,target,amount,key=operation;cmd=[program,'--entry','realistic.transfer.app.main','--arg',f'I64:{source}','--arg',f'I64:{target}','--arg',f'I64:{amount}','--arg','Text:'+key.encode().hex(),'--sqlite-file',path,'--sqlite-mode','write','--sqlite-calls','1024','--sqlite-steps','1000000','--sqlite-rows','128','--sqlite-handles','3','--fuel','100000000','--heap-cells','16000000']
   try:
    call=invoke(cmd);value=json.loads(call['stdout']);normal=R.normalize(value['value']) if call['exit_code']==0 and value.get('kind')=='value' else None
    with sqlite3.connect(path) as db:accounts=[list(x) for x in db.execute('SELECT id,balance FROM accounts ORDER BY id')];requests=[list(x) for x in db.execute('SELECT key,source_id,target_id,amount FROM requests ORDER BY key')]
    actual.append(dict(reply=normal['payload']['outcome'],clean=normal['payload']['clean'],accounts=accounts,requests=requests) if normal and normal['case']=='Ok' else dict(outer=normal,accounts=accounts,requests=requests));calls.append(call)
   except Exception as e:error=repr(e);break
  results.append(dict(input=c,expected=expected,actual=actual,error=error,ok=actual==expected,executed=len(calls)==len(c['operations'])));B.write_json(out/('operations-%02d.json'%index),calls)
 return results

def assess(folder,fixed,row,out,hidden=False):
 out.mkdir(parents=True,exist_ok=False);issues=integrity(folder,fixed);files=C.files(row['family'])
 for name in EDITABLE:
  p=folder/name
  if p.is_file() and not p.is_symlink() and p.stat().st_size<=65536:files[name]=p.read_text()
 B.save_sources(out/'source',files);result=dict(issues=issues,build=None,cases=[],accepted=False);cases=C.cases(row['family'],hidden)
 if not issues:
  sources=C.sources(row['family']);cmd=[B.COMPILER,'compile',out/'source'/sources[0]]
  for source in sources[1:]:cmd+=['--source',out/'source'/source]
  cmd+=['-o',out/'program']
  try:
   build=invoke(cmd,35);result['build']=build|dict(ok=build['exit_code']==0)
   if result['build']['ok']:result['cases']=(http_cases if row['family']=='catalog' else sql_cases)(out/'program',cases,out);result['accepted']=len(result['cases'])==len(cases) and all(c['ok'] for c in result['cases'])
  except Exception as e:result['infrastructure_error']=repr(e)
 result.update(passed=sum(c['ok'] for c in result['cases']),total=len(cases));B.write_json(out/'result.json',result);return result

def configure():B.fixtures=fixtures;B.guide=guide;B.task_text=task_text;B.assess=assess;B.command=D.command

def validate(out):
 out.mkdir(parents=True,exist_ok=False);rows=[]
 for task in C.TASKS:
  for kind in ('reference','starter','mutant')+(('cleanup-mutant',) if task=='transfer' else ()):
   folder=out/(task+'-'+kind);files=C.files(task,kind);B.save_sources(folder,files);result=assess(folder,files,dict(family=task),out/(task+'-'+kind+'-check'),True);rows.append(dict(task=task,kind=kind,result=result));B.write_json(out/'results.json',rows)
   assert result['build'] and result['build']['ok'] and result['accepted']==(kind=='reference'),rows[-1]
 return rows

def register(out,validation):
 values=json.loads(validation.read_text());assert len(values)==7 and all(v['result']['build']['ok'] and v['result']['accepted']==(v['kind']=='reference') for v in values)
 old=json.loads((ROOT/'research/snapshots/2026-11-26-skill-revision/registration/manifest.json').read_text());assert all(sha(ROOT/p)==h for p,h in old['code'].items());assert sha(B.COMPILER)==old['compiler_sha256']
 out.mkdir(parents=True,exist_ok=False);inputs={}
 for row in schedule():
  files=fixtures(row)|EXTRAS|{'TASK.txt':task_text(row),'GUIDE.txt':guide('lobo')};B.save_sources(out/'inputs'/row['family']/row['arm'],files);(out/('prompt-%02d.txt'%row['index'])).write_text(prompt(row['arm']));inputs[row['family']+'/'+row['arm']]=dict(files=hashed(files),prompt_sha256=hashlib.sha256(prompt(row['arm']).encode()).hexdigest())
 paths=[HERE/n for n in ('realistic.py','realistic_corpus.py','test_realistic.py','realistic_report.py','test_realistic_report.py')]
 manifest=dict(format='lobo-realistic-study/1',date='2026-11-26',model=D.MODEL,reasoning='high',scheduled=12,schedule=schedule(),inputs=inputs,code=old['code']|{str(p.relative_to(ROOT)):sha(p) for p in paths},material_hashes=hashed(D.materials()),compiler_sha256=sha(B.COMPILER),validation_sha256=sha(validation),limits=old['limits'],environment=D.L.environment(),compiler_regression=old['compiler_regression'],failure_policy=old['failure_policy'],primary='Completion inside unchanged budgets plus fixture/ID integrity and final native suite. Catalog30 scenarios (5public25private), transfer20 (5public15private); per-task same cases reused across arms/repeats. Separate candidate correctness from timely completion.',treatment='Only exact frozen v2 SKILL.md and its explicit prompt invocation differ. Both arms get same78 references, application scaffolds,4 editable files, author helper and native feedback.',limitations=['Exploratory2 authored application changes x3 repeats,6 matched pairs, not powered or blinded; selected after earlier failures','Real loopback HTTP and SQLite stateful operations; no production TLS/auth/concurrency/deployment coverage','Same model identifier/high/budgets; immutable weights, provider requests and hidden instructions unknown','SQLite engine compilation approximately26s inside120s window; no precompiled cache,35s compile/40s client ceiling. Final grading outside window','Shared host load/provider caching; no billing estimate or causal wall-time ranking','No retry/replacement/fallback/skill edits; unknown terminal usage preserved','Common78 references have incomplete transitive links; direct skill links exist. Bundle isolates executable library copies from normative references','No absolute comparison across older/different tasks; task repetitions reuse cases, not independent software projects','Ephemeral port selection has a small bind race; infrastructure errors retained and not replaced'])
 B.write_json(out/'manifest.json',manifest);return manifest

def run(out,registration):
 reg=json.loads(registration.read_text());assert all(sha(ROOT/p)==h for p,h in reg['code'].items());assert hashed(D.materials())==reg['material_hashes'] and sha(B.COMPILER)==reg['compiler_sha256']
 out.mkdir(parents=True,exist_ok=False);B.write_json(out/'environment-before.json',D.L.environment());configure();rows=[];code=B.run_trial.__code__;assert code.co_consts.count(D._PROMPT_COMMON)==1
 previous=os.environ.get('CODEX_HOME')
 with tempfile.TemporaryDirectory(prefix='lobo-realistic-home-') as parent:
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
