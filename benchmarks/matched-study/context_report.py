#!/usr/bin/env python3
"""Publish all context-study outcomes with matched contrasts and explicit unknowns."""
import hashlib
import importlib.util
import itertools
import json
from pathlib import Path
import statistics

spec=importlib.util.spec_from_file_location('context_study',Path(__file__).with_name('context.py'))
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)


def stats(values):
    return dict(n=len(values),median=statistics.median(values) if values else None,minimum=min(values) if values else None,maximum=max(values) if values else None)


def total(row):
    u=row.get('usage')
    return u['input_tokens']+u['output_tokens'] if isinstance(u,dict) and all(type(u.get(k)) is int for k in ('input_tokens','output_tokens')) else None


def account(rows):
    known=[r for r in rows if total(r) is not None]
    result=dict(known=len(known),unknown=len(rows)-len(known),billing=None)
    for key in ('input_tokens','output_tokens','cached_input_tokens','reasoning_output_tokens','cache_write_input_tokens'):
        values=[r['usage'][key] for r in known if type(r['usage'].get(key)) is int]
        result[key]=sum(values) if values else None
        result[key+'_unknown']=len(rows)-len(values)
    result['reported_input_plus_output_tokens']=sum(total(r) for r in known) if known else None
    return result


def group(rows):
    return dict(attempted=len(rows),accepted=sum(r['accepted'] for r in rows),passed_items=sum(r['grading']['passed'] for r in rows),total_items=sum(r['grading']['total'] for r in rows),usage=account(rows),reported_tokens=stats([total(r) for r in rows if total(r) is not None]),wall_s=stats([r['agent_wall_s'] for r in rows]),status_counts={s:sum(r['status']==s for r in rows) for s in sorted(set(r['status'] for r in rows))})


def paired(rows,before,after):
    key=lambda r:tuple(r['assignment'][k] for k in ('reasoning','family','language'))
    a={key(r):r for r in rows if r['assignment']['arm']==before};b={key(r):r for r in rows if r['assignment']['arm']==after}
    pairs=[(a[k],b[k]) for k in sorted(a.keys()&b.keys())]
    return dict(before=before,after=after,pairs=len(pairs),pass_to_fail=sum(x['accepted'] and not y['accepted'] for x,y in pairs),fail_to_pass=sum(not x['accepted'] and y['accepted'] for x,y in pairs),item_difference=sum(y['grading']['passed']-x['grading']['passed'] for x,y in pairs),token_ratio=stats([total(y)/total(x) for x,y in pairs if total(x) not in (None,0) and total(y) is not None]),wall_ratio=stats([y['agent_wall_s']/x['agent_wall_s'] for x,y in pairs if x['agent_wall_s']>0]),pair_outcomes=[dict(reasoning=x['assignment']['reasoning'],family=x['assignment']['family'],language=x['assignment']['language'],before_accepted=x['accepted'],after_accepted=y['accepted'],item_difference=y['grading']['passed']-x['grading']['passed'],before_index=x['assignment']['index'],after_index=y['assignment']['index']) for x,y in pairs])


