#!/usr/bin/env python3
"""Audit and publish the registered development comparison without changing its scores."""
import json
from pathlib import Path
from collections import Counter

import importlib.util

def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
HERE=Path(__file__).parent;s=load('reported_development',HERE/'development.py');old=load('frozen_dev_accounting',HERE/'context_report.py')

def aggregate(rows):
    return dict(attempted=len(rows),accepted=sum(r['accepted'] for r in rows),passed_cases=sum(r['grading']['passed'] for r in rows),total_cases=sum(r['grading']['total'] for r in rows),usage=old.account(rows),reported_tokens=old.stats([old.total(r) for r in rows if old.total(r) is not None]),wall_s=old.stats([r['agent_wall_s'] for r in rows]),command_items=old.stats([r['command_tool_items'] for r in rows]),visible_submissions=old.stats([r['check_requests'] for r in rows]),statuses=dict(Counter(r['status'] for r in rows)))

def pairs(rows):
    before={(r['assignment']['family'],r['assignment']['repeat']):r for r in rows if r['assignment']['arm']=='control'}
    after={(r['assignment']['family'],r['assignment']['repeat']):r for r in rows if r['assignment']['arm']=='skill'}
    values=[(before[k],after[k]) for k in sorted(before.keys()&after.keys())]
    return dict(pairs=len(values),fail_to_pass=sum(not a['accepted'] and b['accepted'] for a,b in values),pass_to_fail=sum(a['accepted'] and not b['accepted'] for a,b in values),case_difference=sum(b['grading']['passed']-a['grading']['passed'] for a,b in values),token_ratio=old.stats([old.total(b)/old.total(a) for a,b in values if old.total(a) not in (None,0) and old.total(b) is not None]),wall_ratio=old.stats([b['agent_wall_s']/a['agent_wall_s'] for a,b in values if a['agent_wall_s']>0]),outcomes=[dict(family=a['assignment']['family'],repeat=a['assignment']['repeat'],control_index=a['assignment']['index'],skill_index=b['assignment']['index'],control_accepted=a['accepted'],skill_accepted=b['accepted']) for a,b in values])

def publish():
    p=s.PRIVATE;run=p/'run';reg=json.loads((p/'registration/manifest.json').read_text());assert (run/'environment-after.json').exists()
    assert all(s.sha(s.ROOT/f)==h for f,h in reg['code'].items());assert s.hashed(s.materials())==reg['material_hashes']
    rows=json.loads((run/'results.json').read_text());target=s.ROOT/'benchmarks/fixtures/development-study-20261124';target.mkdir(exist_ok=False);audits=[];details=[]
    for r in rows:
        a=r['assignment'];folder=run/('trial-%02d'%a['index']);initial={str(f.relative_to(folder/'initial')):f.read_text() for f in (folder/'initial').rglob('*') if f.is_file()};registered=reg['inputs'][a['family']+'/'+a['arm']];prompt=(folder/'prompt.txt').read_text()
        assert s.hashed(initial)==registered['files'] and s.sha(folder/'prompt.txt')==registered['prompt_sha256']
        assert r['grading']==json.loads((folder/'heldout/result.json').read_text())
        accepted=r['status']=='completed' and r['grading']['accepted'] and r['check_requests']<=6 and r['command_tool_items']<=16;assert accepted==r['accepted']
        events=[]
        for line in (folder/'events.jsonl').read_text().splitlines():
            try:events.append(json.loads(line))
            except ValueError:pass
        assert s.B.usage(events)==r['usage']
        items={e['item']['id']:e['item'] for e in events if isinstance(e.get('item'),dict) and 'id' in e['item']};commands=[x.get('command','') for x in items.values() if x.get('type')=='command_execution']
        first=None;visible=[]
        for i,c in enumerate(r['checks']):
            check=folder/('check-%02d'%i);source=check/'source/logic.lobo';edited=source.is_file() and source.read_text()!=initial['logic.lobo'];visible.append(dict(index=i,edited=edited,feedback=c['feedback']))
            if first is None and edited and c['feedback'].get('status')=='pass':first=c['end_s']
        detail=dict(assignment=a,skill_file_present=s.SKILL in initial,skill_path_in_commands=any('lobo-development' in c for c in commands),external_skill_path_commands=[c for c in commands if re_external(c)],first_visible_edited_pass_s=first,visible_compile_errors=sum(c['feedback'].get('status')=='compile_error' for c in r['checks']),visible_behavior_failures=sum(c['feedback'].get('status')=='fail' for c in r['checks']),visible=visible,commands=commands)
        details.append(detail);audits.append(dict(index=a['index'],initial_prompt_grade_acceptance_usage_match=True))
        final={f.name:f.read_text() for f in (folder/'heldout/source').iterdir() if f.is_file()};last=(folder/'last-message.txt').read_text() if (folder/'last-message.txt').exists() else ''
        s.B.write_json(target/(folder.name+'.json'),dict(result=r,prompt=prompt,last_message=last,final_sources=final,detail=detail,raw_file_hashes={str(f.relative_to(folder)):dict(sha256=s.sha(f),bytes=f.stat().st_size) for f in folder.rglob('*') if f.is_file()}))
    canary=json.loads((run/'availability/result.json').read_text());s.B.write_json(target/'availability.json',canary)
    groups=[dict(family=task,arm=arm,**aggregate([r for r in rows if r['assignment']['arm']==arm and (task is None or r['assignment']['family']==task)])) for task in (None,*s.C.TASKS) for arm in ('control','skill')]
    result=dict(format='lobo-development-skill-study-results/1',date='2026-11-24',registration_sha256=s.sha(p/'registration/manifest.json'),model=reg['model'],reasoning=reg['reasoning'],scheduled=12,attempted=len(rows),unattempted=json.loads((run/'unattempted.json').read_text()),groups=groups,contrasts=[dict(family=task,**pairs([r for r in rows if task is None or r['assignment']['family']==task])) for task in (None,*s.C.TASKS)],trials=rows,details=details,audits=audits,all_participant_usage=old.account(rows+[canary]),scored_usage=old.account(rows),availability=canary,raw_evidence=str(p),limits=reg['limits'],limitations=reg['limitations'])
    s.B.write_json(s.ROOT/'benchmarks/results/2026-11-24-development-study.json',result);print(json.dumps(dict(attempted=len(rows),accepted=sum(r['accepted'] for r in rows),usage=result['all_participant_usage'])));return result

def re_external(command):return '/home/luis/.codex/skills' in command or '/home/luis/.agents/skills' in command

if __name__=='__main__':publish()
