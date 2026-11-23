#!/usr/bin/env python3
"""Check Lobo's REST request parser and wire responses through compiled machine code."""
import argparse,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def verify(case,data):
    header,body=data.split(b'\r\n\r\n',1);lines=header.decode('ascii').split('\r\n');status=int(lines[0].split()[1]);headers=dict(line.split(': ',1) for line in lines[1:])
    if status!=case['status']:raise RuntimeError((case['name'],'status',status,case['status']))
    if headers.get('Connection')!='close' or headers.get('Content-Type')!='application/json':raise RuntimeError((case['name'],'headers',headers))
    if case['name']=='head':
        if body or int(headers['Content-Length'])<=0:raise RuntimeError('HEAD framing')
    elif int(headers['Content-Length'])!=len(body):raise RuntimeError((case['name'],'length'))
    if case['body'] is not None and json.loads(body)!=case['body']:raise RuntimeError((case['name'],'body',body))
    if status==405 and headers.get('Allow')!='GET, HEAD':raise RuntimeError('Missing Allow')
    return status

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--exe',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.mkdir(exist_ok=False,parents=True);records=[]
    try:
        for case in json.loads((ROOT/'tests/native/rest_cases.json').read_text()):
            command=[str(a.exe.resolve()),'--entry','example.rest.controller.respond','--arg','Bytes:'+case['request'].encode().hex(),'--fuel','10000000','--heap-cells','1000000']
            r=subprocess.run(command,capture_output=True,text=True,timeout=30);record={'name':case['name'],'returncode':r.returncode,'stdout':r.stdout,'stderr':r.stderr};records.append(record)
            if r.returncode or r.stderr:raise RuntimeError(record)
            v=json.loads(r.stdout);wire=bytes.fromhex(v['value']['value']);verify(case,wire)
    finally:(a.out/'results.json').write_text(json.dumps(records,indent=2)+'\n')
    print(json.dumps({'status':'pass','cases':len(records)}))
if __name__=='__main__':main()
