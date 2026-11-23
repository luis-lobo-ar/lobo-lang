#!/usr/bin/env python3
"""Publish normalized matched-study evidence, retaining failed attempts and unknown usage."""
import argparse
import importlib.util
import itertools
import json
import statistics
from pathlib import Path
SPEC=importlib.util.spec_from_file_location('matched',Path(__file__).with_name('study.py'))
study=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(study)


def account(rows):
    known=[r['usage'] for r in rows if isinstance(r.get('usage'),dict) and 'input_tokens' in r['usage'] and 'output_tokens' in r['usage']]
    sums={key:sum(u.get(key,0) for u in known) for key in ['input_tokens','cached_input_tokens','cache_write_input_tokens','output_tokens','reasoning_output_tokens']}
    return {'attempted':len(rows),'known':len(known),'unknown':len(rows)-len(known),'complete':len(known)==len(rows),'reported_usage_sums':sums,'reported_input_plus_output_tokens':sums['input_tokens']+sums['output_tokens'],'reported_uncached_input_tokens':sum(u['input_tokens']-u['cached_input_tokens'] for u in known if 'cached_input_tokens' in u),'cache_accounting':'cached input is a subset of input; reasoning output is not added again','billing_cost':None}


def stats(values):
    return {'n':len(values),'median':statistics.median(values),'min':min(values),'max':max(values)} if values else None


def aggregate(rows):
    return {'attempted':len(rows),'accepted':sum(r['accepted'] for r in rows),'final_functional_accepted':sum(r['grading']['accepted'] for r in rows),'wall_seconds':stats([r['agent_wall_s'] for r in rows if r.get('agent_wall_s') is not None]),'command_items':stats([r['command_tool_items'] for r in rows if r.get('command_tool_items') is not None]),'visible_checks':stats([len(r['checks']) for r in rows if r.get('checks') is not None]),'input_tokens':stats([r['usage']['input_tokens'] for r in rows if isinstance(r.get('usage'),dict) and 'input_tokens' in r['usage']]),'output_tokens':stats([r['usage']['output_tokens'] for r in rows if isinstance(r.get('usage'),dict) and 'output_tokens' in r['usage']]),'accounting':account(rows),'comprehension_items_passed':sum(r['grading'].get('passed',0) for r in rows),'comprehension_items_total':sum(r['grading'].get('total',0) for r in rows)}


def verify_assignments(rows,schedule):
    expected={r['index']:r for r in schedule};seen=set()
    for row in rows:
        assignment=row['assignment'];i=assignment['index']
        if i in seen or expected.get(i)!=assignment:raise ValueError('duplicate or mismatched assignment: '+str(i))
        seen.add(i)
    if len(rows)!=len(schedule):raise ValueError('incomplete cohort: '+str(len(rows)))


def normalized(row,folder):
    result={key:row.get(key) for key in ['assignment','status','accepted','agent_wall_s','command_tool_items','check_requests','item_types','usage','started_unix','load_before','load_after','exit_code','reason']}
    grading=row['grading']
    if 'checks' in grading:result['grading']=grading
    else:
        observations=grading['cases'];result['grading']={'accepted':grading['accepted'],'issues':grading['issues'],'build':grading['build'],'passed':sum(c['ok'] for c in observations),'total':len(observations),'failures':[c for c in observations if not c['ok']]}
        if grading.get('infrastructure_error'):result['grading']['infrastructure_error']=grading['infrastructure_error']
    result['checks']=row.get('checks',[])
    source_name='logic.'+{'lobo':'lobo','rust':'rs','python':'py'}[row['assignment']['language']]
    initial=folder/'initial'/source_name
    result['check_trace']=[]
    for check in sorted(folder.glob('check-*')):
        candidate=check/'source'/source_name
        observation=check/'result.json'
        if not candidate.is_file() or not observation.is_file():continue
        assessment=json.loads(observation.read_text());compiled=assessment.get('build')
        result['check_trace'].append({'check':check.name,'initial_code':candidate.read_bytes()==initial.read_bytes(),'source_sha256':study.digest(candidate.read_bytes()),'compiled':compiled.get('ok') if compiled else None,'accepted':assessment['accepted'],'passed':sum(c['ok'] for c in assessment['cases']),'total':len(assessment['cases']),'issues':assessment['issues']})
    result['raw_hashes']={str(p.relative_to(folder)):study.digest(p.read_bytes()) for p in sorted(folder.rglob('*')) if p.is_file() and p.name not in {'program'} and '/__pycache__/' not in str(p) and not p.name.endswith('.pyc')}
    return result


