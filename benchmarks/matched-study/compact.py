#!/usr/bin/env python3
"""Small sequential guide-cleanup and targeted-fact pilot; frozen product/old studies remain unchanged."""
import argparse
import hashlib
import importlib.util
import itertools
import json
import os
from pathlib import Path
import random
import subprocess
import time

HERE=Path(__file__).parent.resolve();ROOT=HERE.parents[1]
PRIVATE=ROOT/'benchmarks/private-evidence/compact-study-20261124'


def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module


L=load('isolated_frozen_learning',HERE/'learning.py')
TOPICS=('ranges','nominal','propagation','contracts');ARMS=('legacy','clean','targeted')
PACKET_PATH=ROOT/'docs/agent-reference/compact-pilot-2026-11-24/cards.json'
PACKET=json.loads(PACKET_PATH.read_text())
LEGACY=L.C.BASE.guide('lobo')
LANGUAGE_BODY=LEGACY[LEGACY.index('Each block returns'):LEGACY.index('New functions/records/variants')]
CLEAN='READING MODE: reason from the supplied source and return the requested JSON without tools.\nLobo 1.11.2 is a native language. There is no projection/interpreter step.\n'+LANGUAGE_BODY


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def card(topic):return 'CHECKED TOPIC CARD: Lobo 1.11.2 / Core 1.11\n'+PACKET['cards'][topic]['text']+'\n'


def corpus(topic):
    if topic=='ranges':
        head='';t='"KñΩS"'
        bodies=[f'Bytes::length(Text::bytes({t}))+Text::length({t})',f'Text::at({t},2)',f'Bytes::at(Bytes::slice(Text::bytes({t}),2,2),0)',f'Text::length(Text::slice({t},1,2))','Bytes::length(Bytes::concat(Text::bytes("uv"),Text::bytes("w")))','if Text::valid(Bytes::push(Bytes::empty(),254)) {1} else {0}']
        values=[10,937,177,2,3,0]
        admissions=[('fn candidate()->Bytes {Bytes::from_list(List::prepend(260,List::empty<I64>()))}',True),('fn candidate()->I64 {Text::length(Bytes::concat(Bytes::empty(),Bytes::empty()))}',False),('fn candidate()->Text {Text::concat("a","z")}',True),('fn candidate()->Bool {Text::valid("ready")}',False)]
        background='Literal KñΩS has Unicode scalars [75,241,937,83] and UTF-8 bytes [75,195,177,206,169,83]. This input legend is supplied in every arm.'
    elif topic=='nominal':
        head='record Crate<T> {item:T}\nrecord Pair {a:I64,b:I64}\nvariant Check<T> {Waiting,Have(T)}\nfn pick(status:Check<I64>)->I64 {match status {Waiting=>-9,Have(item)=>item*2}}\nfn identity<T>(value:T)->T {value}\n'
        bodies=['let start=Pair {a:9,b:11};let finish=Pair {...start,b:17};finish.a+finish.b','let boxed=Crate {item:13};boxed.item','pick(Have(7))','pick(Waiting)','let status:Check<Crate<I64>>=Have(Crate {item:6});match status {Waiting=>0,Have(boxed)=>boxed.item+3}','let boxed=identity(Crate {item:19});boxed.item']
        values=[26,13,14,-9,9,19]
        admissions=[('fn candidate()->Check<Text> {Waiting()}',True),('record Twin {a:I64,b:I64} fn candidate()->Pair {let value=Twin {a:2,b:8};Pair {...value,b:3}}',False),('fn candidate()->Crate<Bool> {Crate {item:true}}',True),('fn make<T>()->Check<T> {Waiting} fn candidate()->Check<I64> {make()}',False)]
        background='No extra input values are required.'
    elif topic=='propagation':
        head='variant Attempt<T> {Ok(T),Err(I64)}\nfn produce(value:I64)->Attempt<I64> {if value<0 {Err(31)} else {Ok(value+4)}}\nfn unpack(item:Attempt<I64>)->I64 {match item {Ok(value)=>value,Err(code)=>-code}}\nfn flow(start:I64)->Attempt<I64> {let value=try produce(start);Ok(value+6)}\nfn scan()->Attempt<I64> {let amount=fold(i in 5,total:I64=1) {let part=try produce(1-i);total+part};Ok(amount)}\nfn choose(flag:Bool)->Attempt<I64> {if flag {Ok(23)} else {let value=try produce(-1);Ok(value)}}\n'
        bodies=['unpack(flow(3))','unpack(flow(-2))','unpack(flow(-2))+50','unpack(scan())','unpack(choose(true))','let outcome=flow(0);unpack(outcome)*2']
        values=[13,-31,19,-31,23,20]
        admissions=[('fn status()->Attempt<I64> {let value=try produce(-1);Ok(value)} fn candidate()->I64 ensures {match (status()) {Ok(value)=>value==result,Err(code)=>true}} {4}',True),('fn candidate()->Attempt<I64> requires {(try produce(2))>0} {Ok(7)}',False),('fn try(value:I64)->I64 {value*3} fn candidate()->I64 {try (4)}',True),('variant FailText {Ok(I64),Err(Text)} fn candidate()->FailText {let value=try produce(0);Ok(value)}',False)]
        background='No extra input values are required.'
    else:
        head='fn add(value:I64)->I64 requires {value>=0} ensures {result>value} {value+5}\nfn post(value:I64)->I64 ensures {result==value*2} {value*2}\n'
        bodies=['add(8)','if false {add(-7)} else {27}','if true || add(-1)>0 {17} else {0}','fold(i in 0,total:I64=12) {add(-9)}','post(9)','if false && 1/0==0 {1} else {29}']
        values=[13,27,17,12,18,29]
        admissions=[('fn candidate()->I64 ensures {result<0} {14}',True),('fn candidate()->I64 ensures {result+1} {9}',False),('fn observe()->I64 actions {pilot.audit} {3} fn candidate()->I64 actions {pilot.audit} ensures {true || observe()>0} {1}',False),('fn candidate()->I64 requires {false} {if false {1/0} else {2}}',True)]
        background='No extra input values are required.'
    shared='module compact;\n'+head
    source=shared+'\n'.join('fn p'+str(i)+'()->I64 {'+body+'}' for i,body in enumerate(bodies))+'\n'
    return dict(source=source,head=shared,expected=values,admissions=[dict(source=shared+text,accepted=value) for text,value in admissions],background=background)