def publish():
    private=c.PRIVATE;run=private/'run';registration=json.loads((private/'registration/manifest.json').read_text())
    if not (run/'environment-after.json').exists():raise ValueError('run not finished')
    rows=json.loads((run/'results.json').read_text())
    if c.sha(Path(c.__file__))!=registration['driver_sha256']:raise ValueError('registered driver changed')
    audits=[];bundles=c.ROOT/'benchmarks/fixtures/context-study-20261123';bundles.mkdir(exist_ok=False)
    for row in rows:
        assignment=row['assignment'];folder=run/('trial-%02d'%assignment['index']);files={p.name:p.read_text() for p in (folder/'initial').iterdir() if p.is_file()};prompt=(folder/'prompt.txt').read_text();last=(folder/'last-message.txt').read_text() if (folder/'last-message.txt').exists() else '';key='/'.join(assignment[k] for k in ('family','language','arm'));registered=registration['inputs'][key]
        grading=c.grade(assignment['family'],c.BASE.parse_answer(last))
        audit=dict(index=assignment['index'],prompt_matches=hashlib.sha256(prompt.encode()).hexdigest()==registered['prompt_sha256'],initial_matches={k:c.hash_text(v) for k,v in files.items()}==registered['files'],grading_replay=grading==row['grading'],tools_absent=all(t in ('agent_message','reasoning') for t in row['item_types']))
        if not all(v for k,v in audit.items() if k!='index'):raise ValueError(audit)
        audits.append(audit)
        evidence={p.name:dict(sha256=c.sha(p),bytes=p.stat().st_size) for p in folder.iterdir() if p.is_file()}
        c.BASE.write_json(bundles/(folder.name+'.json'),dict(assignment=assignment,result=row,initial_files=files,prompt=prompt,last_message=last,raw_evidence_hashes=evidence))
    canaries=[]
    for effort in ('high','low'):
        folder=run/('availability-'+effort);value=json.loads((folder/'result.json').read_text());canaries.append(value)
        c.BASE.write_json(bundles/('availability-'+effort+'.json'),dict(result=value,raw_evidence_hashes={p.name:dict(sha256=c.sha(p),bytes=p.stat().st_size) for p in folder.iterdir() if p.is_file()}))
    contrasts=[('original','dedup'),('original','schema'),('original','both'),('schema','both'),('dedup','both')]
    groups=[]
    for effort in (None,'high','low'):
        for language in (None,*c.BASE.LANGUAGES):
            for arm in c.ARMS:
                selected=[r for r in rows if r['assignment']['arm']==arm and (effort is None or r['assignment']['reasoning']==effort) and (language is None or r['assignment']['language']==language)]
                groups.append(dict(reasoning=effort,language=language,arm=arm,**group(selected)))
    pairs=[]
    for effort in (None,'high','low'):
        for language in (None,*c.BASE.LANGUAGES):
            selected=[r for r in rows if (effort is None or r['assignment']['reasoning']==effort) and (language is None or r['assignment']['language']==language)]
            for a,b in contrasts:pairs.append(dict(reasoning=effort,language=language,**paired(selected,a,b)))
    failures=[]
    for row in rows:
        for key,ok in row['grading']['checks'].items():
            if not ok:failures.append(dict(assignment=row['assignment'],item=key))
    prior=json.loads((private/'prior-paper-pins.json').read_text());preserved=all(c.sha(c.ROOT/p['path'])==p['sha256'] for p in prior['pins'])
    if not preserved:raise ValueError('old paper evidence drift')
    freeze=json.loads((c.BASE.PRIVATE/'freeze/manifest.json').read_text())
    if not all(c.sha(c.BASE.PRIVATE/'freeze'/name)==v['sha256'] for name,v in freeze['files'].items()):raise ValueError('compiler freeze drift')
    planned={r['index'] for r in registration['schedule']};attempted={r['assignment']['index'] for r in rows}
    result=dict(format='lobo-context-factorial-results/1',date='2026-11-23',registered_commit='a2f5048',registration_sha256=c.sha(private/'registration/manifest.json'),driver_sha256=c.sha(c.__file__),reporter_sha256=c.sha(__file__),model=registration['model'],scheduled=len(planned),attempted=len(rows),unattempted=[r for r in registration['schedule'] if r['index'] not in attempted],aggregate=group(rows),groups=groups,contrasts=pairs,incorrect_items=failures,scored_usage=account(rows),excluded_canary_usage=account(canaries),all_participant_usage=account(rows+canaries),availability_canaries=canaries,trials=rows,prior_paper_preservation=dict(entries=prior['entries'],pins=len(prior['pins']),verified=preserved),audits=audits,raw_evidence=str(private),public_bundles=str(bundles.relative_to(c.ROOT)),environment_before=json.loads((run/'environment-before.json').read_text()),environment_after=json.loads((run/'environment-after.json').read_text()),limitations=registration['limitations'])
    target=c.ROOT/'benchmarks/results/2026-11-23-context-factorial-study.json';c.BASE.write_json(target,result);print(json.dumps(dict(attempted=len(rows),accepted=result['aggregate']['accepted'],incorrect_items=len(failures),output=str(target))))
    return result


if __name__=='__main__':publish()
