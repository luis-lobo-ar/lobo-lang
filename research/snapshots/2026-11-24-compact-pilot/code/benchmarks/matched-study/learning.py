#!/usr/bin/env python3
"""Registered, separate response-format, reference-presence and graph-presentation experiments."""
import argparse
import hashlib
import importlib.util
import itertools
import json
import os
from pathlib import Path
import random
import signal
import subprocess
import tempfile
import time

HERE=Path(__file__).parent.resolve();ROOT=HERE.parents[1]
PRIVATE=ROOT/'benchmarks/private-evidence/learning-study-20261124'


def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module


corpus=load('learning_corpus',HERE/'learning_corpus.py');C=corpus.C
contract=load('response_contract',HERE/'response_contract.py')
REF=load('language_reference',ROOT/'research/language_reference.py')
STAGES=('format','reference','graph')
ARMS={'format':('ambiguous','explicit'),'reference':('base','flat'),'graph':('flat','graph')}


def reference(style):
    data=REF.load();examples=(REF.DIRECTORY/'reading-examples.lobo').read_text()
    if style=='graph':
        adjacency={n['id']:[] for n in data['nodes']}
        for e in data['edges']:adjacency[e['source']].append(dict(relation=e['relation'],target=e['target']))
        text=json.dumps(dict(language_version=data['language_version'],compiler_version=data['compiler_version'],coverage=data['coverage'],concepts=data['nodes'],adjacency=adjacency),ensure_ascii=False,indent=2)
    elif style=='flat':
        lines=['LANGUAGE REFERENCE '+data['language_version']+' compiler '+data['compiler_version'],data['coverage']]
        for n in data['nodes']:
            lines += ['CONCEPT '+n['id'],n['title'],'Category: '+n['category'],'Status: '+n['status'],'Rule: '+n['semantics']]
            lines += ['Pitfall: '+p for p in n['pitfalls']]
            lines += ['Source: '+json.dumps(r,ensure_ascii=False,sort_keys=True) for r in n['source_refs']]
        lines += ['Relationship: '+e['source']+' | '+e['relation']+' | '+e['target'] for e in data['edges']]
        text='\n'.join(lines)
    else:raise ValueError(style)
    return text+'\nCHECKED EXAMPLES\n'+examples


def reference_facts_hash(style):
    if style not in ('flat','graph'):raise ValueError(style)
    data=REF.load();facts={k:data[k] for k in ('language_version','compiler_version','coverage','nodes','edges')}
    return C.hash_text(json.dumps(facts,sort_keys=True)+(REF.DIRECTORY/'reading-examples.lobo').read_text())


def format_question(family):
    change={'ledger':'permitted changes its inclusive lower amount bound from0 to1','scheduler':'affordable changes cost<=remaining to cost+3<=remaining','frames':'length_ok additionally rejects length==3'}[family]
    return ('Task family: '+family+'. Source-only comprehension. Return one JSON object: predictions (SIX complete normalized Summary objects for the inputs below); witness (any valid input yielding '+json.dumps(corpus.format_target(family))+'; unspecified output fields can vary); direct_calls (application functions directly called by run, ignoring constructors/builtins/standard-library methods and util.reverse); impacted (functions in logic whose output can change for identical direct arguments if '+change+'; include the edited predicate itself); fault_location (the predicate responsible for that policy); counterexample (any valid input distinguishing original from that mutant). Use plain function names. Output byte values as lowercase hex. Optional top-level notes is a string. No tools. Return JSON only.\nInputs: '+json.dumps(corpus.format_inputs(family)))


