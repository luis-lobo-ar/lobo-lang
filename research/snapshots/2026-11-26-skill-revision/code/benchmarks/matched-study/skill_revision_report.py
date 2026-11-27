#!/usr/bin/env python3
"""Preserve all three arms and preregistered contrasts, including failed/unknown outcomes."""
import json,re
from pathlib import Path
import importlib.util

def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
HERE=Path(__file__).parent;s=load('reported_revision',HERE/'skill_revision.py');prior=load('original_dev_reporting',HERE/'development_report.py');aggregate=prior.aggregate;account=prior.old.account

def contrast(rows,before,after):
 selected=[r|dict(assignment=r['assignment']|dict(arm='control' if r['assignment']['arm']==before else 'skill')) for r in rows if r['assignment']['arm'] in (before,after)]
 return dict(before=before,after=after,**prior.pairs(selected))

def publish():
 p=s.PRIVATE;run=p/'run';reg=json.loads((p/'registration/manifest.json').read_text());assert (run/'environment-after.json').exists()
 assert all(s.sha(s.ROOT/f)==h for f,h in reg['code'].items()) and s.hashed(s.D.materials())==reg['material_hashes']
 rows=json.loads((run/'results.json').read_text());target=s.ROOT/'benchmarks/fixtures/skill-revision-20261126';target.mkdir(exist_ok=False);audits=[];details=[]
 for row in rows:
  a=row['assignment'];folder=run/('trial-%02d'%a['index']);initial={str(f.relative_to(folder/'initial')):f.read_text() for f in (folder/'initial').rglob('*') if f.is_file()};registered=reg['inputs'][a['family']+'/'+a['arm']]
  assert s.hashed(initial)==registered['files'] and s.sha(folder/'prompt.txt')==registered['prompt_sha256']
  grade=row['grading'];assert grade==json.loads((folder/'heldout/result.json').read_text());assert row['accepted']==(row['status']=='completed' and grade['accepted'] and row['check_requests']<=6 and row['command_tool_items']<=16)
  assert grade['passed']==sum(c['ok'] for c in grade['cases']) and grade['total']==70
  for i,c in enumerate(grade['cases']):assert c['input']==s.C.cases(a['family'],True)[i] and c['expected']==s.C.oracle(a['family'],c['input']) and c['ok']==s.B.equal(c['expected'],c['actual'])
  events=[]
  for line in (folder/'events.jsonl').read_text().splitlines():
   try:events.append(json.loads(line))
   except ValueError:pass
  assert s.B.usage(events)==row['usage'];items={e['item']['id']:e['item'] for e in events if isinstance(e.get('item'),dict) and 'id' in e['item']};commands=[v for v in items.values() if v.get('type')=='command_execution'];assert len(commands)==row['command_tool_items']
  authoring=[]
  for item in commands:
   if 'identities' not in item.get('command','') or item.get('exit_code') in (0,None):continue
   for line in item.get('aggregated_output','').splitlines():
    try:observed=json.loads(line)
    except ValueError:continue
    if isinstance(observed.get('diagnostic'),dict):authoring.append(dict(command=item['command'],diagnostic=observed['diagnostic'],exit_code=item['exit_code']))
  visible=[];first=None
  for i,c in enumerate(row['checks']):
   f=folder/('check-%02d'%i)/'source/logic.lobo';edited=f.is_file() and f.read_text()!=initial['logic.lobo'];visible.append(dict(index=i,edited=edited,feedback=c['feedback']))
   if first is None and edited and c['feedback'].get('status')=='pass':first=c['end_s']
  command_text=[c.get('command','') for c in commands]
  detail=dict(assignment=a,skill_paths_in_commands={arm:any(path in t for t in command_text) for arm,path in s.SKILLS.items()},external_skill_home_commands=[t for t in command_text if prior.re_external(t)],first_visible_edited_pass_s=first,visible=visible,authoring_errors=authoring,visible_compile_errors=sum(c['feedback'].get('status')=='compile_error' for c in row['checks']),visible_behavior_failures=sum(c['feedback'].get('status')=='fail' for c in row['checks']),commands=command_text)
  details.append(detail);audits.append(dict(index=a['index'],input_prompt_grade_usage_labels_match=True,native_observations=len(grade['cases'])))
  workspace=Path(row['workspace']);changed=[]
  for f in workspace.rglob('*'):
   if f.is_file() and not f.is_symlink():
    name=str(f.relative_to(workspace))
    if name not in initial or s.sha(f)!=s.hashlib.sha256(initial[name].encode()).hexdigest():
     dest=folder/'candidate-final'/name;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(f.read_bytes());changed.append(dict(path=name,sha256=s.sha(f),bytes=f.stat().st_size))
  final={f.name:f.read_text() for f in (folder/'heldout/source').iterdir() if f.is_file()};last=(folder/'last-message.txt').read_text() if (folder/'last-message.txt').exists() else ''
  s.B.write_json(target/(folder.name+'.json'),dict(result=row,detail=detail,prompt=(folder/'prompt.txt').read_text(),last_message=last,final_sources=final,candidate_changes=changed,raw_file_hashes={str(f.relative_to(folder)):dict(sha256=s.sha(f),bytes=f.stat().st_size) for f in folder.rglob('*') if f.is_file()}))
 canary=json.loads((run/'availability/result.json').read_text());s.B.write_json(target/'availability.json',canary)
 groups=[dict(family=task,arm=arm,**aggregate([r for r in rows if r['assignment']['arm']==arm and (task is None or r['assignment']['family']==task)])) for task in (None,*s.C.TASKS) for arm in s.ARMS]
 result=dict(format='lobo-skill-revision-results/1',date='2026-11-26',model=reg['model'],reasoning=reg['reasoning'],scheduled=12,attempted=len(rows),unattempted=json.loads((run/'unattempted.json').read_text()),groups=groups,contrasts=[dict(family=task,**contrast([r for r in rows if task is None or r['assignment']['family']==task],before,after)) for task in (None,*s.C.TASKS) for before,after in reg['contrasts']],trials=rows,details=details,audits=audits,scored_usage=account(rows),all_participant_usage=account(rows+[canary]),availability=canary,registration_sha256=s.sha(p/'registration/manifest.json'),limits=reg['limits'],limitations=reg['limitations'],raw_evidence=str(p))
 s.B.write_json(s.ROOT/'benchmarks/results/2026-11-26-skill-revision-study.json',result);print(json.dumps(dict(attempted=len(rows),accepted=sum(r['accepted'] for r in rows),usage=result['all_participant_usage'])));return result

if __name__=='__main__':publish()