def correct(topic):
    case=corpus(topic);return dict(predictions=case['expected'],admissions=[a['accepted'] for a in case['admissions']])


def prompt(row):
    topic,arm=row['family'],row['arm'];case=corpus(topic)
    text=('Source-only Lobo comprehension. No tools, execution, editing or delegation.\n'+(LEGACY if arm=='legacy' else CLEAN)+'\nThe host supplies correct stable identity metadata. '+L.contract.documentation('reference',topic)+'\nPredict I64 results of p0..p5 in order as six JSON integers. Classify four admission fragments as static-check acceptance booleans, not runtime success or contract proof. Keep shared types/helpers, remove p0..p5, and append each fragment independently. No fragment sees another. '+case['background']+'\nPROGRAM:\n'+case['source']+'\nADMISSION FRAGMENTS:\n'+'\n'.join('CASE '+str(i)+'\n'+a['source'][len(case['head']):] for i,a in enumerate(case['admissions']))+'\nReturn only the requested JSON object.\n')
    if arm=='targeted':text+='\n'+card(topic)
    elif arm not in ('legacy','clean'):raise ValueError(arm)
    return text


def grade(topic,answer):
    errors=L.contract.errors('reference',topic,answer);answer=answer if isinstance(answer,dict) else {};expected=correct(topic);checks={}
    for field in ('predictions','admissions'):
        value=answer.get(field)
        for i,want in enumerate(expected[field]):checks[field+'_'+str(i)]=isinstance(value,list) and len(value)==len(expected[field]) and L.C.BASE.equal(value[i],want)
    return dict(accepted=not errors and all(checks.values()),format_valid=not errors,format_errors=errors,passed=sum(checks.values()),total=10,checks=checks)


