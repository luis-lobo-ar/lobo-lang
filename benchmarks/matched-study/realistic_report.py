#!/usr/bin/env python3
"""Audit registered prompts, candidates, IO observations and unknown-inclusive costs."""
import importlib.util,json,statistics,shutil
from pathlib import Path
HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location('realistic_report_driver',HERE/'realistic.py');s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
spec=importlib.util.spec_from_file_location('realistic_accounting',HERE/'development_report.py');old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)

def outcome(row):return dict(timely_complete=row['accepted'],candidate_correct=row['grading']['accepted'],status=row['status'])
def contrast(rows):
 a={(r['assignment']['family'],r['assignment']['repeat']):r for r in rows if r['assignment']['arm']=='control'};b={(r['assignment']['family'],r['assignment']['repeat']):r for r in rows if r['assignment']['arm']=='v2'};pairs=[(a[k],b[k]) for k in sorted(a.keys()&b.keys())];ratios=[old.old.total(y)/old.old.total(x) for x,y in pairs if old.old.total(x) not in (None,0) and old.old.total(y) is not None]
 return dict(matched_pairs=len(pairs),complete_token_pairs=len(ratios),median_token_ratio=statistics.median(ratios) if ratios else None,token_ratios=ratios,wall_ratios=[y['agent_wall_s']/x['agent_wall_s'] for x,y in pairs],fail_to_pass=sum(not x['accepted'] and y['accepted'] for x,y in pairs),pass_to_fail=sum(x['accepted'] and not y['accepted'] for x,y in pairs),pairs=[dict(family=x['assignment']['family'],repeat=x['assignment']['repeat'],control=outcome(x),v2=outcome(y)) for x,y in pairs])

def publish():
 p=s.PRIVATE;run=p/'run';reg=json.loads((p/'registration/manifest.json').read_text());assert (run/'environment-after.json').exists();assert all(s.sha(s.ROOT/f)==h for f,h in reg['code'].items());assert s.hashed(s.D.materials())==reg['material_hashes']
 rows=json.loads((run/'results.json').read_text());target=s.ROOT/'benchmarks/fixtures/realistic-study-20261126';target.mkdir(exist_ok=False);details=[];audit=[]
 for row in rows:
  a=row['assignment'];folder=run/('trial-%02d'%a['index']);initial={str(f.relative_to(folder/'initial')):f.read_text() for f in (folder/'initial').rglob('*') if f.is_file()};pin=reg['inputs'][a['family']+'/'+a['arm']];assert s.hashed(initial)==pin['files'] and s.sha(folder/'prompt.txt')==pin['prompt_sha256'];grade=row['grading'];assert grade==json.loads((folder/'heldout/result.json').read_text());assert row['accepted']==(row['status']=='completed' and grade['accepted'] and row['command_tool_items']<=16 and row['check_requests']<=6)
  assert grade['passed']==sum(c['ok'] for c in grade['cases']);assert grade['total']==len(s.C.cases(a['family'],True))
  for i,c in enumerate(grade['cases']):assert c['input']==s.C.cases(a['family'],True)[i] and c['expected']==s.C.oracle(a['family'],c['input']) and c['ok']==(c['actual']==c['expected'])
  events=[]
  for line in (folder/'events.jsonl').read_text().splitlines():
   try:events.append(json.loads(line))
   except ValueError:pass
  assert s.B.usage(events)==row['usage'];items={e['item']['id']:e['item'] for e in events if isinstance(e.get('item'),dict) and 'id' in e['item']};commands=[x for x in items.values() if x.get('type')=='command_execution'];assert len(commands)==row['command_tool_items'];texts=[x.get('command','') for x in commands]
  changed=[];workspace=Path(row['workspace'])
  for f in workspace.rglob('*'):
   if f.is_file() and not f.is_symlink():
    name=str(f.relative_to(workspace))
    if name not in initial or s.sha(f)!=s.hashlib.sha256(initial[name].encode()).hexdigest():
     dest=folder/'candidate-final'/name;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(f.read_bytes());changed.append(dict(path=name,sha256=s.sha(f),bytes=f.stat().st_size))
  builds=[dict(stage='heldout',wall_s=grade['build']['wall_s'])] if grade.get('build') else []
  for i in range(row['check_requests']):
   f=folder/('check-%02d'%i)/'result.json'
   if f.exists():
    g=json.loads(f.read_text())
    if g.get('build'):builds.append(dict(stage='visible-'+str(i),wall_s=g['build']['wall_s']))
  detail=dict(assignment=a,**outcome(row),skill_present=s.SKILL in initial,skill_path_in_commands=any(s.SKILL in x for x in texts),external_skill_commands=[x for x in texts if old.re_external(x)],commands=texts,visible_feedback=[x['feedback'] for x in row['checks']],build_times=builds)
  details.append(detail);audit.append(dict(index=a['index'],input_prompt_grade_usage_case_labels_match=True,observed_scenarios=len(grade['cases']),executed_scenarios=sum(c['executed'] for c in grade['cases'])))
  # Public portable candidate/observations. No copied Markdown is emitted with a .md suffix.
  final={str(f.relative_to(folder/'heldout/source')):f.read_text() for f in (folder/'heldout/source').rglob('*') if f.is_file()}
  s.B.write_json(target/(folder.name+'.json'),dict(result=row,prompt=(folder/'prompt.txt').read_text(),last_message=(folder/'last-message.txt').read_text() if (folder/'last-message.txt').exists() else None,detail=detail,final_sources=final,candidate_changes=changed,raw_file_hashes={str(f.relative_to(folder)):dict(sha256=s.sha(f),bytes=f.stat().st_size) for f in folder.rglob('*') if f.is_file()}))
  dest=target/folder.name;dest.mkdir()
  for f in (folder/'heldout').glob('*.json'):shutil.copyfile(f,dest/f.name)
 canary=json.loads((run/'availability/result.json').read_text());s.B.write_json(target/'availability.json',canary)
 groups=[]
 for task in (None,*s.C.TASKS):
  for arm in s.ARMS:
   selected=[r for r in rows if r['assignment']['arm']==arm and (task is None or r['assignment']['family']==task)];groups.append(dict(family=task,arm=arm,**old.aggregate(selected),candidate_correct=sum(r['grading']['accepted'] for r in selected)))
 result=dict(format='lobo-realistic-results/1',date='2026-11-26',model=reg['model'],reasoning=reg['reasoning'],limits=reg['limits'],attempted=len(rows),scheduled=12,unattempted=json.loads((run/'unattempted.json').read_text()),groups=groups,contrasts=[dict(family=task,**contrast([r for r in rows if task is None or r['assignment']['family']==task])) for task in (None,*s.C.TASKS)],trials=rows,details=details,audits=audit,scored_usage=old.old.account(rows),all_participant_usage=old.old.account(rows+[canary]),availability=canary,limitations=reg['limitations'],registration_sha256=s.sha(p/'registration/manifest.json'),raw_evidence=str(p))
 s.B.write_json(s.ROOT/'benchmarks/results/2026-11-26-realistic-study.json',result);print(json.dumps(dict(groups=[g for g in groups if g['family'] is None],usage=result['all_participant_usage'])));return result
if __name__=='__main__':publish()
