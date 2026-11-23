#!/usr/bin/env python3
"""Cross-build a standalone Windows REST API from Lobo application and HTTP sources."""
import argparse,hashlib,json,re,shutil,subprocess,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCES=['examples/native/rest/app.lobo','examples/native/rest/controller.lobo','libraries/http-server/transport.lobo','libraries/http-server/server.lobo','libraries/http/http.lobo','libraries/json/json.lobo']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--toolchain',type=Path,required=True);p.add_argument('--compiler',type=Path,default=ROOT/'target/release/lobo');p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    tool=a.toolchain.resolve();cc=tool/'bin/x86_64-w64-mingw32-clang';readobj=tool/'bin/llvm-readobj'
    if not all(x.is_file() for x in [cc,readobj,a.compiler]):p.error('explicit installed compiler/toolchain required; no downloads')
    out=a.out.resolve();out.mkdir(exist_ok=False,parents=True);records=[]
    def run(command):
        q=subprocess.run([str(x) for x in command],cwd=ROOT,capture_output=True,text=True,timeout=120);records.append({'command':[str(x) for x in command],'exit_code':q.returncode,'stdout':q.stdout,'stderr':q.stderr});(out/'commands.json').write_text(json.dumps(records,indent=2)+'\n')
        if q.returncode:raise RuntimeError(records[-1])
        return q.stdout
    generated=out/'rest.generated.c';obj=out/'rest.o';exe=out/'LoboRestApi.exe'
    command=[a.compiler.resolve(),'compile',ROOT/SOURCES[0]]
    for source in SOURCES[1:]:command+=['--source',ROOT/source]
    run(command+['--emit','c','-o',generated])
    flags=['-std=c11','-O3','-g0','-DLOBO_WINDOWS_TCP','-ffunction-sections','-fdata-sections']
    run([cc,*flags,'-Dmain=lobo_generated_main','-c',generated,'-o',obj]);run([cc,*flags,ROOT/'examples/native/rest/launcher.c',obj,'-static','-lbcrypt','-lws2_32','-Wl,--gc-sections','-Wl,-s','-o',exe])
    data=exe.read_bytes();pe=int.from_bytes(data[60:64],'little')
    if data[:2]!=b'MZ' or data[pe:pe+4]!=b'PE\0\0' or int.from_bytes(data[pe+4:pe+6],'little')!=0x8664:raise RuntimeError('invalid PE x64')
    imports=run([readobj,'--coff-imports',exe]);(out/'imports.txt').write_text(imports);dlls=re.findall(r'^  Name: ([^\r\n]+)',imports,re.M)
    if not dlls or any(d.lower() not in {'kernel32.dll','bcrypt.dll','ws2_32.dll','ucrtbase.dll','msvcrt.dll'} and not d.lower().startswith('api-ms-win-crt-') for d in dlls):raise RuntimeError(('unexpected DLL imports',dlls))
    for source in SOURCES:
        for f in [ROOT/source,Path(str(ROOT/source)+'.ids.json')]:
            if f.exists():dest=out/'source'/f.relative_to(ROOT);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(f,dest)
    shutil.copy2(ROOT/'examples/native/rest/README.md',out/'README.md');shutil.copy2(tool/'LICENSE.TXT',out/'TOOLCHAIN-LICENSE.txt');notices=out/'notices';notices.mkdir()
    for f in (tool/'x86_64-w64-mingw32/share/mingw32').glob('COPYING*'):shutil.copy2(f,notices/f.name)
    inputs={s:sha(ROOT/s) for s in SOURCES}
    for s in SOURCES:
        f=ROOT/(s+'.ids.json')
        if f.exists():inputs[s+'.ids.json']=sha(f)
    for s in ['src/native_windows_tcp.c','src/native_api.c','src/native_windows_snapshot.c','examples/native/rest/launcher.c','scripts/build-native-rest.py']:inputs[s]=sha(ROOT/s)
    manifest={'format':'lobo-native-rest-build/0.1','target':'Windows 10/11 x64','executable':{'path':exe.name,'bytes':len(data),'sha256':sha(exe)},'sources':inputs,'compiler_sha256':sha(a.compiler),'profile':'LOBO_WINDOWS_TCP','dll_imports':dlls,'commands':records,'actual_windows_execution':None}
    (out/'build.json').write_text(json.dumps(manifest,indent=2)+'\n')
    package=out/'LoboRestApi-windows-x64.zip'
    with zipfile.ZipFile(package,'w',zipfile.ZIP_DEFLATED) as z:
        for f in [exe,out/'README.md',out/'TOOLCHAIN-LICENSE.txt',out/'build.json',*sorted((out/'source').rglob('*')),*sorted(notices.iterdir())]:
            if f.is_file():z.write(f,arcname=str(f.relative_to(out)))
    print(json.dumps({'status':'built','exe_bytes':len(data),'zip_bytes':package.stat().st_size,'dlls':dlls}))
if __name__=='__main__':main()
