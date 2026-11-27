#!/usr/bin/env python3
"""All four preregistered v2 contrasts, unknown-inclusive costs and independent evidence audit."""
import importlib.util,json,statistics,shutil
from pathlib import Path
HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location('v3_reporting_driver',HERE/'v3.py');S=importlib.util.module_from_spec(spec);spec.loader.exec_module(S)
spec=importlib.util.spec_from_file_location('v3_prior_reporting',HERE/'development_report.py');P=importlib.util.module_from_spec(spec);spec.loader.exec_module(P)
def outcome(r):return dict(timely_complete=r['accepted'],candidate_correct=r['grading']['accepted'],status=r['status'])
def contrast(rows,arm):
 before={(r['assignment']['family'],r['assignment']['repeat']):r for r in rows if r['assignment']['arm']=='v2'};after={(r['assignment']['family'],r['assignment']['repeat']):r for r in rows if r['assignment']['arm']==arm};pairs=[(before[k],after[k]) for k in sorted(before.keys()&after.keys())];ratios=[P.old.total(b)/P.old.total(a) for a,b in pairs if P.old.total(a) not in (None,0) and P.old.total(b) is not None]
 return dict(baseline='v2',arm=arm,matched_pairs=len(pairs),complete_token_pairs=len(ratios),median_token_ratio=statistics.median(ratios) if ratios else None,token_ratios=ratios,wall_ratios=[b['agent_wall_s']/a['agent_wall_s'] for a,b in pairs],fail_to_pass=sum(not a['accepted'] and b['accepted'] for a,b in pairs),pass_to_fail=sum(a['accepted'] and not b['accepted'] for a,b in pairs),candidate_correct_difference=sum(b['grading']['accepted']-a['grading']['accepted'] for a,b in pairs),case_difference=sum(b['grading']['passed']-a['grading']['passed'] for a,b in pairs),pairs=[dict(family=a['assignment']['family'],repeat=a['assignment']['repeat'],baseline=outcome(a),treatment=outcome(b),baseline_tokens=P.old.total(a),treatment_tokens=P.old.total(b)) for a,b in pairs])

