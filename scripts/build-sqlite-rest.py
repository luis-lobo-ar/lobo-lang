#!/usr/bin/env python3
"""Build the persistent Lobo SQLite REST demo with explicit tools and retained provenance."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import zipfile

ROOT=Path(__file__).resolve().parents[1]
SOURCES=json.loads((ROOT/'scripts/sqlite-rest-sources.json').read_text())

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler',type=Path,default=ROOT/'target/release/lobo')
    parser.add_argument('--toolchain',type=Path,help='installed LLVM-MinGW, required for Windows')
    parser.add_argument('--target',choices=['linux','windows'],required=True)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();out=args.out.resolve();out.mkdir(parents=True,exist_ok=False)
    records=[]
    def run(command,timeout=240):
        result=subprocess.run([str(x) for x in command],cwd=ROOT,capture_output=True,text=True,timeout=timeout)
        records.append({'command':[str(x) for x in command],'exit_code':result.returncode,'stdout':result.stdout,'stderr':result.stderr})
        (out/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
        if result.returncode:raise RuntimeError(records[-1])
        return result.stdout
    c=out/'application.c';exe=out/('LoboSqliteRest.exe' if args.target=='windows' else 'lobo-sqlite-rest')
    command=[args.compiler.resolve(),'compile',ROOT/SOURCES[0]]
    for source in SOURCES[1:]:command+=['--source',ROOT/source]
    run(command+['--emit','c','-o',c])
    flags=['-std=c11','-O3','-g0','-ffunction-sections','-fdata-sections']
    imports=[]
    if args.target=='windows':
        if not args.toolchain:parser.error('Windows requires an explicitly installed --toolchain; no downloads')
        tool=args.toolchain.resolve();cc=tool/'bin/x86_64-w64-mingw32-clang';obj=out/'application.o'
        if not cc.is_file():parser.error('missing Windows compiler')
        run([cc,*flags,'-DLOBO_WINDOWS_TCP','-Dmain=lobo_generated_main','-c',c,'-o',obj])
        run([cc,*flags,'-municode',ROOT/'examples/native/sqlite-rest/launcher.c',obj,'-static','-lbcrypt','-lws2_32','-Wl,--gc-sections','-Wl,-s','-o',exe])
        output=run([tool/'bin/llvm-readobj','--coff-imports',exe]);(out/'imports.txt').write_text(output)
        imports=re.findall(r'^  Name: ([^\r\n]+)',output,re.M)
        if not imports or any(name.lower() not in {'kernel32.dll','bcrypt.dll','ws2_32.dll','ucrtbase.dll','msvcrt.dll'} and not name.lower().startswith('api-ms-win-crt-') for name in imports):raise RuntimeError(('unexpected DLL imports',imports))
        data=exe.read_bytes();pe=int.from_bytes(data[60:64],'little')
        if data[:2]!=b'MZ' or data[pe:pe+4]!=b'PE\0\0' or int.from_bytes(data[pe+4:pe+6],'little')!=0x8664:raise RuntimeError('not Windows x64 PE')
        shutil.copy2(tool/'LICENSE.TXT',out/'TOOLCHAIN-LICENSE.txt');(out/'notices').mkdir()
        for path in (tool/'x86_64-w64-mingw32/share/mingw32').glob('COPYING*'):shutil.copy2(path,out/'notices'/path.name)
    else:run(['clang',*flags,c,'-Wl,--gc-sections','-Wl,-s','-o',exe])
    inputs={}
    for name in [*SOURCES,*[s+'.ids.json' for s in SOURCES if (ROOT/(s+'.ids.json')).is_file()],'src/native_sqlite.c','src/sqlite_api.h','src/native_api.h','src/sqlite_profile.h','vendor/sqlite/provenance.json','vendor/sqlite/sqlite3.c','vendor/sqlite/sqlite3.h','examples/native/sqlite-rest/launcher.c','scripts/build-sqlite-rest.py','scripts/sqlite-rest-sources.json']:
        path=ROOT/name;inputs[name]=sha(path)
        if name.endswith('.lobo') or name.endswith('.ids.json'):
            dest=out/'source'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(path,dest)
    for name in ['README.md']:
        path=ROOT/'examples/native/sqlite-rest'/name
        if path.is_file():shutil.copy2(path,out/name)
    for name in ['NOTICE.txt','provenance.json']:
        path=ROOT/'vendor/sqlite'/name
        if path.is_file():shutil.copy2(path,out/('SQLITE-'+name))
    manifest={'format':'lobo-sqlite-rest-build/0.1','target':args.target,'compiler_sha256':sha(args.compiler),'language':'core/1.11','native_abi':1,'sqlite_extension':1,'sqlite_version':'3.54.1','executable':{'path':exe.name,'bytes':exe.stat().st_size,'sha256':sha(exe)},'dll_imports':imports,'inputs':inputs,'commands':records,'actual_windows_execution':None}
    (out/'build.json').write_text(json.dumps(manifest,indent=2)+'\n')
    if args.target=='windows':
        with zipfile.ZipFile(out/'LoboSqliteRest-windows-x64.zip','w',zipfile.ZIP_DEFLATED) as archive:
            for path in sorted(out.rglob('*')):
                if path.is_file() and path.suffix not in ['.c','.o','.zip'] and path.name!='commands.json':archive.write(path,path.relative_to(out))
    print(json.dumps({'status':'built','target':args.target,'bytes':exe.stat().st_size,'executable':str(exe),'imports':imports}))

if __name__=='__main__':main()