def prompt(row):
    stage,family,language,arm=[row[k] for k in ('stage','family','language','arm')]
    if stage=='format':
        requirement=C.BASE.BASE_REQUIREMENTS[family]
        if family=='ledger':requirement=requirement.replace('0<=amount<=1000000','0<=amount<=83')
        elif family=='frames':requirement=requirement.replace('sum(payload)%256','(13+sum(payload))%256').replace('needs checksum0','needs checksum13')
        files=corpus.format_sources(family,language)
        text='Use no tools.\n'+C.BASE.guide(language)+'\n'+C.input_schema(family)+'\n'+requirement+'\n'+format_question(family)+'\n'+'\n'.join('FILE '+n+'\n'+v for n,v in files.items() if not n.startswith('app.') and '.ids.' not in n)
        if arm=='explicit':text+='\n'+contract.documentation(stage,family)
        elif arm!='ambiguous':raise ValueError(arm)
        return text
    form=0 if stage=='reference' else 1;case=corpus.micro(family,form)
    text=('Source-only Lobo 1.11.2 comprehension. No tools, execution or delegation.\n'+C.BASE.guide('lobo')+'\nAll required stable identity metadata is correctly supplied by the host and omitted here. Do not reject snippets for missing identity metadata.\n'+contract.documentation(stage,family)+'\nPredict the I64 result of p0,p1,p2,p3,p4,p5 in order, as six JSON integers. Classify whether each of four admission snippets passes static checking, as four JSON booleans. Admission asks TYPE/CHECK acceptance, not successful runtime execution or proof of contracts. For each admission case, keep the shared helper/type declarations in PROGRAM, remove p0..p5, and append only that snippet. Each case is checked independently; no snippet sees another. The host supplies correct metadata for the resulting complete module. Return only the requested JSON.\nPROGRAM:\n'+case['source']+'\nADMISSION SNIPPETS:\n'+'\n'.join('CASE '+str(i)+'\n'+a['source'][len(case['head']):] for i,a in enumerate(case['admissions'])))
    if arm in ('flat','graph'):text+='\n'+reference(arm)
    elif arm!='base':raise ValueError(arm)
    return text


def grade(row,answer):
    stage,family=row['stage'],row['family'];format_errors=contract.errors(stage,family,answer)
    if stage=='format':checks=corpus.format_grade(family,answer)
    else:
        a=answer if isinstance(answer,dict) else {};wanted=corpus.micro_correct(family,0 if stage=='reference' else 1);checks={}
        for key in ('predictions','admissions'):
            v=a.get(key);expected=wanted[key]
            for i,x in enumerate(expected):checks[key+'_'+str(i)]=isinstance(v,list) and len(v)==len(expected) and C.BASE.equal(v[i],x)
    return dict(accepted=not format_errors and all(checks.values()),format_valid=not format_errors,format_errors=format_errors,passed=sum(checks.values()),total=len(checks),checks=checks)


def schedule():
    rows=[]
    for stage in STAGES:
        for effort in ('high','low'):
            blocks=list(itertools.product(C.BASE.FAMILIES,C.BASE.LANGUAGES)) if stage=='format' else [(f,'lobo') for f in corpus.TOPICS]
            random.Random(20261124).shuffle(blocks)
            for i,(family,language) in enumerate(blocks):
                order=ARMS[stage] if i%2==0 else tuple(reversed(ARMS[stage]))
                for arm in order:rows.append(dict(index=len(rows),stage=stage,reasoning=effort,family=family,language=language,arm=arm,phase='comprehension'))
    return rows