def schedule():
    rows=[]
    for effort_index,effort in enumerate(('high','low')):
        topics=list(TOPICS);random.Random(20261125).shuffle(topics)
        for i,topic in enumerate(topics):
            shift=(i+effort_index)%3
            for arm in ARMS[shift:]+ARMS[:shift]:rows.append(dict(index=len(rows),stage='compact',reasoning=effort,family=topic,language='lobo',arm=arm,phase='comprehension'))
    return rows


def validate(out):
    out.mkdir(parents=True,exist_ok=False);rows=[]
    for topic in TOPICS:
        case=corpus(topic);folder=out/topic;folder.mkdir();source=folder/'program.lobo';source.write_text(case['source']);prefix='compact.'+topic;Path(str(source)+'.ids.json').write_text(L.corpus.identities(case['source'],prefix));cmd=[str(L.C.BASE.COMPILER),'compile',str(source),'-o',str(folder/'program')];p=subprocess.run(cmd,capture_output=True,text=True,timeout=35)
        record=dict(topic=topic,build=dict(command=cmd,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr),values=[],admissions=[]);rows.append(record);L.C.BASE.write_json(out/'results.json',rows)
        if p.returncode:raise ValueError(record['build'])
        for i,want in enumerate(case['expected']):
            for route in ('native','eval'):
                cmd=([str(folder/'program')] if route=='native' else [str(L.C.BASE.COMPILER),'eval',str(source)])+['--entry',prefix+'.p'+str(i)];p=subprocess.run(cmd,capture_output=True,text=True,timeout=10);actual=json.loads(p.stdout) if p.stdout else {};ok=p.returncode==0 and actual.get('value')==dict(type='I64',value=str(want));record['values'].append(dict(route=route,entry=i,expected=want,command=cmd,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr,ok=ok))
                if not ok:L.C.BASE.write_json(out/'results.json',rows);raise ValueError(record['values'][-1])
        for i,a in enumerate(case['admissions']):
            path=folder/('admission-'+str(i)+'.lobo');path.write_text(a['source']);Path(str(path)+'.ids.json').write_text(L.corpus.identities(a['source'],prefix+'.case'+str(i)));cmd=[str(L.C.BASE.COMPILER),'check',str(path)];p=subprocess.run(cmd,capture_output=True,text=True,timeout=10);record['admissions'].append(dict(command=cmd,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr,expected=a['accepted'],ok=(p.returncode==0)==a['accepted']))
            if not record['admissions'][-1]['ok']:L.C.BASE.write_json(out/'results.json',rows);raise ValueError(record['admissions'][-1])
        # A compiled source mutation changes p0's normal result, rather than relying on a build failure.
        before,after={'ranges':('Bytes::length(Text::bytes("KñΩS"))+','1+Bytes::length(Text::bytes("KñΩS"))+'),'nominal':('b:17','b:18'),'propagation':('Ok(value+6)','Ok(value+7)'),'contracts':('{value+5}','{value+6}') }[topic]
        if before not in case['source']:raise ValueError('missing mutation target')
        changed=case['source'].replace(before,after,1);path=folder/'mutant.lobo';path.write_text(changed);Path(str(path)+'.ids.json').write_text(L.corpus.identities(changed,prefix+'.mutant'));cmd=[str(L.C.BASE.COMPILER),'compile',str(path),'-o',str(folder/'mutant')];p=subprocess.run(cmd,capture_output=True,text=True,timeout=35);record['mutation']=dict(build=dict(command=cmd,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr))
        if p.returncode:L.C.BASE.write_json(out/'results.json',rows);raise ValueError(record['mutation'])
        cmd=[str(folder/'mutant'),'--entry',prefix+'.mutant.p0'];p=subprocess.run(cmd,capture_output=True,text=True,timeout=10);actual=json.loads(p.stdout) if p.stdout else {};record['mutation'].update(command=cmd,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr,killed=p.returncode==0 and actual.get('value',{}).get('value')!=str(case['expected'][0]));L.C.BASE.write_json(out/'results.json',rows)
        if not record['mutation']['killed']:raise ValueError('surviving mutation')
    probes=[]
    for name,signature in PACKET['buffer_signatures'].items():
        for valid in (True,False):
            params=signature['parameters'];declarations=[f'arg{i}:{t}' for i,t in enumerate(params)];arguments=[f'arg{i}' for i in range(len(params))]
            if not valid:
                if params:declarations[0]='arg0:Bool'
                else:arguments=['true']
            source='module probe; fn candidate('+','.join(declarations)+')->'+signature['result']+' {'+name+'('+','.join(arguments)+')}'
            folder=out/'signature-probes';folder.mkdir(exist_ok=True);path=folder/(name.replace('::','-')+'-'+str(valid)+'.lobo');path.write_text(source);Path(str(path)+'.ids.json').write_text(L.corpus.identities(source,'compact.probe'));cmd=[str(L.C.BASE.COMPILER),'check',str(path)];p=subprocess.run(cmd,capture_output=True,text=True,timeout=10);entry=dict(operation=name,valid_signature=valid,command=cmd,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr,ok=(p.returncode==0)==valid);probes.append(entry);L.C.BASE.write_json(out/'signature-probes.json',probes)
            if not entry['ok']:raise ValueError(entry)
    return rows


