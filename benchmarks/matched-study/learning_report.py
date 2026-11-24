#!/usr/bin/env python3
"""Publish each registered stage independently, preserving every answer and strict outcome."""
import importlib.util
import json
from pathlib import Path


def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module


HERE=Path(__file__).parent
s=load('learning',HERE/'learning.py');old=load('old_context_report',HERE/'context_report.py')


def aggregate(rows):
    return old.group(rows)|dict(format_valid=sum(r['grading']['format_valid'] for r in rows),all_task_checks_correct=sum(r['grading']['passed']==r['grading']['total'] for r in rows))


def contrast(rows,stage):
    selected=[r for r in rows if r['assignment']['stage']==stage]
    return old.paired(selected,*s.ARMS[stage])


def publish():
    root=s.ROOT;private=s.PRIVATE;run=private/'run';manifest=json.loads((private/'registration/manifest.json').read_text())
    if not (run/'environment-after.json').exists():raise ValueError('study unfinished')
    if not all(s.C.sha(root/p)==sha for p,sha in manifest['code'].items()):raise ValueError('registered code changed')
    rows=json.loads((run/'results.json').read_text());out=root/'benchmarks/fixtures/learning-study-20261124';out.mkdir(exist_ok=False);audits=[];errors=[]
    for row in rows:
        a=row['assignment'];folder=run/('trial-%02d'%a['index']);text=(folder/'prompt.txt').read_text();last=(folder/'last-message.txt').read_text() if (folder/'last-message.txt').exists() else '';answer=s.contract.decode(last);grade=s.grade(a,answer)
        accepted=row['status']=='completed' and grade['accepted'] and all(t in ('agent_message','reasoning') for t in row['item_types'])
        audit=dict(index=a['index'],prompt_matches=s.C.hash_text(text)==manifest['prompts'][str(a['index'])]['sha256'],grade_matches=grade==row['grading'],acceptance_matches=accepted==row['accepted'],tools_absent=all(t in ('agent_message','reasoning') for t in row['item_types']))
        if not all(v for k,v in audit.items() if k!='index'):raise ValueError(audit)
        audits.append(audit);expected=s.corpus.format_correct(a['family']) if a['stage']=='format' else s.corpus.micro_correct(a['family'],0 if a['stage']=='reference' else 1)
        if not grade['accepted']:errors.append(dict(assignment=a,format_errors=grade['format_errors'],incorrect_items=[k for k,v in grade['checks'].items() if not v],answer=answer,canonical_example=expected))
        raw={p.name:dict(sha256=s.C.sha(p),bytes=p.stat().st_size) for p in folder.iterdir() if p.is_file()}
        s.C.BASE.write_json(out/(folder.name+'.json'),dict(assignment=a,result=row,prompt=text,last_message=last,canonical_example=expected,raw_evidence_hashes=raw))
    canaries=[]
    for stage in s.STAGES:
        for effort in ('high','low'):
            folder=run/('availability-'+stage+'-'+effort);result=json.loads((folder/'result.json').read_text());canaries.append(result)
            s.C.BASE.write_json(out/(folder.name+'.json'),dict(result=result,raw_evidence_hashes={p.name:dict(sha256=s.C.sha(p),bytes=p.stat().st_size) for p in folder.iterdir() if p.is_file()}))
    groups=[];pairs=[]
    for stage in s.STAGES:
        for effort in (None,'high','low'):
            for language in (None,*s.C.BASE.LANGUAGES) if stage=='format' else (None,):
                selected=[r for r in rows if r['assignment']['stage']==stage and (effort is None or r['assignment']['reasoning']==effort) and (language is None or r['assignment']['language']==language)]
                for arm in s.ARMS[stage]:groups.append(dict(stage=stage,reasoning=effort,language=language,arm=arm,**aggregate([r for r in selected if r['assignment']['arm']==arm])))
                pairs.append(dict(stage=stage,reasoning=effort,language=language,**contrast(selected,stage)))
    previous=json.loads((private/'prior-paper-pins.json').read_text())
    if not all(s.C.sha(root/f['path'])==f['sha256'] for f in previous['pins']):raise ValueError('prior evidence changed')
    attempted={r['assignment']['index'] for r in rows};result=dict(format='lobo-learning-study-results/1',date='2026-11-24',model=manifest['model'],frozen_version='1.11.2',registration_sha256=s.C.sha(private/'registration/manifest.json'),reporter_sha256=s.C.sha(__file__),scheduled=len(manifest['schedule']),attempted=len(rows),unattempted=[r for r in manifest['schedule'] if r['index'] not in attempted],groups=groups,contrasts=pairs,trials=rows,strict_errors=errors,audits=audits,scored_usage=old.account(rows),excluded_availability_usage=old.account(canaries),all_participant_usage=old.account(rows+canaries),canaries=canaries,prior_paper=dict(entries=len(previous['entries']),pins=len(previous['pins']),verified=True),raw_evidence=str(private),public_bundles=str(out.relative_to(root)),environment_before=json.loads((run/'environment-before.json').read_text()),environment_after=json.loads((run/'environment-after.json').read_text()),limitations=manifest['limitations'])
    target=root/'benchmarks/results/2026-11-24-learning-study.json';s.C.BASE.write_json(target,result);print(json.dumps(dict(attempted=len(rows),accepted=sum(r['accepted'] for r in rows),failures=len(errors),known_usage=result['all_participant_usage']['known'],unknown_usage=result['all_participant_usage']['unknown'])));return result


if __name__=='__main__':publish()