def publish(out,fixtures_out):
    registration=json.loads((study.PRIVATE/'registration-amendment-1/manifest.json').read_text());raw=study.PRIVATE/'attempts-amendment-1';rows=json.loads((raw/'results.json').read_text());verify_assignments(rows,registration['schedule'])
    fixtures_out.mkdir(parents=True,exist_ok=False)
    original=json.loads((study.PRIVATE/'attempts/results.json').read_text());canaries=[]
    for name in ['transport-canaries-1','transport-canaries-amendment-1','transport-canaries-amendment-2']:
        canaries+=json.loads((study.PRIVATE/name/'results.json').read_text())
    result={'date':'2026-11-23','format':'lobo-matched-agent-study/1','frozen_version':'1.11.2','frozen_source_commit':registration['source_commit'],'model':registration['model'],'reasoning':registration['reasoning'],'limits':{key:registration[key] for key in ['deadline_s','command_tool_items','visible_checks','hard_token_cap']},'registration_sha256':study.digest((study.PRIVATE/'registration-amendment-1/manifest.json').read_bytes()),'primary_cohort':{'scheduled':54,'attempted':len(rows),'accepted':sum(r['accepted'] for r in rows),'accounting':account(rows)},'groups':[],'attempts':[],'original_cohort':{'scheduled':54,'attempted':len(original),'accepted':sum(r['accepted'] for r in original),'unattempted':json.loads((study.PRIVATE/'attempts/stopped.json').read_text())['remaining'],'accounting':account(original),'attempts':[normalized(r,study.PRIVATE/'attempts'/('trial-%02d'%r['assignment']['index'])) for r in original]},'excluded_transport_canaries':canaries,'all_model_calls_accounting':account(rows+original+canaries),'limitations':['One configured model alias, two repetitions per task/phase/language; no immutable model weights','Three small pure-function task families; not repository-scale or ecosystem workloads','Time/tool/check opportunity caps, not a hard token cap','Different language guide/source lengths, idioms and prior familiarity','Provider-managed cache and shared-host load; no billing estimate','One infrastructure restart; original cohort retained separately','No feature or feedback ablations; H1/H2/H3 mechanisms are not causally distinguished','Predictions also have a supplied contract; predicate-location item is cued, not blinded fault localization'],'fixture_manifest':[]}
    for phase,language in itertools.product(study.PHASES,study.LANGUAGES):
        subset=[r for r in rows if r['assignment']['phase']==phase and r['assignment']['language']==language];result['groups'].append({'phase':phase,'language':language}|aggregate(subset))
    for row in rows:
        i=row['assignment']['index'];folder=raw/('trial-%02d'%i);value=normalized(row,folder)
        bundle={'assignment':row['assignment']}
        if row['assignment']['phase']=='comprehension':
            bundle['answer']=(folder/'last-message.txt').read_text() if (folder/'last-message.txt').exists() else None
        else:
            bundle['files']={p.name:p.read_text() for p in (folder/'heldout/source').iterdir() if p.name not in {'check.py','TASK.txt','GUIDE.txt','AGENTS.md'} and p.is_file() and p.suffix in {'.lobo','.json','.rs','.py'}}
        path=fixtures_out/('trial-%02d.json'%i);study.write_json(path,bundle);value['public_bundle']=str(path.relative_to(study.ROOT));value['public_bundle_sha256']=study.digest(path.read_bytes());result['attempts'].append(value);result['fixture_manifest'].append({'path':str(path.relative_to(study.ROOT)),'sha256':value['public_bundle_sha256']})
    for family,language,changed in itertools.product(study.FAMILIES,study.LANGUAGES,[False,True]):
        path=fixtures_out/('reference-'+family+'-'+language+'-'+str(int(changed))+'.json');study.write_json(path,{'family':family,'language':language,'changed':changed,'files':study.sources(family,language,changed)});result['fixture_manifest'].append({'path':str(path.relative_to(study.ROOT)),'sha256':study.digest(path.read_bytes())})
    study.write_json(out,result);return result

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',type=Path,required=True);parser.add_argument('--fixtures-out',type=Path,required=True);args=parser.parse_args();publish(args.out.resolve(),args.fixtures_out.resolve())
