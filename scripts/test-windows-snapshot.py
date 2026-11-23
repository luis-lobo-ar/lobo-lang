#!/usr/bin/env python3
"""Exercise the Windows snapshot PE and its provider using an explicit isolated Wine prefix."""
import argparse
import json
import os
from pathlib import Path
import subprocess
ROOT=Path(__file__).resolve().parents[1]

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain',type=Path,required=True)
    parser.add_argument('--exe',type=Path,required=True)
    parser.add_argument('--wine-prefix',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();out=args.out.resolve();out.mkdir(parents=True,exist_ok=False)
    cc=args.toolchain.resolve()/'bin/x86_64-w64-mingw32-clang';records=[]
    env={**os.environ,'WINEPREFIX':str(args.wine_prefix.resolve()),'WINEARCH':'win64','WINEDEBUG':'-all','WINEDLLOVERRIDES':'mscoree,mshtml='}
    def run(command):
        p=subprocess.run([str(x) for x in command],cwd=ROOT,env=env,input='',capture_output=True,text=True,timeout=60)
        r={'command':[str(x) for x in command],'exit_code':p.returncode,'stdout':p.stdout,'stderr':p.stderr};records.append(r)
        (out/'commands.json').write_text(json.dumps(records,indent=2)+'\n');return r
    def require(condition,detail):
        if not condition:raise RuntimeError(detail)
    try:
        exe=args.exe.resolve();both=['--native-clock','true','--native-random','true'];entry=['--entry','native.snapshot.main']
        tests=[([],None),(['--no-pause'],None), (entry,'Denied'),(entry+['--native-clock','true'],'Denied'),(entry+['--native-random','true'],'Denied'),
               (entry+both,None),(entry+both+['--native-bytes','0'],'LimitExceeded'),
               (entry+both+['--native-handles','1'],'LimitExceeded'),(entry+both+['--native-calls','1'],'LimitExceeded')]
        for options,error in tests:
            r=run(['wine',exe,*options]);require(r['exit_code']==0,r)
            data=json.loads(r['stdout']);value=data['value']['value']
            if error:require(value['case']=='Err' and value['payload']['value']['case']==error,r)
            else:
                require(value['case']=='Ok',r);fields=value['payload']['value']['fields']
                require(int(fields['monotonic_ms']['value'])>=0 and len(bytes.fromhex(fields['random_bytes']['value']))==16,r)
        for options in [entry+['--native-clock','false'],entry+['--native-calls','0'],entry+['--native-clock','true','--native-clock','true'],entry+['--native-listen','127.0.0.1:0'],entry+['--native-handles','01']]:
            r=run(['wine',exe,*options]);require(r['exit_code']==1 and json.loads(r['stdout'])['diagnostic']['code']=='invocation.cli.invalid',r)
        unit=out/'provider-tests.exe';r=run([cc,'-std=c11','-O2','-I',ROOT/'src',ROOT/'tests/native/windows_snapshot_harness.c','-static','-lbcrypt','-o',unit]);require(r['exit_code']==0,r)
        cases={'matrix':None,'deny':None,'bounds':None,'quotas':None,'rng-failure':None,'clock-overflow':None,
               'root-depth':'invocation.native.authority','unsupported':'invocation.native.operation_unsupported',
               'fuel':'resource.fuel.exhausted','heap':'resource.memory.exhausted'}
        for case,diagnostic in cases.items():
            r=run(['wine',unit,case])
            if diagnostic:require(r['exit_code']==1 and json.loads(r['stdout'])['diagnostic']['code']==diagnostic,r)
            else:require(r['exit_code']==0 and r['stdout']=='windows-provider-case-passed\n',r)
        result={'status':'pass','platform':'Wine Windows API compatibility layer, not actual Windows hardware','application_cases':len(tests)+5,'provider_cases':len(cases),'permission_pairs':64,'actual_windows_execution':None,'commands':records}
    except (RuntimeError,ValueError,OSError,subprocess.TimeoutExpired) as error:
        result={'status':'fail','failure':str(error),'commands':records}
    (out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='commands'}));return 0 if result['status']=='pass' else 1

if __name__=='__main__':raise SystemExit(main())
