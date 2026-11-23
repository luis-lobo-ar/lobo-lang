#!/usr/bin/env python3
"""Check the bounded SQLite lifecycle model and replay matching deliberate C defects."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

ROOT=Path(__file__).resolve().parents[1]
JAR_SHA='936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88'

def require(condition,message):
    if not condition:
        raise RuntimeError(message)

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def verify_map(path):
    coverage=json.loads(path.read_text())
    require(coverage.get('format')=='lobo-sqlite-formal-coverage/0.1','unsupported SQLite coverage format')
    for name,record in coverage['files'].items():
        require(digest(ROOT/name)==record['sha256'],f'stale reviewed SQLite input: {name}; review the source mapping before updating fingerprints')
    return coverage

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out',type=Path)
    parser.add_argument('--verify-map',action='store_true',help='read-only reviewed source correspondence gate')
    parser.add_argument('--coverage',type=Path,default=ROOT/'verification/sqlite/coverage.json')
    parser.add_argument('--tla2tools',type=Path,default=os.environ.get('TLA2TOOLS'))
    parser.add_argument('--models-only',action='store_true',help='diagnostic only, not the full acceptance gate')
    args=parser.parse_args()
    coverage=verify_map(args.coverage)
    if args.verify_map:
        print(json.dumps({'status':'pass','reviewed_files':len(coverage['files'])}));return
    require(args.out is not None,'--out is required for model checking')
    require(args.tla2tools and args.tla2tools.is_file(),'an explicitly installed TLA2TOOLS jar is required')
    require(digest(args.tla2tools)==JAR_SHA,'TLA2TOOLS identity mismatch')
    args.out.mkdir(parents=True,exist_ok=False)
    output=args.out.resolve()
    spec=importlib.util.spec_from_file_location('native_models',ROOT/'scripts/check-native-models.py')
    shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
    summary={'status':'running','scope':'finite SQLite adapter lifecycle; no engine/filesystem or full refinement proof','bounds':{'operation_budget':8,'pending_plus_committed_writes':2,'connections':1,'statements':1,'permission_atoms':3},'models':[],'c':[],
             'inputs':{str(p.relative_to(ROOT)):digest(p) for p in [ROOT/'src/native_sqlite.c',ROOT/'src/sqlite_profile.h',ROOT/'vendor/sqlite/sqlite3.c',ROOT/'tests/native/sqlite_harness.c',ROOT/'verification/sqlite/Lifecycle.tla']}}
    def save():
        (output/'results.json').write_text(json.dumps(summary,indent=2)+'\n')
    cases={
        'none':None,'cleanup':'Ownership','persist':'OnlyCommitPersists','commit':'LegalCommit','amplify':'NoAmplification'
    }
    try:
        for mutation,invariant in cases.items():
            folder=output/('model-'+mutation);folder.mkdir()
            config=ROOT/f'verification/sqlite/configs/{mutation}.cfg'
            result=shared.run(['java','-XX:+UseParallelGC','-Xmx512m','-cp',args.tla2tools.resolve(),'tlc2.TLC','-workers','1','-coverage','1','-metadir',folder/'states','-config',config,ROOT/'verification/sqlite/Lifecycle.tla'],folder,'tlc',60)
            text=result['stdout']+result['stderr']
            if invariant:
                require(result['exit_code']==12 and f'Invariant {invariant} is violated' in text,(mutation,text[-2000:]))
                trace=shared.trace_of(text);require(len(trace)>1,(mutation,'missing counterexample'))
                (folder/'counterexample.json').write_text(json.dumps({'invariant':invariant,'states':trace},indent=2)+'\n')
            else:
                require(result['exit_code']==0 and 'Model checking completed. No error has been found.' in text and re.search(r'0 states left on queue',text),('incomplete baseline',text[-2000:]))
                actions={m[1]:int(m[3]) for m in re.finditer(r'^<(\w+) line [^\n]+>: (\d+):(\d+)$',text,re.M)}
                for action in ['Open','Prepare','ReadRow','DoneStatement','Finalize','Begin','Write','Commit','Rollback','Derive','Exit']:
                    require(actions.get(action,0)>0,('vacuous action',action,actions))
                summary['action_successors']=actions
            summary['models'].append({'mutation':mutation,'status':'counterexample' if invariant else 'pass','invariant':invariant,'command_exit_code':result['exit_code']});save()
        if not args.models_only:
            # One engine object per instrumentation mode. Each adapter mutation still compiles separately.
            # The header and amalgamation share the pinned upstream interface; the engine is not mutated.
            mutations={
                'cleanup':('for(unsigned i=0;i<32;++i) if(lobo_sqlite_slots[i].kind==LS_STMT) lobo_sqlite_finalize_owned(&lobo_sqlite_slots[i]);','', 'fuel-cleanup','lobo_sqlite_slots[i].kind!=LS_DB'),
                'persist':('if(!sqlite3_get_autocommit(db)) (void)sqlite3_exec(db,"ROLLBACK",NULL,NULL,NULL);','if(!sqlite3_get_autocommit(db)) (void)sqlite3_exec(db,"COMMIT",NULL,NULL,NULL);','fuel-cleanup','oracle(observer_file)==0'),
                'commit':('if((action==0)==active) return lobo_sqlite_error(LS_STATE);','','transactions','category(lobo_sqlite_commit_v1(db))==LS_STATE'),
                'amplify':('if(((unsigned)mask&slot->rights)!=(unsigned)mask) return lobo_sqlite_error(LS_DENIED);','','policy','category(lobo_sqlite_restrict_v1(readcap,7))==LS_DENIED')}
            original=(ROOT/'src/native_sqlite.c').read_text()
            for mode,flags in [('optimized',['-O2']),('sanitized',['-O1','-fsanitize=address,undefined','-fno-sanitize-recover=all'])]:
                engine=output/(mode+'-engine.o')
                build=shared.run(['clang','-std=c11','-g0',*flags,'-include',ROOT/'src/sqlite_profile.h','-c',ROOT/'vendor/sqlite/sqlite3.c','-o',engine],output,mode+'-engine-build',180)
                require(build['exit_code']==0,build['stderr'])
                for mutation,(before,after,witness,marker) in mutations.items():
                    folder=output/(mode+'-'+mutation);folder.mkdir()
                    require(original.count(before)==1 or mutation=='persist' and original.count(before)==2,('mutation anchor changed',mutation))
                    for variant in ['baseline','mutant']:
                        provider=folder/(variant+'.c');provider.write_text(original if variant=='baseline' else original.replace(before,after,1))
                        binary=folder/variant
                        build=shared.run(['clang','-std=c11','-g0',*flags,'-DLOBO_SQLITE_EXTERNAL',f'-DLOBO_SQLITE_PROVIDER="{provider}"',ROOT/'tests/native/sqlite_harness.c',engine,'-o',binary],folder,variant+'-build',60)
                        require(build['exit_code']==0,('mutation build is not a kill',mutation,build['stderr']))
                        observation=shared.run([binary,witness,folder/(variant+'.sqlite')],folder,variant+'-run',15)
                        if variant=='baseline':
                            require(observation['exit_code']==(1 if witness=='fuel-cleanup' else 0) and not observation['stderr'],('baseline witness failed',mutation,observation))
                            require(('sqlite-fault-cleanup-passed' if witness=='fuel-cleanup' else 'sqlite-adapter-case-passed') in observation['stdout'],('missing baseline acceptance marker',mutation))
                        else:
                            require(observation['exit_code'] in [1,23] and 'sqlite check failed' in observation['stderr'] and marker in observation['stderr'],('mutant failed for the wrong reason or survived',mutation,observation))
                    summary['c'].append({'mode':mode,'mutation':mutation,'witness':witness,'status':'baseline replayed; isolated mutant rejected by its expected check'});save()
        summary['status']='diagnostic-models-only' if args.models_only else 'pass';save()
        print(json.dumps({'status':summary['status'],'models':len(summary['models']),'c_mutations':len(summary['c'])}))
    except BaseException as error:
        summary.update(status='fail',error=str(error));save();raise

if __name__=='__main__':
    main()