def register(out,validation):
    records=json.loads(validation.read_text());probes=json.loads((validation.parent/'signature-probes.json').read_text())
    if len(records)!=4 or not all(r['build']['exit_code']==0 and len(r['values'])==12 and all(v['ok'] for v in r['values']) and len(r['admissions'])==4 and all(v['ok'] for v in r['admissions']) and r['mutation']['killed'] for r in records) or len(probes)!=28 or not all(p['ok'] for p in probes):raise ValueError('validation failed')
    for c in PACKET['cards'].values():
        for ref in c['sources']:
            if sha(ROOT/ref['path'])!=ref['sha256']:raise ValueError('source drift')
    old=json.loads((ROOT/'benchmarks/results/2026-11-24-learning-verification.json').read_text())
    if sha(L.C.BASE.COMPILER)!=json.loads((L.PRIVATE/'registration/manifest.json').read_text())['compiler_sha256'] or old['checks']['cargo']['failed']:raise ValueError('frozen compiler receipt mismatch')
    out.mkdir(parents=True,exist_ok=False);prompts={}
    for row in schedule():
        text=prompt(row);(out/('prompt-%02d.txt'%row['index'])).write_text(text);prompts[str(row['index'])]=dict(sha256=L.C.hash_text(text),bytes=len(text.encode()))
    own=[HERE/'compact.py',HERE/'test_compact.py',HERE/'compact_report.py',HERE/'test_compact_report.py',PACKET_PATH]
    prior=json.loads((L.PRIVATE/'registration/manifest.json').read_text())['code']
    if not all(sha(ROOT/p)==v for p,v in prior.items()):raise ValueError('prior frozen code changed')
    code={p:sha(ROOT/p) for p in prior}|{str(p.relative_to(ROOT)):sha(p) for p in own}
    record=dict(format='lobo-compact-pilot/1',date='2026-11-24',model=L.C.LUNA.MODEL,scheduled=24,schedule=schedule(),code=code,prompts=prompts,compiler_sha256=sha(L.C.BASE.COMPILER),validation_sha256=sha(validation),signature_probes_sha256=sha(validation.parent/'signature-probes.json'),guide_bytes=dict(legacy=len(LEGACY.encode()),clean=len(CLEAN.encode())),card_bytes={k:len(card(k).encode()) for k in TOPICS},limits=dict(wall_s=120,tools=0,hard_token_cap=None),contrasts=[['legacy','clean'],['clean','targeted'],['legacy','targeted']],analysis='Report all three contrasts: whole-task acceptance, total task items, predictions, static admissions, shape validity and paired reported token/wall ratios. All failures and unknowns retained. Descriptive gain requires more correct items without lower observed task acceptance; mixed outcomes are not a winner. No noninferiority/general-language claim.',failure_policy='No retries/replacements. Two excluded source-free availability checks. Stop each cohort after3 consecutive infrastructure failures, preserving unattempted rows. No fallback model.',regression_evidence='Reuse the hash-matched frozen compiler and its unchanged312-pass receipt from decision0067. New code/cards/corpus receive focused tests and native/evaluator/typing/mutation checks; full product suite is not rerun for this research-only change.',environment=L.C.LUNA.environment(),limitations=['Small development-targeted pilot: four topics selected after earlier errors; new examples but not independent blinded holdouts','One program instance/topic, high-before-low order, aliases and provider caches limit generalization','Cleanup removes authoring instructions and adds a reading-mode cue while preserving semantic paragraphs; it is not a pure token-count ablation','Targeted facts contrast is conditional on the cleaned guide; no facts-only arm or interaction estimate','Cards cover this pilot, not the complete language; signature provenance kept outside the delivered text','Source-only presentation, no tool-assisted verification or interactive graph','Parent-agent development tokens, underlying provider requests and billing unknown'])
    L.C.BASE.write_json(out/'manifest.json',record);return record


