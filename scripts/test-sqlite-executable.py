#!/usr/bin/env python3
"""Verify SQLite persistence, bindings, rollback and REST restart in a built Linux or Wine executable."""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import time

ROOT=Path(__file__).resolve().parents[1]

def require(ok,detail):
    if not ok:raise RuntimeError(detail)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--exe',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--wine-prefix',type=Path)
    p.add_argument('--harness',type=Path,help='optional Windows C provider harness; excludes Linux symlink case')
    a=p.parse_args();a.out.mkdir(parents=True,exist_ok=False);out=a.out.resolve()
    env=os.environ.copy();prefix=[]
    if a.wine_prefix:
        env.update(WINEPREFIX=str(a.wine_prefix.resolve()),WINEARCH='win64',WINEDEBUG='-all',WINEDLLOVERRIDES='mscoree,mshtml=');prefix=['wine']
    def native_path(path):return 'Z:'+str(path).replace('/','\\') if prefix else str(path)
    records=[];result={'status':'running','mode':'wine' if prefix else 'linux','actual_windows_execution':None,'records':records}
    def run(command):
        r=subprocess.run(prefix+[str(v) for v in command],capture_output=True,text=True,env=env,timeout=45)
        record={'command':prefix+[str(v) for v in command],'returncode':r.returncode,'stdout':r.stdout,'stderr':r.stderr};records.append(record)
        return r
    try:
        if a.harness:
            require(bool(prefix),'the optional harness profile is Windows/Wine')
            for case in ['open','schema','commit-busy','memory','corrupt','values','empty-invalid','transactions','policy','injection','budgets','stale','busy','handles','generation','deny','options','authority','fuel-cleanup','heap-cleanup']:
                r=run([a.harness.resolve(),case,native_path(out/(case+'.sqlite'))])
                fault=case in ['authority','fuel-cleanup','heap-cleanup']
                require(r.returncode==(1 if fault else 0) and not r.stderr,records[-1])
                if fault:
                    value=json.loads(r.stdout.splitlines()[0]);require(value['status']=='error',records[-1])
                    if case!='authority':require('sqlite-fault-cleanup-passed' in r.stdout,records[-1])
                else:require(r.stdout=='sqlite-adapter-case-passed\n',records[-1])
            result['provider_cases']=20
        # Non-ASCII paths exercise the wide launcher on Windows and UTF-8 engine paths on both platforms.
        database=out/'data-🐺.sqlite'
        def call(entry,args=(),role='read'):
            command=[a.exe.resolve(),'--entry','example.sqlite.users.'+entry,'--fuel','1000000','--heap-cells','1000000']
            for arg in args:command+=['--arg',arg]
            if role:command+=['--sqlite-file',native_path(database),'--sqlite-mode',role]
            r=run(command);require(r.returncode==0 and not r.stderr,records[-1]);value=json.loads(r.stdout)
            require(value['status']=='ok',value);return value['value']['value']
        def ok(value):require(value['case']=='Ok',value);return value['payload']['value']
        def rows(value):return [r['value']['fields']['values']['value']['elements'] for r in ok(value)['elements']]
        denied=call('list',role=None);require(denied['case']=='Err',denied)
        ok(call('setup',role='schema'));ok(call('setup',role='schema'))
        require(len(rows(call('list')))==2,'idempotent bootstrap')
        name="Grace 🐺'); DROP TABLE users;--\0"
        ok(call('save',['I64:3','Text:'+name.encode().hex()],role='write'))
        found=rows(call('get',['I64:3']));require(found[0][1]['value']['payload']['value']==name,found)
        ok(call('undo',role='write'));require(len(rows(call('list')))==3,'rollback failed')
        denied=call('save',['I64:1','Text:78'],role='read')
        require(denied['case']=='Err' and denied['payload']['value']['fields']['kind']['value']['case']=='Denied',denied)
        typed=rows(call('typed'))[0];require([v['value']['case'] for v in typed]==['Null','Integer','Decimal','String','Binary'],typed)
        require(typed[2]['value']['payload']['value']=='0x3ff4000000000000',typed)
        # Restart with a read-only grant and no bootstrap; data must survive every previous process exit.
        for cycle in range(2):
            with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
            command=[str(a.exe.resolve()),'--entry','example.sqlite.rest.main','--arg',f'I64:{port}','--arg','I64:2','--arg','Bool:false','--sqlite-file',native_path(database),'--sqlite-mode','read','--sqlite-calls','65536','--sqlite-steps','1000000','--native-clock','true','--native-listen',f'127.0.0.1:{port}','--native-calls','65536','--fuel','100000000','--heap-cells','16000000']
            proc=subprocess.Popen(prefix+command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env)
            try:
                deadline=time.monotonic()+45
                while True:
                    require(proc.poll() is None,'server exited before startup')
                    try:
                        with socket.create_connection(('127.0.0.1',port),timeout=.1):pass
                        break
                    except (ConnectionRefusedError,TimeoutError):
                        require(time.monotonic()<deadline,'startup timeout');time.sleep(.05)
                with socket.create_connection(('127.0.0.1',port),timeout=5) as s:
                    s.settimeout(8);s.sendall(b'GET /api/users/3 HTTP/1.1\r\nHost: local\r\n\r\n');s.shutdown(socket.SHUT_WR);data=b''
                    while True:
                        block=s.recv(8192)
                        if not block:break
                        data+=block
                require(data.startswith(b'HTTP/1.1 200'),data)
                body=json.loads(data.split(b'\r\n\r\n',1)[1]);require(body=={'id':'3','name':name},body)
                stdout,stderr=proc.communicate(timeout=15);require(proc.returncode==0 and not stderr,(stdout,stderr))
                records.append({'name':'read-only REST restart','cycle':cycle,'command':prefix+command,'wire_hex':data.hex(),'stdout':stdout.decode(),'stderr':stderr.decode()})
            finally:
                if proc.poll() is None:proc.kill();proc.communicate(timeout=15)
        # A separate engine reads the on-disk file only after all Wine processes have closed it.
        import sqlite3
        with sqlite3.connect(database) as oracle:
            require(oracle.execute('SELECT id,name FROM users ORDER BY id').fetchall()==[(1,'Luis'),(2,'Ada'),(3,name)],'independent persistence oracle')
        result.update(status='pass',application_operations=10,rest_restarts=2,independent_file_oracle=True)
    except BaseException as error:result.update(status='fail',error=str(error));raise
    finally:(out/'results.json').write_text(json.dumps(result,indent=2,ensure_ascii=True)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k!='records'}))

if __name__=='__main__':main()
