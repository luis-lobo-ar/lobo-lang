#!/usr/bin/env python3
"""End-to-end process measurements for the frozen matched-study programs; no model calls."""
import argparse
import importlib.util
import itertools
import json
import os
import platform
import random
import shutil
import statistics
import subprocess
import time
from pathlib import Path
SPEC=importlib.util.spec_from_file_location('matched_study',Path(__file__).with_name('study.py'))
study=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(study)


def inputs(family,changed):
    if family=='ledger':
        return {'initial':20,'events':[(k,a) for k,a in ([(0,7),(1,3),(2,0),(1,1),(0,2),(3,0),(1,8),(4 if changed else 0,0)]*4)]}
    if family=='scheduler':
        jobs=[{'id':i,'cost':i%4,'deps':[] if i%3==0 else [i-1],**({'priority':(i*7)%9-4} if changed else {})} for i in range(12)]
        return {'jobs':jobs[::2]+jobs[1::2],'budget':16}
    blob=b''
    for i in range(32):
        data=bytes((i*13+j)%256 for j in range(i%8+1));header=bytes([len(data)])
        if changed and i%4==0:header=b'\xff\xfe'+len(data).to_bytes(2,'little')
        blob+=header+data+bytes([sum(data)%256])
    return {'data':blob.hex(),'limit':8}


def checksum(family,value):
    if family=='ledger':return value['balance']+value['rejected']+value['peak']+sum(value['audit'])
    if family=='scheduler':return value['spent']+value['blocked']+sum(value['order'])
    return value['consumed']+value['error']+sum(len(bytes.fromhex(v))+sum(bytes.fromhex(v)) for v in value['frames'])


def expected(family,changed,args,repeats):
    if repeats is None:return study.oracle(family,args,changed)
    key={'ledger':'initial','scheduler':'budget','frames':'limit'}[family]
    return sum(checksum(family,study.oracle(family,args|{key:args[key]+i%3},changed)) for i in range(repeats))


def command(folder,language,family,args,binary,repeats):
    if language=='lobo':
        cmd=[str(binary),'--entry','study.'+family+('.entry' if repeats is None else '.bench'),'--fuel','100000000','--heap-cells','4000000']
        for arg in study.lobo_args(family,args)+([] if repeats is None else ['I64:'+str(repeats)]):cmd+=['--arg',arg]
        return cmd
    return ([str(binary)] if language=='rust' else ['python3',str(folder/'app.py')])+study.other_args(family,args)+([] if repeats is None else ['--bench',str(repeats)])


def measure(folder,language,family,args,binary,repeats):
    cmd=command(folder,language,family,args,binary,repeats);start=time.perf_counter_ns()
    try:
        process=subprocess.run(cmd,cwd=folder,capture_output=True,text=True,timeout=15);elapsed=(time.perf_counter_ns()-start)/1e6
        result={'wall_ms':elapsed,'exit_code':process.returncode,'stdout_bytes':len(process.stdout.encode()),'stderr':process.stderr}
        if process.returncode:result['error']=process.stdout[:4096]
        else:
            try:
                value=json.loads(process.stdout);result['actual']=study.untyped(value['value']) if language=='lobo' else value
            except Exception as exc:result['error']=repr(exc)
        return result
    except subprocess.TimeoutExpired:return {'wall_ms':(time.perf_counter_ns()-start)/1e6,'error':'timeout'}


def distribution(values):
    return {'n':len(values),'median_ms':statistics.median(values),'min_ms':min(values),'max_ms':max(values)} if values else None


def version(command):
    p=subprocess.run(command,capture_output=True,text=True);return {'command':command,'exit_code':p.returncode,'stdout':p.stdout,'stderr':p.stderr}


def cohort():
    rows=[]
    for family,changed,language in itertools.product(study.FAMILIES,[False,True],study.LANGUAGES):
        rows.append({'id':'reference-'+family+'-'+str(int(changed))+'-'+language,'cohort':'canonical','family':family,'changed':changed,'language':language,'files':study.sources(family,language,changed)})
    results=json.loads((study.PRIVATE/'attempts-amendment-1/results.json').read_text())
    if len(results)!=54:raise ValueError('wait until all 54 scheduled attempts finish before benchmarking')
    for result in results:
        row=result['assignment']
        if row['phase']=='comprehension' or not result['accepted']:continue
        folder=study.PRIVATE/'attempts-amendment-1'/('trial-%02d'%row['index'])/'heldout/source'
        fixed=study.sources(row['family'],row['language'],row['phase']=='change')
        files={name:(folder/name).read_text() for name in fixed}
        rows.append({'id':'agent-%02d'%row['index'],'cohort':'accepted-agent','family':row['family'],'changed':row['phase']=='change','language':row['language'],'trial_index':row['index'],'files':files})
    return rows


def profile(row,out):
    args=inputs(row['family'],row['changed']);folder=out/row['id']/'source';binary=out/row['id']/'program'
    cmd=command(folder,row['language'],row['family'],args,binary,100)
    result={'id':row['id'],'outside_timing':True}
    for name,prefix in [('syscalls',['strace','-c','-o',str(out/row['id']/'strace.txt')]),('cpu_memory',['/usr/bin/time','-v','-o',str(out/row['id']/'time.txt')]),('hardware',['perf','stat','-e','task-clock,cycles,instructions','-o',str(out/row['id']/'perf.txt'),'--'])]:
        try:
            p=subprocess.run(prefix+cmd,cwd=folder,capture_output=True,text=True,timeout=30);result[name]={'exit_code':p.returncode,'stderr':p.stderr}
        except Exception as exc:result[name]={'error':repr(exc)}
    if row['language']=='python':
        p=subprocess.run(['python3','-m','cProfile','-s','cumtime']+cmd[1:],cwd=folder,capture_output=True,text=True,timeout=30);(out/row['id']/'cprofile.txt').write_text(p.stdout);result['cprofile']={'exit_code':p.returncode,'stderr':p.stderr}
    return result