def validate(out):
    out.mkdir(parents=True,exist_ok=False);records=[]
    for family,language,mutant in itertools.product(C.BASE.FAMILIES,C.BASE.LANGUAGES,(False,True)):
        files=corpus.format_sources(family,language);files['logic.'+{'lobo':'lobo','rust':'rs','python':'py'}[language]]=corpus.format_logic(family,language,mutant)
        folder=out/(family+'-'+language+('-mutant' if mutant else ''));C.BASE.save_sources(folder,files);build=C.BASE.build(folder,language,folder/'program');record=dict(kind='format',family=family,language=language,mutant=mutant,build=build,cases=[]);records.append(record);C.BASE.write_json(out/'results.json',records)
        if not build['ok']:raise ValueError(build)
        probes=corpus.format_inputs(family)+[corpus.format_correct(family)[k] for k in ('witness','counterexample')]+C.BASE.cases(family,hidden=True)
        if family=='frames':
            for n in range(10):
                payload=bytes(range(n));probes.append(dict(data=(bytes([n])+payload+bytes([(13+sum(payload))%256])).hex(),limit=n))
        for args in probes:
            actual=C.BASE.execute(folder,language,family,args,folder/'program');expected=corpus.format_oracle(family,args,mutant);record['cases'].append(dict(input=args,actual=actual,expected=expected,ok=C.BASE.equal(actual,expected)))
            if not C.BASE.equal(actual,expected):C.BASE.write_json(out/'results.json',records);raise ValueError(record['cases'][-1])
        record['killed']=not mutant or corpus.format_oracle(family,corpus.format_correct(family)['counterexample'])!=C.BASE.execute(folder,language,family,corpus.format_correct(family)['counterexample'],folder/'program')
        if not record['killed']:raise ValueError('surviving format mutant')
        C.BASE.write_json(out/'results.json',records)
    for topic,form in itertools.product(corpus.TOPICS,(0,1)):
        case=corpus.micro(topic,form);folder=out/(topic+'-'+str(form));folder.mkdir();source=folder/'program.lobo';source.write_text(case['source']);prefix='learning.'+topic+'.'+str(form);(folder/'program.lobo.ids.json').write_text(corpus.identities(case['source'],prefix));cmd=[str(C.BASE.COMPILER),'compile',str(source),'-o',str(folder/'program')]
        build=subprocess.run(cmd,capture_output=True,text=True,timeout=35);record=dict(kind='micro',topic=topic,form=form,build=dict(command=cmd,exit_code=build.returncode,stdout=build.stdout,stderr=build.stderr),values=[],admissions=[]);records.append(record);C.BASE.write_json(out/'results.json',records)
        if build.returncode:raise ValueError(record['build'])
        for i,expected in enumerate(case['expected']):
            for route in ('native','eval'):
                cmd=([str(folder/'program')] if route=='native' else [str(C.BASE.COMPILER),'eval',str(source)])+['--entry',prefix+'.p'+str(i)]
                p=subprocess.run(cmd,capture_output=True,text=True,timeout=10);value=json.loads(p.stdout) if p.stdout else {};ok=p.returncode==0 and value.get('value')==dict(type='I64',value=str(expected));record['values'].append(dict(route=route,entry=i,expected=expected,command=cmd,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr,ok=ok))
                if not ok:C.BASE.write_json(out/'results.json',records);raise ValueError(record['values'][-1])
        for i,admission in enumerate(case['admissions']):
            path=folder/('admission-'+str(i)+'.lobo');path.write_text(admission['source']);Path(str(path)+'.ids.json').write_text(corpus.identities(admission['source'],prefix+'.admission'+str(i)));cmd=[str(C.BASE.COMPILER),'check',str(path)];p=subprocess.run(cmd,capture_output=True,text=True,timeout=10);ok=(p.returncode==0)==admission['accepted'];record['admissions'].append(dict(command=cmd,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr,expected_accept=admission['accepted'],ok=ok))
            if not ok:C.BASE.write_json(out/'results.json',records);raise ValueError(record['admissions'][-1])
        mutated,entry=corpus.micro_mutant(topic,form);path=folder/'mutant.lobo';path.write_text(mutated);Path(str(path)+'.ids.json').write_text(corpus.identities(mutated,prefix+'.mutant'));cmd=[str(C.BASE.COMPILER),'compile',str(path),'-o',str(folder/'mutant')];p=subprocess.run(cmd,capture_output=True,text=True,timeout=35);record['mutation']=dict(build=dict(command=cmd,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr))
        if p.returncode:C.BASE.write_json(out/'results.json',records);raise ValueError(record['mutation'])
        cmd=[str(folder/'mutant'),'--entry',prefix+'.mutant.p'+str(entry)];p=subprocess.run(cmd,capture_output=True,text=True,timeout=10);value=json.loads(p.stdout) if p.stdout else {};killed=p.returncode==0 and value.get('value',{}).get('value')!=str(case['expected'][entry]);record['mutation'].update(command=cmd,exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr,killed_by_value=killed)
        if not killed:C.BASE.write_json(out/'results.json',records);raise ValueError('surviving micro mutation')
        C.BASE.write_json(out/'results.json',records)
    return records


