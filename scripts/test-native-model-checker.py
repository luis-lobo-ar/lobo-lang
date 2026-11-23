#!/usr/bin/env python3
"""Prove a known-broken baseline cannot pass when Python optimization is enabled."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tla2tools',type=Path,required=True)
    parser.add_argument('--java',default='java')
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();out=args.out.resolve();out.mkdir(parents=True,exist_ok=False)
    fixture=out/'fixture'
    for folder in ['src','verification/native','tests/native']:
        shutil.copytree(ROOT/folder,fixture/folder)
    for name in ['tests/native_formal.rs','tests/native_adapter.rs','scripts/check-native-models.py','scripts/test-native-model-checker.py']:
        destination=fixture/name;destination.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/name,destination)
    config=fixture/'verification/native/configs/authority.cfg'
    config.write_text(config.read_text().replace('Mutant = "none"','Mutant = "root"'))
    coverage=fixture/'verification/native/coverage.json';data=json.loads(coverage.read_text())
    data['files']['verification/native/configs/authority.cfg']['sha256']=hashlib.sha256(config.read_bytes()).hexdigest()
    coverage.write_text(json.dumps(data,indent=2)+'\n')
    command=[sys.executable,'-O',str(fixture/'scripts/check-native-models.py'),'--tla2tools',str(args.tla2tools.resolve()),'--java',args.java,'--models-only','--out',str(out/'run')]
    result=subprocess.run(command,capture_output=True,text=True,timeout=30)
    (out/'process.json').write_text(json.dumps({'command':command,'exit_code':result.returncode,'stdout':result.stdout,'stderr':result.stderr},indent=2)+'\n')
    log=out/'run/authority/tlc.log';summary=out/'run/results.json'
    valid=(result.returncode==1 and log.exists() and summary.exists()
           and 'Invariant EntryAuthority is violated' in log.read_text()
           and json.loads(summary.read_text())['status']=='fail')
    spec=importlib.util.spec_from_file_location('native_model_checker',ROOT/'scripts/check-native-models.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    states=module.trace_of(log.read_text()) if log.exists() else []
    clean_trace=bool(states) and all('coverage statistics' not in state['state_text'] for state in states)
    valid=valid and clean_trace
    report={'status':'pass' if valid else 'fail','counterexample_excludes_statistics':clean_trace,'known_broken_baseline':'authority-root','python_optimization':True,
            'broken_model_process_exit':result.returncode,'output':str(out)}
    (out/'results.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report));return 0 if valid else 1

if __name__=='__main__':sys.exit(main())
