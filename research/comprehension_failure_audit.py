#!/usr/bin/env python3
"""Offline audit of frozen comprehension responses. Makes no model calls or study changes."""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess

ROOT=Path(__file__).resolve().parents[1]
PRIVATE=ROOT/'benchmarks/private-evidence/comprehension-audit-20261123'

def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def load_study():
    path=ROOT/'benchmarks/matched-study/study.py'
    registration=json.loads((ROOT/'benchmarks/results/2026-11-23-matched-study-registration-amendment-1.json').read_text())
    if sha(path)!=registration['harness_sha256']:raise ValueError('frozen controller changed')
    spec=importlib.util.spec_from_file_location('frozen_comprehension',path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

def prompt_metrics(prompt,task,guide):
    task_count,guide_count=prompt.count(task),prompt.count(guide)
    return {'prompt_bytes':len(prompt.encode()),'task_bytes':len(task.encode()),'guide_bytes':len(guide.encode()),'task_occurrences':task_count,'guide_occurrences':guide_count,'duplicate_task_guide_bytes':max(0,task_count-1)*len(task.encode())+max(0,guide_count-1)*len(guide.encode()),'contains_check_client':'FILE check.py\n' in prompt,'contains_agent_file_copy':'FILE AGENTS.md\n' in prompt,'contains_authoring_command':'identities sync' in prompt,'token_saving':None,'interpretation':'Byte counts only. No token-saving or comprehension-improvement estimate.'}

def error_items(rows):
    result=[]
    for row in rows:
        for item,passed in row['checks'].items():
            if not passed:result.append({'configuration':row['configuration'],**row['assignment'],'item':item})
    return result

def counterparts(item,items,same_configuration=False):
    return sorted({r['language'] for r in items if r['language']!=item['language'] and (r['family'],r['item'])==(item['family'],item['item']) and (not same_configuration or r['configuration']==item['configuration'])})

def records():
    baseline=json.loads((ROOT/'benchmarks/results/2026-11-23-matched-agent-study.json').read_text());luna=json.loads((ROOT/'benchmarks/results/2026-11-23-luna-reasoning-study.json').read_text());groups=[('sol-high',baseline['attempts'],ROOT/'benchmarks/private-evidence/matched-study-20261123/attempts-amendment-1')]+[('luna-'+c['reasoning'],c['attempts'],ROOT/'benchmarks/private-evidence/luna-study-20261123/attempts'/c['reasoning']) for c in luna['cohorts']];result=[]
    for name,rows,folder in groups:
        for row in rows:
            if row['assignment']['phase']=='comprehension':
                bundle=json.loads((ROOT/row['public_bundle']).read_text());result.append({'configuration':name,'assignment':row['assignment'],'checks':row['grading']['checks'],'accepted':row['accepted'],'passed':row['grading']['passed'],'answer_text':bundle['answer'],'bundle':row['public_bundle'],'private_folder':folder/('trial-%02d'%row['assignment']['index']),'wall_s':row['agent_wall_s'],'item_types':row['item_types']})
    return result

def direct_entry(m,binary,entry,args):
    command=[str(binary),'--entry',entry,'--fuel','100000000','--heap-cells','4000000']
    for arg in args:command+=['--arg',arg]
    p=subprocess.run(command,capture_output=True,text=True,timeout=10)
    if p.returncode:raise ValueError(p.stdout+p.stderr)
    return m.untyped(json.loads(p.stdout)['value'])

def audit(out,private,snapshot):
    m=load_study();private.mkdir(parents=True,exist_ok=False);snapshot.parent.mkdir(parents=True,exist_ok=True)
    rows=records();errors=error_items(rows);builds=[];controls={};predictions=[]
    for family in m.FAMILIES:
        for language in m.LANGUAGES:
            folder=private/('base-'+family+'-'+language);files=m.sources(family,language);m.save_sources(folder,files);binary=folder/'program';build=m.build(folder,language,binary);builds.append({'family':family,'language':language,'build':build})
            if not build['ok']:raise ValueError(build)
            controls[family,language]=(folder,binary)
            for i,args in enumerate(m.prediction_inputs(family)):
                actual=m.execute(folder,language,family,args,binary);expected=m.oracle(family,args);predictions.append({'family':family,'language':language,'item':'prediction_'+str(i),'input':args,'actual':actual,'expected':expected,'ok':m.equal(actual,expected)})
    if not all(p['ok'] for p in predictions):raise ValueError('canonical prediction/oracle mismatch')
    lookup={(r['configuration'],r['assignment']['index']):r for r in rows};targets={'ledger':{'balance':7,'rejected':1,'peak':9},'scheduler':{'order':[1,0],'spent':3,'blocked':0},'frames':{'frames':['01','02'],'consumed':6,'error':0}}
    for item in errors:
        row=lookup[item['configuration'],item['index']];answer=m.parse_answer(row['answer_text']);key=item['item'];family,language=item['family'],item['language'];folder,binary=controls[family,language]
        if key.startswith('prediction_'):
            i=int(key.split('_')[1]);item['answer']=answer['predictions'][i];item['input']=m.prediction_inputs(family)[i];item['expected']=m.oracle(family,item['input']);item['canonical_actual']=m.execute(folder,language,family,item['input'],binary)
        elif key=='witness':
            item['answer']=answer.get(key);item['expected_constraints']=targets[family];item['valid_input']=m.valid_input(family,item['answer']);item['canonical_actual']=m.execute(folder,language,family,item['answer'],binary) if item['valid_input'] else None
        else:item['answer']=answer.get(key);item['expected']=m.correct_comprehension(family)[key]
        item['other_languages_with_same_failed_item']=counterparts(item,errors);item['same_configuration_other_languages']=counterparts(item,errors,True);item['source_bundle']=row['bundle']
    prompts={};prompt_observations=[];source_metrics=[]
    for row in rows:
        folder=row['private_folder'];prompt=(folder/'prompt.txt').read_text();initial=folder/'initial';task=(initial/'TASK.txt').read_text();guide=(initial/'GUIDE.txt').read_text();a=row['assignment'];key=a['family']+'/'+a['language'];observed=prompt_metrics(prompt,task,guide);prompt_observations.append({'configuration':row['configuration'],**a,**observed,'prompt_sha256':sha(folder/'prompt.txt')})
        if key not in prompts:
            sources={p.name:p.read_text() for p in initial.iterdir() if p.is_file() and p.name not in {'app.lobo','app.rs','app.py'} and '.ids.' not in p.name}
            prompts[key]={'family':a['family'],'language':a['language'],'prompt':prompt,'prompt_sha256':sha(folder/'prompt.txt'),'initial_files':sources}
            suffix={'lobo':'lobo','rust':'rs','python':'py'}[a['language']];source=(initial/('logic.'+suffix)).read_text();source_metrics.append({'family':a['family'],'language':a['language'],'logic_bytes':len(source.encode()),'logic_nonblank_lines':sum(bool(x.strip()) for x in source.splitlines()),'function_declarations':len(re.findall(r'(?m)^(?:pub )?(?:fn|def)\s+',source)),'record_or_struct_declarations':len(re.findall(r'(?m)^(?:pub )?(?:record|struct)\s+',source)),'note':'Lexical descriptions, not language-normalized complexity or token measurements.'})
        elif prompts[key]['prompt']!=prompt:raise ValueError('unexpected prompt variation for repeated assignment')
        answer=m.parse_answer(row['answer_text']);grade=m.grade_comprehension(a['family'],answer)
        if grade['checks']!=row['checks']:raise ValueError('stored answer/grader drift')
    m.write_json(snapshot,{'date':'2026-11-23','scope':'Nine unique participant user prompts and supplied source files. No hidden system instructions or model reasoning transcripts.','prompts':list(prompts.values())})
    # The omitted impact function is itself edited. Exhibit both changed outputs without altering a study file.
    folder=private/'frames-lobo-strict';files=m.sources('frames','lobo');files['logic.lobo']=m.mutant('frames','lobo');m.save_sources(folder,files);binary=folder/'program';build=m.build(folder,'lobo',binary)
    if not build['ok']:raise ValueError(build)
    base_folder,base_binary=controls['frames','lobo'];args={'data':'0000','limit':0};impact={'question_definition':'Functions in logic whose output can change for identical direct arguments; include the changed function.','base_fits':direct_entry(m,base_binary,'study.frames.fits',['I64:0','I64:0']),'strict_fits':direct_entry(m,binary,'study.frames.fits',['I64:0','I64:0']),'base_run':m.execute(base_folder,'lobo','frames',args,base_binary),'strict_run':m.execute(folder,'lobo','frames',args,binary),'direct_input':args,'expected_functions':['fits','run'],'proof_scope':'Concrete output witnesses plus reviewed source relationships, not a general formal proof.'}
    if impact['base_fits']==impact['strict_fits'] or impact['base_run']==impact['strict_run']:raise ValueError('impact witness failed')
    grouped=[]
    for config in ['sol-high','luna-high','luna-low']:
        for language in m.LANGUAGES:
            selected=[r for r in rows if r['configuration']==config and r['assignment']['language']==language];grouped.append({'configuration':config,'language':language,'attempted':len(selected),'accepted':sum(r['accepted'] for r in selected),'items_passed':sum(r['passed'] for r in selected),'items_total':11*len(selected),'failed_item_counts':dict(Counter(e['item'] for e in errors if e['configuration']==config and e['language']==language))})
    lobo_errors=[e for e in errors if e['language']=='lobo'];result={'date':'2026-11-23','format':'lobo-comprehension-failure-audit/1','scope':'Offline analysis and copied-program diagnostics only; no participant calls, changed guides, rescoring or product edits.','controller_sha256':sha(ROOT/'benchmarks/matched-study/study.py'),'analyzer_sha256':sha(__file__),'input_artifacts':[{'path':str(p.relative_to(ROOT)),'sha256':sha(p)} for p in [ROOT/'benchmarks/results/2026-11-23-matched-agent-study.json',ROOT/'benchmarks/results/2026-11-23-luna-reasoning-study.json']],'groups':grouped,'comprehension_attempts':len(rows),'failed_attempts':sum(not r['accepted'] for r in rows),'failed_items':errors,'lobo_failed_attempts':sum(not r['accepted'] for r in rows if r['assignment']['language']=='lobo'),'lobo_failed_items':len(lobo_errors),'lobo_items_also_failed_in_other_languages':sum(bool(e['other_languages_with_same_failed_item']) for e in lobo_errors),'lobo_items_also_failed_in_other_languages_same_configuration':sum(bool(e['same_configuration_other_languages']) for e in lobo_errors),'prompt_observations':prompt_observations,'prompt_snapshot':{'path':str(snapshot.relative_to(ROOT)),'sha256':sha(snapshot)},'source_metrics':source_metrics,'canonical_builds':builds,'prediction_replays':predictions,'impact_diagnostic':impact,'all_source_only_items_without_execution_tools':all(all(t in {'agent_message','reasoning'} for t in r['item_types']) for r in rows),'attribution':'Observational model/configuration sensitivity; documented prompt defects and API differences are plausible contributors. No intervention identified a causal share.','token_saving_estimate':None}
    m.write_json(out,result);return result

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',type=Path,required=True);parser.add_argument('--private-out',type=Path,default=PRIVATE);parser.add_argument('--snapshot',type=Path,default=ROOT/'research/snapshots/2026-11-23-comprehension-audit/prompts.json');args=parser.parse_args();audit(args.out.resolve(),args.private_out.resolve(),args.snapshot.resolve())
