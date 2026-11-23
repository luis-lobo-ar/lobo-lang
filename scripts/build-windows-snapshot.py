#!/usr/bin/env python3
"""Build the unchanged Lobo snapshot as a standalone Windows x64 console executable."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile
ROOT=Path(__file__).resolve().parents[1]

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain',type=Path,required=True,help='installed LLVM-MinGW directory; never downloaded')
    parser.add_argument('--compiler',type=Path,default=ROOT/'target/release/lobo')
    parser.add_argument('--out',type=Path,required=True,help='new output directory')
    args=parser.parse_args();toolchain=args.toolchain.resolve();cc=toolchain/'bin/x86_64-w64-mingw32-clang';readobj=toolchain/'bin/llvm-readobj'
    if not cc.is_file() or not args.compiler.is_file():parser.error('The explicit compiler and cross-toolchain must exist')
    out=args.out.resolve();out.mkdir(parents=True,exist_ok=False);records=[]
    def run(command):
        result=subprocess.run([str(x) for x in command],cwd=ROOT,capture_output=True,text=True,timeout=120)
        record={'command':[str(x) for x in command],'exit_code':result.returncode,'stdout':result.stdout,'stderr':result.stderr};records.append(record)
        (out/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
        if result.returncode:raise RuntimeError(record)
        return result.stdout
    generated=out/'snapshot.generated.c';obj=out/'snapshot.o';exe=out/'LoboSnapshot.exe'
    run([args.compiler.resolve(),'compile',ROOT/'examples/native/snapshot.lobo','--source',ROOT/'libraries/native/native.lobo','--emit','c','-o',generated])
    flags=['-std=c11','-O3','-g0','-ffunction-sections','-fdata-sections','-D_WIN32_WINNT=0x0A00']
    run([cc,*flags,'-Dmain=lobo_generated_main','-c',generated,'-o',obj])
    run([cc,*flags,ROOT/'examples/native/windows/launcher.c',obj,'-static','-lbcrypt','-Wl,--gc-sections','-Wl,-s','-o',exe])
    header=exe.read_bytes()
    if header[:2]!=b'MZ':raise RuntimeError('Missing Windows executable header')
    pe=int.from_bytes(header[60:64],'little')
    if header[pe:pe+4]!=b'PE\0\0' or int.from_bytes(header[pe+4:pe+6],'little')!=0x8664:raise RuntimeError('Not a Windows x64 PE')
    imports=run([readobj,'--coff-imports',exe]);(out/'imports.txt').write_text(imports)
    import re
    dlls=re.findall(r'^  Name: ([^\r\n]+)',imports,re.M)
    allowed={'kernel32.dll','bcrypt.dll','ucrtbase.dll','msvcrt.dll'}
    if any(d.lower() not in allowed and not d.lower().startswith('api-ms-win-crt-') for d in dlls):raise RuntimeError(('Unexpected runtime DLL',dlls))
    license_path=toolchain/'LICENSE.TXT'
    if license_path.is_file():(out/'TOOLCHAIN-LICENSE.txt').write_bytes(license_path.read_bytes())
    notices=out/'notices';notices.mkdir()
    for license_file in (toolchain/'x86_64-w64-mingw32/share/mingw32').glob('COPYING*'):
        (notices/license_file.name).write_bytes(license_file.read_bytes())
    (out/'README.txt').write_text('Lobo snapshot demo - Windows 10/11 x64\n\nDouble-click LoboSnapshot.exe. It runs the compiled Lobo program with clock/random\npermission, prints a typed JSON snapshot and waits for Enter in an interactive console.\nNo WSL, Rust, Lobo installation or extra runtime DLL download is needed.\n\nFor automation, --no-pause runs the same demo without waiting.\n\nExplicit native arguments use the normal default-deny invocation policy:\nLoboSnapshot.exe --entry native.snapshot.main --native-clock true --native-random true\n\nThe program body is generated from snapshot.lobo. Networking is unavailable in this\nWindows profile. This local unsigned build is tested under Wine; actual Windows\nvalidation is recorded separately and must not be inferred from Wine.\n',encoding='utf-8')
    for name in ['snapshot.lobo','snapshot.lobo.ids.json']:(out/name).write_bytes((ROOT/'examples/native'/name).read_bytes())
    manifest={'format':'lobo-windows-snapshot-build/0.1','target':'Windows 10/11 x64','application_source':'examples/native/snapshot.lobo','standalone':True,'default_launch_grants':['clock','random'],'unsupported':['networking','full native Windows compiler distribution'],'dll_imports':dlls,'executable':{'path':str(exe),'bytes':exe.stat().st_size,'sha256':hashlib.sha256(header).hexdigest()},'compiler_sha256':hashlib.sha256(args.compiler.read_bytes()).hexdigest(),'provider_sha256':hashlib.sha256((ROOT/'src/native_windows_snapshot.c').read_bytes()).hexdigest(),'commands':records,'actual_windows_execution':None}
    (out/'build.json').write_text(json.dumps(manifest,indent=2)+'\n')
    with zipfile.ZipFile(out/'LoboSnapshot-windows-x64.zip','w',zipfile.ZIP_DEFLATED) as archive:
        for name in ['LoboSnapshot.exe','README.txt','TOOLCHAIN-LICENSE.txt','snapshot.lobo','snapshot.lobo.ids.json','build.json']:
            if (out/name).exists():archive.write(out/name,arcname=name)
        for notice in sorted(notices.iterdir()):archive.write(notice,arcname='notices/'+notice.name)
    print(json.dumps({'status':'built','exe':str(exe),'zip':str(out/'LoboSnapshot-windows-x64.zip'),'bytes':exe.stat().st_size,'dlls':dlls}))

if __name__=='__main__':main()