def run(out):
    out.mkdir(parents=True,exist_ok=False);rows=cohort();environment={'date':'2026-11-23','platform':platform.platform(),'machine':platform.machine(),'processor':platform.processor(),'logical_cpus':os.cpu_count(),'load_before':os.getloadavg(),'versions':[version(c) for c in [['rustc','--version'],['clang','--version'],['python3','--version']]],'cpuinfo':Path('/proc/cpuinfo').read_text(),'limits':Path('/proc/self/limits').read_text(),'cgroup_cpu_max':Path('/sys/fs/cgroup/cpu.max').read_text() if Path('/sys/fs/cgroup/cpu.max').exists() else None,'claim':'Scoped observations of optimized canonical and accepted-agent CLI programs on the shared tasks; not a language-wide ranking','timing':'subprocess launch through process exit, including adapter input parsing and output; normalization/grading after timing','runtime_repeats':7,'compile_repeats':5,'modes':{'single':None,'empty_batch':0,'batch100':100},'lobo_limits':{'fuel':100000000,'heap_cells':4000000},'python_cache':'py_compile creates warm bytecode caches; timing does not include compiling Python source'}
    study.write_json(out/'environment.json',environment)
    for row in rows:
        folder=out/row['id']/'source';study.save_sources(folder,row['files']);row['source_sizes']={name:len(text.encode()) for name,text in row.pop('files').items()};row['source_sha256']={p.name:study.digest(p.read_bytes()) for p in folder.iterdir()};row['compiles']=[];row['samples']=[];row['warmups']=[]
    # Every native build gets a fresh output path. Interleave arms/rounds; retain exact commands and failures.
    for repeat in range(5):
        order=list(rows);random.Random(20261123+repeat).shuffle(order)
        for row in order:
            folder=out/row['id']/'source';binary=out/row['id']/('program-'+str(repeat));started=time.perf_counter_ns()
            try:result=study.build(folder,row['language'],binary)
            except Exception as exc:result={'ok':False,'error':repr(exc)}
            result.update(repeat=repeat,wall_ms=(time.perf_counter_ns()-started)/1e6);row['compiles'].append(result)
            if repeat==0 and result['ok'] and row['language']!='python':shutil.copy2(binary,out/row['id']/'program')
        study.write_json(out/'results.json',rows)
    rows_to_time=[]
    for row in rows:
        row['all_builds_ok']=all(r['ok'] for r in row['compiles']);row['compile_distribution']=distribution([r['wall_ms'] for r in row['compiles'] if r['ok']]);binary=out/row['id']/'program';row['binary_bytes']=binary.stat().st_size if binary.exists() else None
        if not row['all_builds_ok']:continue
        rows_to_time.append(row)
        for mode,repeats in environment['modes'].items():
            a=inputs(row['family'],row['changed']);result=measure(out/row['id']/'source',row['language'],row['family'],a,binary,repeats);result.update(mode=mode,expected=expected(row['family'],row['changed'],a,repeats));result['ok']='actual' in result and study.equal(result['actual'],result['expected']);row['warmups'].append(result)
    for repeat in range(7):
        # Within each cohort/task/variant/mode, rotate and reverse the language order each round.
        for group in itertools.product(['canonical','accepted-agent'],study.FAMILIES,[False,True],environment['modes']):
            cohort_name,family,changed,mode=group;selected=[r for r in rows_to_time if r['cohort']==cohort_name and r['family']==family and r['changed']==changed]
            order=study.LANGUAGES[repeat%3:]+study.LANGUAGES[:repeat%3]
            if repeat%2:order=tuple(reversed(order))
            selected.sort(key=lambda r:(order.index(r['language']),r.get('trial_index',-1)))
            for row in selected:
                args=inputs(family,changed);repeats=environment['modes'][mode];result=measure(out/row['id']/'source',row['language'],family,args,out/row['id']/'program',repeats);result.update(mode=mode,repeat=repeat,load=os.getloadavg(),expected=expected(family,changed,args,repeats));result['ok']='actual' in result and study.equal(result['actual'],result['expected']);row['samples'].append(result)
        study.write_json(out/'results.json',rows)
    for row in rows:
        row['runtime_distributions']={mode:distribution([s['wall_ms'] for s in row['samples'] if s['mode']==mode and s['ok']]) for mode in environment['modes']};row['error_count']=sum(not x['ok'] for x in row['samples']+row['warmups'])+sum(not x['ok'] for x in row['compiles'])
    study.write_json(out/'results.json',rows)
    profiles=[profile(row,out) for row in rows if row['cohort']=='canonical' and not row['changed'] and row['all_builds_ok']];study.write_json(out/'profiles.json',profiles)
    environment['load_after']=os.getloadavg();study.write_json(out/'environment.json',environment)
    return rows

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True,type=Path);args=parser.parse_args();run(args.out.resolve())