def register(out,validation):
    records=json.loads(validation.read_text())
    if len(records)!=30 or not all((r['build'].get('ok',True) and all(c['ok'] for c in r['cases']) and r['killed']) if r['kind']=='format' else (r['build']['exit_code']==0 and len(r['values'])==12 and all(v['ok'] for v in r['values']) and len(r['admissions'])==4 and all(a['ok'] for a in r['admissions']) and r['mutation']['killed_by_value']) for r in records):raise ValueError('incomplete or failed validation')
    if (PRIVATE/'development/cargo-exit.txt').read_text().strip()!='0':raise ValueError('compiler regression gate failed')
    REF.validate(REF.load());freeze=json.loads((C.BASE.PRIVATE/'freeze/manifest.json').read_text())
    if not all(C.sha(C.BASE.PRIVATE/'freeze'/name)==v['sha256'] for name,v in freeze['files'].items()):raise ValueError('freeze drift')
    out.mkdir(parents=True,exist_ok=False);prompts={}
    for row in schedule():
        text=prompt(row);path=out/('prompt-%02d.txt'%row['index']);path.write_text(text);prompts[str(row['index'])]=dict(sha256=C.hash_text(text),bytes=len(text.encode()))
    code={str(p.relative_to(ROOT)):C.sha(p) for p in [HERE/n for n in ('learning.py','learning_corpus.py','response_contract.py','test_learning.py','learning_report.py','test_learning_report.py','context_report.py','context.py','luna.py','study.py')]+[ROOT/'research/language_reference.py',REF.DIRECTORY/'concepts.json',REF.DIRECTORY/'reading-examples.lobo']}
    manifest=dict(format='lobo-learning-study/1',date='2026-11-24',model=C.LUNA.MODEL,stages=list(STAGES),schedule=schedule(),prompts=prompts,code=code,validation=dict(path=str(validation),sha256=C.sha(validation)),compiler_sha256=C.sha(C.BASE.COMPILER),frozen_version='1.11.2',reference_facts_sha256=reference_facts_hash('flat'),prior_paper_index_sha256=C.sha(ROOT/'research/local-evidence.json'),limits=dict(wall_s=120,tools=0,hard_token_cap=None),analysis='Analyze stages separately, format validity separately from semantic items and whole-task acceptance; paired discordances, actual usage/cache/wall, all failures and unknowns. No posthoc score replacement. Reference and graph programs/forms differ; compare only within stages.',failure_policy='No retries or replacements. One source-free availability check before each stage/effort cohort, excluded from scores and included in costs. Stop that cohort after3 consecutive infrastructure errors and explicitly retain unattempted assignments. No fallback model.',environment=C.LUNA.environment(),limitations=['Three format families and six Lobo microtask topics, one fresh form per topic/stage; descriptive pilot, not powered population inference.','Format contrasts wording/profile disclosure with a common unchanged strict grader; no native provider schema enforcement.','Reference adds facts/examples and tokens; graph contrasts presentation of identical facts/examples, not interactive graph retrieval or tool access.','High precedes low within each stage; alias/caching/service/time drift remain confounds.','Source-only predictions/static acceptance do not measure repository construction quality.','Questions authored after earlier outcomes; new instances, not independent blinded holdouts.','Parent-agent development usage and actual billing unknown.'])
    C.BASE.write_json(out/'manifest.json',manifest);return manifest


