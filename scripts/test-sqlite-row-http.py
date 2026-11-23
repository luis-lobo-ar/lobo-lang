#!/usr/bin/env python3
"""Check that incompatible database rows become HTTP 500 responses in the real Linux executable."""
import argparse
import json
import socket
import sqlite3
import subprocess
import time
from pathlib import Path


def require(ok,detail):
    if not ok:raise RuntimeError(detail)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--exe',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    a.out.mkdir(parents=True,exist_ok=False);out=a.out.resolve();records=[]
    cases=[('null_name',1,None),('blob_name',1,sqlite3.Binary(b'name')),('integer_name',1,42),('text_id','one','Ada')]
    result={'status':'running','cases':records}
    try:
        for name,user_id,user_name in cases:
            database=out/(name+'.sqlite')
            with sqlite3.connect(database) as db:
                db.execute('CREATE TABLE users(id,name)');db.execute('INSERT INTO users VALUES(?,?)',(user_id,user_name))
            with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
            command=[str(a.exe.resolve()),'--entry','example.sqlite.rest.main','--arg',f'I64:{port}','--arg','I64:2','--arg','Bool:false','--sqlite-file',str(database),'--sqlite-mode','read','--native-clock','true','--native-listen',f'127.0.0.1:{port}','--fuel','10000000','--heap-cells','1000000']
            process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            try:
                deadline=time.monotonic()+10
                while True:
                    require(process.poll() is None,'server exited early')
                    try:
                        with socket.create_connection(('127.0.0.1',port),timeout=.1):pass
                        break
                    except (ConnectionRefusedError,TimeoutError):
                        require(time.monotonic()<deadline,'startup timed out');time.sleep(.01)
                with socket.create_connection(('127.0.0.1',port),timeout=3) as client:
                    client.settimeout(3);client.sendall(b'GET /api/users HTTP/1.1\r\nHost: local\r\n\r\n');client.shutdown(socket.SHUT_WR);wire=b''
                    while True:
                        chunk=client.recv(8192)
                        if not chunk:break
                        wire+=chunk
                stdout,stderr=process.communicate(timeout=5)
                records.append({'name':name,'command':command,'returncode':process.returncode,'stdout':stdout.decode(),'stderr':stderr.decode(),'wire_hex':wire.hex()})
                require(wire.startswith(b'HTTP/1.1 500 '),records[-1])
                require(process.returncode==0 and not stderr,records[-1])
                require(json.loads(stdout)['value']['value']['case']=='Ok',records[-1])
            finally:
                if process.poll() is None:process.kill();process.communicate(timeout=5)
        result['status']='pass'
    except BaseException as error:result.update(status='fail',error=str(error));raise
    finally:(out/'results.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'status':'pass','cases':len(records)}))

if __name__=='__main__':main()
