#!/usr/bin/env python3
"""Report the two requested Luna configurations without changing frozen scoring."""
import argparse
from collections import Counter
import importlib.util
import itertools
import json
from pathlib import Path

HERE=Path(__file__).resolve().parent

def load(name):
    spec=importlib.util.spec_from_file_location(name,HERE/(name+'.py'));module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

luna=load('luna');base_report=load('report');study=luna.load_base()

def failure_categories(row):
    if row['accepted']:return []
    result=[];grading=row['grading']
    if row['status']!='completed':result.append(row['status'])
    if grading.get('issues'):result.append('scope_rejected')
    if grading.get('infrastructure_error'):result.append('grading_infrastructure_error')
    if grading.get('build') is not None and not grading['build']['ok']:result.append('compile_failed')
    if grading.get('cases') and any(not c['ok'] for c in grading['cases']):result.append('program_observation_failed')
    if 'checks' in grading and not grading['accepted']:result.append('comprehension_items_failed')
    return result or ['other_protocol_failure']

def tokens(row):
    value=row.get('usage')
    return value['input_tokens']+value['output_tokens'] if isinstance(value,dict) and 'input_tokens' in value and 'output_tokens' in value else None

def compare_pairs(left,right):
    lookup={r['assignment']['index']:r for r in right};pairs=[]
    for row in left:
        i=row['assignment']['index'];other=lookup.get(i)
        if other is None:continue
        if row['assignment']!=other['assignment']:raise ValueError('mismatched assignment')
        a,b=tokens(row),tokens(other);wall_left,wall_right=row.get('agent_wall_s'),other.get('agent_wall_s')
        pairs.append({'assignment':row['assignment'],'left_accepted':row['accepted'],'right_accepted':other['accepted'],'left_status':row['status'],'right_status':other['status'],'left_tokens':a,'right_tokens':b,'left_over_right_tokens':a/b if a is not None and b else None,'left_over_right_wall':wall_left/wall_right if wall_left is not None and wall_right else None})
    return pairs

def summarize_pairs(pairs):
    return {'paired_attempts':len(pairs),'both_accepted':sum(p['left_accepted'] and p['right_accepted'] for p in pairs),'only_left_accepted':sum(p['left_accepted'] and not p['right_accepted'] for p in pairs),'only_right_accepted':sum(p['right_accepted'] and not p['left_accepted'] for p in pairs),'neither_accepted':sum(not p['left_accepted'] and not p['right_accepted'] for p in pairs),'reported_token_ratio_known_pairs':base_report.stats([p['left_over_right_tokens'] for p in pairs if p['left_over_right_tokens'] is not None]),'all_attempt_wall_ratio':base_report.stats([p['left_over_right_wall'] for p in pairs if p['left_over_right_wall'] is not None])}

def verify_rows(rows,schedule):
    assignments={r['index']:r for r in schedule};seen=set()
    for row in rows:
        i=row['assignment']['index']
        if i in seen or assignments.get(i)!=row['assignment']:raise ValueError('duplicate/mismatched assignment')
        seen.add(i)
    return [r for r in schedule if r['index'] not in seen]

