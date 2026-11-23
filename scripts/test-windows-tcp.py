#!/usr/bin/env python3
"""Build and run the real Windows TCP provider tests under an isolated Wine prefix."""
import argparse,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--toolchain',type=Path,required=True);p.add_argument('--wine-prefix',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.mkdir(exist_ok=False,parents=True);records=[]
    env=os.environ.copy();env.update(WINEPREFIX=str(a.wine_prefix.resolve()),WINEARCH='win64',WINEDEBUG='-all',WINEDLLOVERRIDES='mscoree,mshtml=')
    def run(command,expected=0):
        v=subprocess.run([str(x) for x in command],capture_output=True,text=True,timeout=60,env=env,cwd=ROOT);records.append({'command':[str(x) for x in command],'exit_code':v.returncode,'stdout':v.stdout,'stderr':v.stderr});(a.out/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
        if v.returncode!=expected:raise RuntimeError(records[-1])
        return v.stdout
    exe=a.out.resolve()/'provider.exe';cc=a.toolchain.resolve()/'bin/x86_64-w64-mingw32-clang'
    run([cc,'-std=c11','-O3','-Isrc',ROOT/'tests/native/windows_tcp_harness.c','-static','-lws2_32','-lbcrypt','-o',exe])
    for case in ['options','deny','authority','bounds','quota','stale','close-failure','cleanup','generation','roundtrip']:
        if run(['wine',exe,case]).strip()!='windows-tcp-case-passed':raise RuntimeError(case)
    fault=run(['wine',exe,'fuel-cleanup'],1)
    if json.loads(fault)['diagnostic']['code']!='resource.fuel.exhausted':raise RuntimeError(fault)
    result={'status':'pass','cases':11,'authority_mask_pairs':256,'actual_windows_execution':None,'commands':records};(a.out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='commands'}))
if __name__=='__main__':main()
