#!/usr/bin/env python3
"""Derived diagnostics for preregistered format/task metrics; never changes grades or repairs replies."""
import importlib.util
import json
from pathlib import Path

spec=importlib.util.spec_from_file_location('learning',Path(__file__).with_name('learning.py'))
s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)


def profile_pairs(rows,stage):
    selected=[r for r in rows if r['assignment']['stage']==stage];before,after=s.ARMS[stage];key=lambda r:tuple(r['assignment'][k] for k in ('reasoning','family','language'))
    a={key(r):r for r in selected if r['assignment']['arm']==before};b={key(r):r for r in selected if r['assignment']['arm']==after};pairs=[(a[k],b[k]) for k in sorted(a.keys()&b.keys())]
    result=dict(stage=stage,before=before,after=after,pairs=len(pairs))
    for name,get in [('format',lambda r:r['grading']['format_valid']),('task_checks',lambda r:r['grading']['passed']==r['grading']['total']),('acceptance',lambda r:r['accepted'])]:
        result[name+'_fail_to_pass']=sum(not get(x) and get(y) for x,y in pairs);result[name+'_pass_to_fail']=sum(get(x) and not get(y) for x,y in pairs)
    return result


def describe(family,key,value):
    valid=s.C.BASE.valid_input(family,value)
    if key=='witness':return dict(input=value,input_valid=valid,expected_target=s.corpus.format_target(family),actual=s.corpus.format_oracle(family,value) if valid else None)
    return dict(input=value,input_valid=valid,criterion='different original and mutant summaries',original=s.corpus.format_oracle(family,value) if valid else None,mutant=s.corpus.format_oracle(family,value,True) if valid else None)


def publish():
    path=s.ROOT/'benchmarks/results/2026-11-24-learning-study.json';primary=json.loads(path.read_text());failures=[]
    for row in primary['trials']:
        a=row['assignment'];last=s.PRIVATE/'run'/('trial-%02d'%a['index'])/'last-message.txt';answer=s.contract.decode(last.read_text()) if last.exists() else None;answer=answer if isinstance(answer,dict) else {}
        for key,ok in row['grading']['checks'].items():
            if ok:continue
            item=dict(assignment=a,item=key)
            if a['stage']=='format':
                if key.startswith('prediction_'):
                    i=int(key.split('_')[-1]);args=s.corpus.format_inputs(a['family'])[i];v=answer.get('predictions');item.update(input=args,expected=s.corpus.format_oracle(a['family'],args),actual=v[i] if isinstance(v,list) and len(v)>i else None)
                elif key in ('witness','counterexample'):item.update(describe(a['family'],key,answer.get(key)))
                else:item.update(expected=s.corpus.format_correct(a['family'])[key],actual=answer.get(key))
            else:
                field,index=key.rsplit('_',1);i=int(index);form=0 if a['stage']=='reference' else 1;wanted=s.corpus.micro_correct(a['family'],form)[field];v=answer.get(field);item.update(expected=wanted[i],actual=v[i] if isinstance(v,list) and len(v)>i else None)
                if field=='admissions':item['checked_source']=s.corpus.micro(a['family'],form)['admissions'][i]['source']
            failures.append(item)
    result=dict(format='lobo-learning-diagnostics/1',date='2026-11-24',primary_sha256=s.C.sha(path),script_sha256=s.C.sha(__file__),scope='Derived displays of metrics selected in registration; primary scores unchanged; no projection, retry or replacement',contrasts=[profile_pairs(primary['trials'],p) for p in s.STAGES],incorrect_items=failures,format_errors=[dict(assignment=r['assignment'],errors=r['grading']['format_errors']) for r in primary['trials'] if not r['grading']['format_valid']])
    out=s.ROOT/'benchmarks/results/2026-11-24-learning-diagnostics.json';s.C.BASE.write_json(out,result);print(json.dumps(dict(incorrect_items=len(failures),format_errors=len(result['format_errors']))));return result


if __name__=='__main__':publish()
