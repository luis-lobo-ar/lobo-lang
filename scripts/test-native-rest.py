#!/usr/bin/env python3
"""Exercise the real REST executable over TCP, on Linux or through an isolated Wine prefix."""
import argparse,importlib.util,json,os,socket,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('protocol',ROOT/'scripts/test-native-rest-protocol.py');protocol=importlib.util.module_from_spec(spec);spec.loader.exec_module(protocol)
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--exe',type=Path,required=True);p.add_argument('--default-launch',action='store_true');p.add_argument('--entry',default='example.rest.main');p.add_argument('--entry-extra-arg',action='append',default=[]);p.add_argument('--sqlite-file');p.add_argument('--sqlite-mode',choices=['read','write','schema'],default='schema');p.add_argument('--cwd',type=Path);p.add_argument('--wine-prefix',type=Path);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.mkdir(exist_ok=False,parents=True)
    env=os.environ.copy();prefix=[]
    if a.wine_prefix:
        env.update(WINEPREFIX=str(a.wine_prefix.resolve()),WINEARCH='win64',WINEDEBUG='-all',WINEDLLOVERRIDES='mscoree,mshtml=');prefix=['wine']
    records=[];cases=json.loads((ROOT/'tests/native/rest_cases.json').read_text());total=len(cases)+8
    with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
    if a.default_launch:port=8080;total=128
    args=[str(a.exe.resolve()),'--entry',a.entry,'--arg',f'I64:{port}','--arg',f'I64:{total}','--native-clock','true','--native-listen',f'127.0.0.1:{port}','--native-calls','65536','--native-bytes','1048576','--fuel','100000000','--heap-cells','16000000']
    for atom in a.entry_extra_arg:args+=['--arg',atom]
    if a.sqlite_file:args+=['--sqlite-file',a.sqlite_file,'--sqlite-mode',a.sqlite_mode,'--sqlite-calls','65536','--sqlite-steps','1000000']
    if a.default_launch:args=[str(a.exe.resolve())]
    proc=subprocess.Popen(prefix+args,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env,cwd=a.cwd);count=0
    def exchange(request,chunks=None,delay=0,half_close=True):
        nonlocal count
        with socket.create_connection(('127.0.0.1',port),timeout=5) as s:
            count+=1;s.settimeout(8)
            if chunks:
                for chunk in chunks:s.sendall(chunk);time.sleep(delay)
            else:s.sendall(request)
            if half_close:s.shutdown(socket.SHUT_WR)
            data=b''
            while True:
                try:part=s.recv(8192)
                except ConnectionResetError:
                    if data:break
                    raise
                if not part:break
                data+=part
            return data
    try:
        deadline=time.monotonic()+60
        while True:
            if proc.poll() is not None:raise RuntimeError(('early exit',proc.communicate()))
            try:
                with socket.create_connection(('127.0.0.1',port),timeout=.1):count+=1
                break
            except (ConnectionRefusedError,TimeoutError):
                if time.monotonic()>=deadline:raise RuntimeError('startup timeout')
                time.sleep(.05)
        for case in cases:
            data=exchange(case['request'].encode());records.append({'name':case['name'],'wire_hex':data.hex()});protocol.verify(case,data)
        health={'name':'fragmented','status':200,'body':{'status':'ok','language':'lobo'}};req=b'GET /health HTTP/1.1\r\nHost: a\r\n\r\n'
        data=exchange(req,[req[:1],req[1:15],req[15:-1],req[-1:]],.02);protocol.verify(health,data);records.append({'name':'fragmented','wire_hex':data.hex()})
        data=exchange(b'GET /health',half_close=False);protocol.verify({'name':'timeout','status':408,'body':None},data);records.append({'name':'request timeout','wire_hex':data.hex()})
        data=exchange(b'GET /health HTTP/1.1\r\nHost: a\r\n');protocol.verify({'name':'truncated','status':400,'body':None},data);records.append({'name':'truncated EOF','wire_hex':data.hex()})
        # Occupied port fails with a typed OS error and does not disrupt the first server.
        occupied=subprocess.run(prefix+args,capture_output=True,text=True,env=env,timeout=15,cwd=a.cwd);v=json.loads(occupied.stdout.strip().splitlines()[-1])
        if occupied.returncode or v['value']['value']['case']!='Err' or v['value']['value']['payload']['value'] not in ['9','1']:raise RuntimeError(('occupied port',occupied.stdout,occupied.stderr))
        records.append({'name':'occupied port','stdout':occupied.stdout})
        # Explicit arguments never inherit the launcher's demo grants.
        denied=subprocess.run(prefix+[str(a.exe.resolve()),'--entry',a.entry,'--arg',f'I64:{port}','--arg','I64:1',*[v for atom in a.entry_extra_arg for v in ['--arg',atom]]],capture_output=True,text=True,env=env,timeout=15,cwd=a.cwd);v=json.loads(denied.stdout)
        if denied.returncode or v['value']['value']['case']!='Err' or v['value']['value']['payload']['value']!='1':raise RuntimeError(('default deny',denied.stdout))
        records.append({'name':'default deny','stdout':denied.stdout})
        while count<total:
            data=exchange(req);protocol.verify(health,data)
        stdout,stderr=proc.communicate(timeout=15);v=json.loads(stdout.decode().strip().splitlines()[-1])
        if proc.returncode or v['value']['value']['case']!='Ok' or v['value']['value']['payload']['value']!=str(total):raise RuntimeError(('server exit',proc.returncode,stdout,stderr))
        result={'status':'pass','mode':'wine' if prefix else 'linux','cases':len(records),'connections':count,'server_stdout':stdout.decode(),'server_stderr':stderr.decode(),'command':prefix+args,'records':records,'actual_windows_execution':None}
    except BaseException as e:
        proc.kill();stdout,stderr=proc.communicate(timeout=15);result={'status':'fail','error':str(e),'records':records,'connections':count,'stdout':stdout.decode(errors='replace'),'stderr':stderr.decode(errors='replace')};raise
    finally:(a.out/'results.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ['records','command','server_stdout','server_stderr']}))
if __name__=='__main__':main()