def publish():
 p=S.PRIVATE;run=p/'run';reg=json.loads((p/'registration/manifest.json').read_text());assert (run/'environment-after.json').exists();assert all(S.sha(S.ROOT/f)==h for f,h in reg['code'].items());assert S.hashed(S.D.materials())==reg['material_hashes']
 rows=json.loads((run/'results.json').read_text());target=S.ROOT/'benchmarks/fixtures/v3-study-20261126';target.mkdir(exist_ok=False);details=[];audits=[]
 for row in rows:
  a=row['assignment'];folder=run/('trial-%02d'%a['index']);initial={str(f.relative_to(folder/'initial')):f.read_text() for f in (folder/'initial').rglob('*') if f.is_file()};pin=reg['inputs'][a['family']+'/'+a['arm']];assert S.hashed(initial)==pin['files'] and S.sha(folder/'prompt.txt')==pin['prompt_sha256'];grade=row['grading'];assert grade==json.loads((folder/'heldout/result.json').read_text());assert row['accepted']==(row['status']=='completed' and grade['accepted'] and row['command_tool_items']<=16 and row['check_requests']<=6);assert grade['passed']==sum(c['ok'] for c in grade['cases']);assert grade['total']==len(S.C.cases(a['family'],True))
  for i,c in enumerate(grade['cases']):assert c['input']==S.C.cases(a['family'],True)[i] and c['expected']==S.C.oracle(a['family'],c['input']) and c['ok']==S.strict(c['actual'],c['expected'])
  events=[]
  for line in (folder/'events.jsonl').read_text().splitlines():
   try:events.append(json.loads(line))
   except ValueError:pass
  assert S.B.usage(events)==row['usage'];items={e['item']['id']:e['item'] for e in events if isinstance(e.get('item'),dict) and 'id' in e['item']};commands=[x for x in items.values() if x.get('type')=='command_execution'];assert len(commands)==row['command_tool_items'];texts=[x.get('command','') for x in commands];authoring=[]
  for item in commands:
   if item.get('exit_code') in (0,None):continue
   for line in item.get('aggregated_output','').splitlines():
    try:value=json.loads(line)
    except ValueError:continue
    if isinstance(value,dict) and isinstance(value.get('diagnostic'),dict):authoring.append(dict(command=item.get('command'),diagnostic=value['diagnostic']))
  visible=[];first=None;previous=None
  for i,check in enumerate(row['checks']):
   f=folder/('check-%02d'%i)/'result.json';g=json.loads(f.read_text()) if f.exists() else None;source=f.parent/'source';current={n:(source/n).read_text() for n in S.H.EDITABLE} if all((source/n).exists() for n in S.H.EDITABLE) else None;edited=[n for n in ('logic.lobo','policy.lobo') if current is not None and current[n]!=initial[n]];feedback=check['feedback']
   if first is None and edited and feedback.get('status')=='pass' and check['end_s']<=120:first=check['end_s']
   visible.append(dict(index=i,start_s=check['start_s'],end_s=check['end_s'],after_deadline=check['end_s']>120,edited_files=edited,same_editable_bytes_as_previous=current==previous if previous else None,feedback=feedback,build=g.get('build') if g else None));previous=current
  workspace=Path(row['workspace']);changed=[]
  for f in workspace.rglob('*'):
   if f.is_file() and not f.is_symlink():
    name=str(f.relative_to(workspace))
    if name not in initial or S.sha(f)!=S.hashlib.sha256(initial[name].encode()).hexdigest():
     dest=folder/'candidate-final'/name;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(f.read_bytes());changed.append(dict(path=name,sha256=S.sha(f),bytes=f.stat().st_size))
  detail=dict(assignment=a,**outcome(row),skill_path_observed=any(S.SKILL in text for text in texts),external_skill_commands=[x for x in texts if P.re_external(x)],commands=texts,direct_command_diagnostics=authoring,visible=visible,first_edited_public_pass_s=first,seconds_after_first_public_pass=row['agent_wall_s']-first if first is not None else None)
  details.append(detail);audits.append(dict(index=a['index'],input_prompt_usage_acceptance_case_labels_match=True,observed_scenarios=len(grade['cases']),executed_scenarios=sum(c['executed'] for c in grade['cases'])))
  final={str(f.relative_to(folder/'heldout/source')):f.read_text() for f in (folder/'heldout/source').rglob('*') if f.is_file()};S.B.write_json(target/(folder.name+'.json'),dict(result=row,detail=detail,prompt=(folder/'prompt.txt').read_text(),last_message=(folder/'last-message.txt').read_text() if (folder/'last-message.txt').exists() else None,final_sources=final,candidate_changes=changed,raw_file_hashes={str(f.relative_to(folder)):dict(sha256=S.sha(f),bytes=f.stat().st_size) for f in folder.rglob('*') if f.is_file()}));dest=target/folder.name;dest.mkdir()
  for f in (folder/'heldout').glob('*.json'):shutil.copyfile(f,dest/f.name)
 canary=json.loads((run/'availability/result.json').read_text());S.B.write_json(target/'availability.json',canary);groups=[]
 for task in (None,*S.C.TASKS):
  for arm in S.ARMS:
   selected=[r for r in rows if r['assignment']['arm']==arm and (task is None or r['assignment']['family']==task)];groups.append(dict(family=task,arm=arm,**P.aggregate(selected),candidate_correct=sum(r['grading']['accepted'] for r in selected)))
 result=dict(format='lobo-v3-ablation-results/1',date='2026-11-26',model=reg['model'],reasoning=reg['reasoning'],limits=reg['limits'],scheduled=20,attempted=len(rows),unattempted=json.loads((run/'unattempted.json').read_text()),groups=groups,contrasts=[dict(family=task,**contrast([r for r in rows if task is None or r['assignment']['family']==task],arm)) for task in (None,*S.C.TASKS) for arm in S.ARMS if arm!='v2'],trials=rows,details=details,audits=audits,scored_usage=P.old.account(rows),all_participant_usage=P.old.account(rows+[canary]),availability=canary,registration_sha256=S.sha(p/'registration/manifest.json'),limitations=reg['limitations'],raw_evidence=str(p))
 S.B.write_json(S.ROOT/'benchmarks/results/2026-11-26-v3-study.json',result);print(json.dumps(dict(groups=[g for g in groups if g['family'] is None],usage=result['all_participant_usage'])));return result
if __name__=='__main__':publish()
