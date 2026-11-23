#!/usr/bin/env python3
"""Check finite native models, deliberate model/C defects, and concrete witness replays."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
MODEL = ROOT / 'verification/native'
JAR_SHA256 = '936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88'

def require(condition, detail):
    if not condition:
        raise AssertionError(detail)

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def run(command, directory, stem, timeout=60):
    started = time.monotonic()
    try:
        result = subprocess.run([str(x) for x in command], cwd=directory, capture_output=True,
                                text=True, timeout=timeout, env={**os.environ,
                                'ASAN_OPTIONS':'detect_leaks=1:halt_on_error=1:symbolize=0',
                                'UBSAN_OPTIONS':'halt_on_error=1'})
        record = {'command':[str(x) for x in command], 'cwd':str(directory), 'exit_code':result.returncode,
                  'seconds':time.monotonic()-started, 'stdout':result.stdout, 'stderr':result.stderr}
    except subprocess.TimeoutExpired as error:
        record = {'command':[str(x) for x in command], 'cwd':str(directory), 'exit_code':None,
                  'seconds':time.monotonic()-started, 'stdout':str(error.stdout or ''),
                  'stderr':str(error.stderr or ''), 'status':'inconclusive-timeout'}
    (directory / (stem+'.json')).write_text(json.dumps(record, indent=2)+'\n')
    (directory / (stem+'.log')).write_text(record['stdout']+record['stderr'])
    return record

def trace_of(text):
    states=[]
    for match in re.finditer(r'^State (\d+): <([^\n]+)>\n(.*?)(?=^State \d+:|^The coverage statistics|^Progress\(|^\d+ states generated|\Z)',text,re.M|re.S):
        states.append({'number':int(match[1]),'action':match[2].split(' line ')[0],
                       'state_text':match[3].strip()})
    return states

def success_c(record, case):
    if case == 'root':
        return record['exit_code']==1 and not record['stderr'] and '"code":"invocation.native.authority"' in record['stdout']
    if case == 'untracked-fault':
        return record['exit_code']==1 and not record['stderr'] and '"code":"resource.fuel.exhausted"' in record['stdout'] and 'native-cleanup-on-fault-passed' in record['stdout']
    expected='native-adapter-case-passed\n' if case in ['publish','deadline'] else 'formal-trace-passed\n'
    return record['exit_code']==0 and not record['stderr'] and record['stdout']==expected

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tla2tools',type=Path,default=os.environ.get('TLA2TOOLS'))
    parser.add_argument('--java',default=os.environ.get('JAVA','java'))
    parser.add_argument('--out',type=Path,required=True,help='new directory for complete evidence')
    parser.add_argument('--models-only',action='store_true')
    args=parser.parse_args()
    if not args.tla2tools or not args.tla2tools.is_file():parser.error('Supply installed TLC 1.7.4 with --tla2tools or TLA2TOOLS; no automatic download')
    if digest(args.tla2tools)!=JAR_SHA256:parser.error('TLC jar SHA-256 differs from pinned 1.7.4')
    out=args.out.resolve();out.mkdir(parents=True,exist_ok=False)
    summary={'format':'lobo-native-model-check/0.1','status':'running','tlc_jar_sha256':JAR_SHA256,
             'model_results':[],'c_replays':[],'c_mutations':[],'source_hashes':{},
             'mode':'models-only' if args.models_only else 'full'}
    for p in [ROOT/'src/native_api.c',ROOT/'src/native_runtime.c',ROOT/'tests/native/formal_trace_harness.c',ROOT/'tests/native/adapter_harness.c',*MODEL.glob('*.tla')]:
        summary['source_hashes'][str(p.relative_to(ROOT))]=digest(p)
    coverage=json.loads((MODEL/'coverage.json').read_text())
    for name,expected in coverage['files'].items():
        if digest(ROOT/name)!=expected['sha256']:
            parser.error('Coverage is stale for '+name+'; review mapping with the lobo-native-verification skill before updating fingerprints')
    summary['coverage_head']=coverage['reviewed_head']
    summary['coverage_note']=coverage['working_tree_note']
    cases=json.loads((MODEL/'mutations.json').read_text())['cases']
    try:
        if not args.models_only:
            self_test=run([sys.executable,ROOT/'scripts/test-native-model-checker.py','--tla2tools',args.tla2tools.resolve(),
                           '--java',args.java,'--out',out/'checker-self-test'],out,'checker-self-test',60)
            require(self_test['exit_code']==0, ('checker self-test failed',self_test))
            summary['checker_self_test']=json.loads(self_test['stdout'])
        for name,config,invariant in [('authority',MODEL/'configs/authority.cfg',None),('operations',MODEL/'configs/operations.cfg',None)]+[(c['model'].lower()+'-'+c['name'],MODEL/c['config'],c['invariant']) for c in cases]:
            folder=out/name;folder.mkdir();module='Authority' if name.startswith('authority') else 'Operations'
            record=run([args.java,'-XX:+UseParallelGC','-Xmx512m','-cp',args.tla2tools.resolve(),'tlc2.TLC',
                        '-workers','1','-seed','1','-fp','0','-coverage','1','-config',config,'-metadir',folder/'states',MODEL/(module+'.tla')],folder,'tlc')
            text=record['stdout'];violated=re.search(r'Invariant (\w+) is violated',text)
            if invariant:
                require(record['exit_code']==12 and violated and violated[1]==invariant, (name,record['exit_code'],text[-3000:]))
                trace=trace_of(text);require(len(trace)>=2, (name,'no counterexample states'))
                (folder/'counterexample.json').write_text(json.dumps({'invariant':invariant,'states':trace},indent=2)+'\n')
            else:
                require(record['exit_code']==0 and 'Model checking completed. No error has been found.' in text, (name,text[-3000:]))
                require(re.search(r'0 states left on queue',text), (name,'incomplete exploration'))
            action_counts={m[1]:{'new_states':int(m[2]),'successors':int(m[3])}
                           for m in re.finditer(r'^<(\w+) line [^\n]+>: (\d+):(\d+)$',text,re.M)}
            if not invariant:
                expected_actions=coverage['models'][module]['actions']
                require(all(action_counts.get(action,{}).get('successors',0)>0 for action in expected_actions), (name,'vacuous action',action_counts))
            counts=re.search(r'([\d,]+) states generated, ([\d,]+) distinct states found, ([\d,]+) states left on queue',text)
            require(counts, (name,'missing statistics'))
            summary['model_results'].append({'name':name,'status':'expected-counterexample' if invariant else 'pass',
                'expected_invariant':invariant,'generated':int(counts[1].replace(',','')),'distinct':int(counts[2].replace(',','')),
                'queued':int(counts[3].replace(',','')),'trace_states':len(trace) if invariant else None,'action_counts':action_counts})
            print(name,summary['model_results'][-1]['status'],counts[2],flush=True)
        if not args.models_only:
            for case in cases:
                folder=out/('c-'+case['name']);folder.mkdir()
                for variant in ['current','mutant']:
                    source=folder/variant;shutil.copytree(ROOT/'src',source)
                    if variant=='mutant':
                        p=source/'native_api.c';text=p.read_text();mutation=case['c_mutation']
                        if 'start' in mutation:
                            start=text.index(mutation['start']);end=text.index(mutation['end'],start);text=text[:start]+text[end:]
                        else:
                            count=text.count(mutation['before']);require(count==1 or count>1 and mutation['first_only'], (case['name'],count))
                            text=text.replace(mutation['before'],mutation['after'],1)
                        p.write_text(text)
                    for mode,flags in [('optimized',['-O2']),('sanitized',['-O1','-g','-fsanitize=address,undefined','-fno-sanitize-recover=all'])]:
                        binary=folder/(variant+'-'+mode)
                        built=run(['clang','-std=c11',*flags,'-I',source,ROOT/'tests/native/formal_trace_harness.c','-o',binary],folder,variant+'-'+mode+'-build')
                        require(built['exit_code']==0, (case['name'],built['stderr']))
                        observation=run([binary,case['c_case']],folder,variant+'-'+mode+'-run',15)
                        accepted=success_c(observation,case['c_case'])
                        require(accepted == (variant=='current'), (case['name'],variant,mode,observation))
                        if variant=='mutant':require(observation['exit_code'] in [1,2,3], ('unexpected mutant failure class',observation))
                        row={'case':case['name'],'mode':mode,'status':'pass' if accepted else 'expected-rejection','exit_code':observation['exit_code'],'model_invariant':case['invariant']}
                        summary['c_replays' if variant=='current' else 'c_mutations'].append(row)
                print('C',case['name'],'replayed; mutation rejected on both builds',flush=True)
        summary['status']='pass'
    except (AssertionError,OSError) as error:
        summary['status']='fail';summary['failure']=str(error)
    finally:
        (out/'results.json').write_text(json.dumps(summary,indent=2)+'\n')
    if summary['status']!='pass':print(summary.get('failure'),file=sys.stderr);return 1
    print(json.dumps({'status':'pass','models':len(summary['model_results']),'c_replays':len(summary['c_replays']),'c_mutations':len(summary['c_mutations'])}));return 0

if __name__=='__main__':sys.exit(main())