def run(out,registration):
    manifest=json.loads(registration.read_text())
    if not all(sha(ROOT/p)==v for p,v in manifest['code'].items()) or sha(L.C.BASE.COMPILER)!=manifest['compiler_sha256']:raise ValueError('registered drift')
    if not all(L.C.hash_text(prompt(r))==manifest['prompts'][str(r['index'])]['sha256'] for r in manifest['schedule']):raise ValueError('prompt drift')
    out.mkdir(parents=True,exist_ok=False);L.C.BASE.write_json(out/'environment-before.json',L.C.LUNA.environment());results=[]
    # Only hooks in this isolated in-memory module change; the retained runner's file bytes stay frozen.
    L.prompt=prompt;L.grade=lambda row,answer:grade(row['family'],answer)
    for effort in ('high','low'):
        available=L.C.LUNA.canary(L.C.LUNA.configure(effort),effort,out/('availability-'+effort));print('CANARY',effort,available['available'],flush=True);plan=[r for r in manifest['schedule'] if r['reasoning']==effort];streak=0
        if not available['available']:L.C.BASE.write_json(out/('unattempted-'+effort+'.json'),dict(reason='unavailable',assignments=plan));continue
        for i,row in enumerate(plan):
            print('START',row,flush=True);result=L.trial(row,out/('trial-%02d'%row['index']));results.append(result);L.C.BASE.write_json(out/'results.json',results);print('END',row['index'],result['status'],result['accepted'],result['grading']['passed'],result['usage'],flush=True);streak=streak+1 if result['status'] in ('agent_error','infrastructure_error') else 0
            if streak>=3:L.C.BASE.write_json(out/('unattempted-'+effort+'.json'),dict(reason='three consecutive infrastructure failures',assignments=plan[i+1:]));break
    L.C.BASE.write_json(out/'environment-after.json',L.C.LUNA.environment());print('FINISHED',len(results),flush=True)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('operation',choices=['validate','register','run']);p.add_argument('--out',required=True,type=Path);p.add_argument('--validation',type=Path,default=PRIVATE/'validation/results.json');p.add_argument('--registration',type=Path,default=PRIVATE/'registration/manifest.json');a=p.parse_args()
    if a.operation=='validate':validate(a.out.resolve())
    elif a.operation=='register':register(a.out.resolve(),a.validation.resolve())
    else:run(a.out.resolve(),a.registration.resolve())