def publish(out,fixtures_out):
    private=luna.PRIVATE;registration=json.loads((private/'registration/manifest.json').read_text());root=private/'attempts'
    if not (root/'environment-after.json').exists():raise ValueError('wait until both cohorts finish or stop under their registered policy')
    fixtures_out.mkdir(parents=True,exist_ok=False);cohorts=[];all_rows=[];canaries=[];raw_by_config={}
    for effort in registration['ordered_cohorts']:
        folder=root/effort;rows=json.loads((folder/'results.json').read_text());remaining=verify_rows(rows,registration['schedule']);availability=json.loads((folder/'availability.json').read_text());canaries.append(availability);all_rows+=rows;raw_by_config['luna-'+effort]=rows
        destination=fixtures_out/effort;destination.mkdir();attempts=[];manifest=[]
        for row in rows:
            assignment=row['assignment'];i=assignment['index'];trial=folder/('trial-%02d'%i);value=base_report.normalized(row,trial);value['configuration']=row['configuration'];value['failure_categories']=failure_categories(row)
            value['functional_evaluation_status']='scope_rejected' if row['grading'].get('issues') else 'compile_failed' if row['grading'].get('build') and not row['grading']['build']['ok'] else 'evaluated' if row['grading'].get('cases') else 'comprehension' if assignment['phase']=='comprehension' else 'not_evaluated'
            bundle={'configuration':row['configuration'],'assignment':assignment}
            if assignment['phase']=='comprehension':bundle['answer']=(trial/'last-message.txt').read_text() if (trial/'last-message.txt').exists() else None
            else:
                source=trial/'heldout/source';bundle['files']={p.name:p.read_text() for p in source.iterdir() if p.is_file() and p.suffix in {'.lobo','.json','.rs','.py'} and p.name not in {'TASK.txt','GUIDE.txt','check.py','AGENTS.md'}}
            path=destination/('trial-%02d.json'%i);study.write_json(path,bundle);value['public_bundle']=str(path.relative_to(luna.ROOT));value['public_bundle_sha256']=luna.sha(path);manifest.append({'path':value['public_bundle'],'sha256':value['public_bundle_sha256']});attempts.append(value)
        groups=[]
        for phase,language in itertools.product(study.PHASES,study.LANGUAGES):
            subset=[r for r in rows if r['assignment']['phase']==phase and r['assignment']['language']==language];success=[r for r in subset if r['accepted']]
            groups.append({'phase':phase,'language':language,'scheduled':6}|base_report.aggregate(subset)|{'failure_categories':dict(Counter(c for r in subset for c in failure_categories(r))),'successful_only_wall_seconds':base_report.stats([r['agent_wall_s'] for r in success]),'successful_only_reported_tokens':base_report.stats([tokens(r) for r in success if tokens(r) is not None])})
        cohorts.append({'model':luna.MODEL,'reasoning':effort,'scheduled':54,'attempted':len(rows),'accepted':sum(r['accepted'] for r in rows),'unattempted':remaining,'availability':availability,'accounting':base_report.account(rows),'groups':groups,'attempts':attempts,'fixture_manifest':manifest,'failure_categories':dict(Counter(c for r in rows for c in failure_categories(r)))})
    baseline=json.loads((study.PRIVATE/'attempts-amendment-1/results.json').read_text());raw_by_config['sol-high']=baseline;comparisons=[]
    for left,right in [('luna-high','sol-high'),('luna-low','sol-high'),('luna-low','luna-high')]:
        pairs=compare_pairs(raw_by_config[left],raw_by_config[right]);groups=[]
        for phase,language in itertools.product(study.PHASES,study.LANGUAGES):
            subset=[p for p in pairs if p['assignment']['phase']==phase and p['assignment']['language']==language]
            groups.append({'phase':phase,'language':language}|summarize_pairs(subset))
        comparisons.append({'left':left,'right':right,'groups':groups,'pairs':pairs})
    result={'date':'2026-11-23','format':'lobo-luna-reasoning-results/1','registration_sha256':luna.sha(private/'registration/manifest.json'),'baseline_data_sha256':luna.sha(luna.ROOT/'benchmarks/results/2026-11-23-matched-agent-study.json'),'frozen_version':'1.11.2','cohorts':cohorts,'comparisons':comparisons,'all_new_scored_accounting':base_report.account(all_rows),'all_new_participant_and_canary_accounting':base_report.account(all_rows+canaries),'accounting_unit':'CLI agent conversations, each potentially containing multiple underlying model requests; parent/harness development and billing unknown','environment_before':json.loads((root/'environment-before.json').read_text()),'environment_after':json.loads((root/'environment-after.json').read_text()),'limitations':registration['limitations']+['No hard token cap; all-attempt and successful-only costs are separate','Scope rejection can leave functional behavior ungraded; do not label it a proven algorithm defect','Runtime of new generated programs was not benchmarked; older binary timings remain unchanged','Three small task families, two repetitions and multiple descriptive comparisons; no population ranking or causal effect claim']}
    study.write_json(out,result);return result

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',type=Path,required=True);parser.add_argument('--fixtures-out',type=Path,required=True);args=parser.parse_args();publish(args.out.resolve(),args.fixtures_out.resolve())
