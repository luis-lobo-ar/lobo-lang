#!/usr/bin/env python3
"""Retain all compact-pilot outcomes and all three preregistered contrasts."""
import importlib.util
import json
from pathlib import Path


def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m


HERE=Path(__file__).parent;s=load('compact',HERE/'compact.py');old=load('frozen_accounting',HERE/'context_report.py')


def aggregate(rows):
    result=old.group(rows)|dict(format_valid=sum(r['grading']['format_valid'] for r in rows))
    for kind in ('predictions','admissions'):
        bits=[v for r in rows for k,v in r['grading']['checks'].items() if k.startswith(kind+'_')];result[kind+'_passed']=sum(bits);result[kind+'_total']=len(bits)
    return result


def publish():
    root=s.ROOT;p=s.PRIVATE;run=p/'run';reg=json.loads((p/'registration/manifest.json').read_text())
    if not (run/'environment-after.json').exists():raise ValueError('unfinished pilot')
    if not all(s.sha(root/path)==v for path,v in reg['code'].items()):raise ValueError('registered code changed')
    rows=json.loads((run/'results.json').read_text());target=root/'benchmarks/fixtures/compact-study-20261124';target.mkdir(exist_ok=False);audits=[];errors=[]
    for row in rows:
        a=row['assignment'];folder=run/('trial-%02d'%a['index']);text=(folder/'prompt.txt').read_text();last=(folder/'last-message.txt').read_text() if (folder/'last-message.txt').exists() else '';answer=s.L.contract.decode(last);grade=s.grade(a['family'],answer);accepted=row['status']=='completed' and grade['accepted'] and all(t in ('agent_message','reasoning') for t in row['item_types'])
        audit=dict(index=a['index'],prompt_matches=s.L.C.hash_text(text)==reg['prompts'][str(a['index'])]['sha256'],grade_matches=grade==row['grading'],acceptance_matches=accepted==row['accepted'],no_tools=all(t in ('agent_message','reasoning') for t in row['item_types']))
        if not all(v for k,v in audit.items() if k!='index'):raise ValueError(audit)
        audits.append(audit);wanted=s.correct(a['family']);actual=answer if isinstance(answer,dict) else {}
        for key,ok in grade['checks'].items():
            if ok:continue
            field,index=key.rsplit('_',1);i=int(index);values=actual.get(field);entry=dict(assignment=a,item=key,expected=wanted[field][i],actual=values[i] if isinstance(values,list) and len(values)>i else None)
            if field=='admissions':entry['checked_source']=s.corpus(a['family'])['admissions'][i]['source']
            errors.append(entry)
        s.L.C.BASE.write_json(target/(folder.name+'.json'),dict(result=row,prompt=text,last_message=last,canonical_answer=wanted,raw_evidence_hashes={f.name:dict(sha256=s.sha(f),bytes=f.stat().st_size) for f in folder.iterdir() if f.is_file()}))
    canaries=[]
    for effort in ('high','low'):
        folder=run/('availability-'+effort);value=json.loads((folder/'result.json').read_text());canaries.append(value);s.L.C.BASE.write_json(target/(folder.name+'.json'),dict(result=value,raw_evidence_hashes={f.name:s.sha(f) for f in folder.iterdir() if f.is_file()}))
    groups=[];pairs=[]
    for effort in (None,'high','low'):
        selected=[r for r in rows if effort is None or r['assignment']['reasoning']==effort]
        for arm in s.ARMS:groups.append(dict(reasoning=effort,arm=arm,**aggregate([r for r in selected if r['assignment']['arm']==arm])))
        for a,b in reg['contrasts']:pairs.append(dict(reasoning=effort,**old.paired(selected,a,b)))
    prior=json.loads((p/'prior-paper-pins.json').read_text())
    if not all(s.sha(root/f['path'])==f['sha256'] for f in prior['pins']):raise ValueError('old evidence changed')
    attempted={r['assignment']['index'] for r in rows}
    result=dict(format='lobo-compact-pilot-results/1',date='2026-11-24',registration_sha256=s.sha(p/'registration/manifest.json'),registered_model=reg['model'],scheduled=24,attempted=len(rows),unattempted=[r for r in reg['schedule'] if r['index'] not in attempted],groups=groups,contrasts=pairs,trials=rows,incorrect_items=errors,format_errors=[dict(assignment=r['assignment'],errors=r['grading']['format_errors']) for r in rows if not r['grading']['format_valid']],audits=audits,scored_usage=old.account(rows),availability_usage=old.account(canaries),all_participant_usage=old.account(rows+canaries),canaries=canaries,prior_paper=dict(entries=len(prior['entries']),pins=len(prior['pins']),verified=True),raw_evidence=str(p),public_bundles=str(target.relative_to(root)),environment_before=json.loads((run/'environment-before.json').read_text()),environment_after=json.loads((run/'environment-after.json').read_text()),limitations=reg['limitations'])
    out=root/'benchmarks/results/2026-11-24-compact-pilot.json';s.L.C.BASE.write_json(out,result);print(json.dumps(dict(attempted=len(rows),accepted=sum(r['accepted'] for r in rows),incorrect_items=len(errors),known_usage=result['all_participant_usage']['known'],unknown_usage=result['all_participant_usage']['unknown'])));return result


if __name__=='__main__':publish()
