#!/usr/bin/env python3
"""Posthoc response-format sensitivity and concrete failure replay; primary scores stay frozen."""
import copy
import importlib.util
import json
from pathlib import Path

spec=importlib.util.spec_from_file_location('context_study',Path(__file__).with_name('context.py'))
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)


def project(family,value):
    if not isinstance(value,dict):return value
    keys={'ledger':('initial','events'),'scheduler':('jobs','budget'),'frames':('data','limit')}[family]
    return {k:value[k] for k in keys if k in value}


def analyze():
    primary_path=c.ROOT/'benchmarks/results/2026-11-23-context-factorial-study.json'
    primary=json.loads(primary_path.read_text());rows=[];changes=[];replays=[]
    for row in primary['trials']:
        a=row['assignment'];path=c.PRIVATE/'run'/('trial-%02d'%a['index'])/'last-message.txt';answer=c.BASE.parse_answer(path.read_text());answer=copy.deepcopy(answer) if isinstance(answer,dict) else {}
        for key in ('witness','counterexample'):answer[key]=project(a['family'],answer.get(key))
        projected=c.grade(a['family'],answer)
        rows.append(dict(assignment=a,strict_accepted=row['accepted'],strict_passed=row['grading']['passed'],projected_accepted=row['status']=='completed' and projected['accepted'] and all(t in ('agent_message','reasoning') for t in row['item_types']),projected_grade=projected))
        for key,ok in projected['checks'].items():
            if ok and not row['grading']['checks'][key]:changes.append(dict(assignment=a,item=key))
    for error in primary['incorrect_items']:
        if 'input' not in error:continue
        family=error['assignment']['family'];args=project(family,error['input'])
        if not c.BASE.valid_input(family,args):continue
        for language in c.BASE.LANGUAGES:
            for mutant in ([False,True] if error['item']=='counterexample' else [False]):
                folder=c.PRIVATE/'validation'/(family+'-'+language+('-mutant' if mutant else ''));binary=folder/'program'
                actual=c.BASE.execute(folder,language,family,args,binary);expected=c.oracle(family,args,mutant)
                replay=dict(assignment=error['assignment'],item=error['item'],language=language,mutant=mutant,input=args,metadata_was_removed=args!=error['input'],expected=expected,actual=actual,ok=c.BASE.equal(actual,expected),adapter='frozen study.execute',adapter_sha256=c.sha(Path(c.BASE.__file__)),source_folder=str(folder),binary_sha256=c.sha(binary) if binary.exists() else None)
                replays.append(replay)
                if not replay['ok']:raise ValueError(replay)
    groups=[]
    for language in (None,*c.BASE.LANGUAGES):
        for arm in c.ARMS:
            selected=[r for r in rows if r['assignment']['arm']==arm and (language is None or r['assignment']['language']==language)]
            groups.append(dict(language=language,arm=arm,attempted=len(selected),strict_accepted=sum(r['strict_accepted'] for r in selected),projected_accepted=sum(r['projected_accepted'] for r in selected),strict_items=sum(r['strict_passed'] for r in selected),projected_items=sum(r['projected_grade']['passed'] for r in selected)))
    result=dict(format='lobo-context-posthoc-format-sensitivity/1',date='2026-11-23',primary_sha256=c.sha(primary_path),script_sha256=c.sha(__file__),primary_scores_unchanged=True,preregistered=False,definition='For witness/counterexample only, discard unknown top-level keys; preserve all required values and nested structure. Never add missing inputs. Keep every other strict rule.',reason='Registered grader closes top-level input maps, but delivered documentation does not explicitly reject unknown top-level properties. Quantify this interface ambiguity without replacing primary scores.',strict_accepted=sum(r['strict_accepted'] for r in rows),projected_accepted=sum(r['projected_accepted'] for r in rows),strict_passed_items=sum(r['strict_passed'] for r in rows),projected_passed_items=sum(r['projected_grade']['passed'] for r in rows),metadata_only_item_changes=changes,groups=groups,trials=rows,concrete_replays=replays,limitations=['Posthoc scoring sensitivity, not a new primary outcome or model run','Projection is not proof that a model understood the input contract','Missing required inputs, incorrect semantics and explicit instruction errors remain failures','A future response-schema treatment needs new questions and registration, not edits to this corpus'])
    out=c.ROOT/'benchmarks/results/2026-11-23-context-format-sensitivity.json';c.BASE.write_json(out,result);print(json.dumps(dict(strict=result['strict_accepted'],projected=result['projected_accepted'],metadata_only_items=len(changes),concrete_replays=len(replays),all_replays_ok=all(r['ok'] for r in replays))))
    return result


if __name__=='__main__':analyze()