def trial(row,out):
    out.mkdir(parents=True,exist_ok=False);folder=Path(tempfile.mkdtemp(prefix='lobo-learning-'));(folder/'AGENTS.md').write_text('Source-only experiment. Follow the supplied prompt; use no tools and do not inspect other directories.\n');text=prompt(row);(out/'prompt.txt').write_text(text);args=C.command(folder,row['reasoning'],out/'last-message.txt');C.BASE.write_json(out/'command.json',args);C.BASE.write_json(out/'assignment.json',row)
    env=dict(os.environ);env.pop('CODEX_THREAD_ID',None);env.pop('CODEX_INTERNAL_ORIGINATOR_OVERRIDE',None);start=time.monotonic();result=dict(assignment=row,command=args,started_unix=time.time(),load_before=os.getloadavg(),workspace=str(folder));stdout='';stderr=''
    try:
        p=subprocess.Popen(args,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=folder,env=env,text=True,start_new_session=True)
        try:stdout,stderr=p.communicate(text,timeout=120);status='completed' if p.returncode==0 else 'agent_error'
        except subprocess.TimeoutExpired:
            status='wall_limit';os.killpg(p.pid,signal.SIGTERM)
            try:stdout,stderr=p.communicate(timeout=3)
            except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);stdout,stderr=p.communicate()
        result.update(status=status,exit_code=p.returncode)
    except Exception as exc:result.update(status='infrastructure_error',error=repr(exc))
    result.update(agent_wall_s=time.monotonic()-start,load_after=os.getloadavg());(out/'events.jsonl').write_text(stdout);(out/'stderr.txt').write_text(stderr);events=[];items={}
    for line in stdout.splitlines():
        try:events.append(json.loads(line))
        except ValueError:pass
    for event in events:
        item=event.get('item',{})
        if item.get('id'):items[item['id']]=item
    answer=contract.decode((out/'last-message.txt').read_text()) if (out/'last-message.txt').exists() else None;result.update(usage=C.BASE.usage(events),item_types=[x.get('type') for x in items.values()],grading=grade(row,answer));result['accepted']=result['status']=='completed' and result['grading']['accepted'] and all(t in ('agent_message','reasoning') for t in result['item_types']);C.BASE.write_json(out/'result.json',result);return result


def run(out,registration):
    manifest=json.loads(registration.read_text())
    if not all(C.sha(ROOT/p)==sha for p,sha in manifest['code'].items()) or C.sha(C.BASE.COMPILER)!=manifest['compiler_sha256']:raise ValueError('registered code changed')
    if not all(C.hash_text(prompt(row))==manifest['prompts'][str(row['index'])]['sha256'] for row in manifest['schedule']):raise ValueError('prompt drift')
    out.mkdir(parents=True,exist_ok=False);C.BASE.write_json(out/'environment-before.json',C.LUNA.environment());results=[]
    for stage in STAGES:
        for effort in ('high','low'):
            cohort=stage+'-'+effort;availability=C.LUNA.canary(C.LUNA.configure(effort),effort,out/('availability-'+cohort));print('CANARY',cohort,availability['available'],flush=True);plan=[r for r in manifest['schedule'] if r['stage']==stage and r['reasoning']==effort];streak=0
            if not availability['available']:C.BASE.write_json(out/('unattempted-'+cohort+'.json'),dict(reason='unavailable',assignments=plan));continue
            for i,row in enumerate(plan):
                print('START',row,flush=True);result=trial(row,out/('trial-%02d'%row['index']));results.append(result);C.BASE.write_json(out/'results.json',results);print('END',row['index'],result['status'],result['accepted'],result['grading']['format_valid'],result['grading']['passed'],result['usage'],flush=True);streak=streak+1 if result['status'] in ('agent_error','infrastructure_error') else 0
                if streak>=3:C.BASE.write_json(out/('unattempted-'+cohort+'.json'),dict(reason='three consecutive infrastructure failures',assignments=plan[i+1:]));break
    C.BASE.write_json(out/'environment-after.json',C.LUNA.environment());print('FINISHED',len(results),flush=True)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('operation',choices=['validate','register','run']);p.add_argument('--out',type=Path,required=True);p.add_argument('--validation',type=Path,default=PRIVATE/'validation/results.json');p.add_argument('--registration',type=Path,default=PRIVATE/'registration/manifest.json');a=p.parse_args()
    if a.operation=='validate':validate(a.out.resolve())
    elif a.operation=='register':register(a.out.resolve(),a.validation.resolve())
    else:run(a.out.resolve(),a.registration.resolve())
