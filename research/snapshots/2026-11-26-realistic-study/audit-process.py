"""Supplemental process observations from preserved events; does not change registered scoring."""
import argparse,json
from collections import Counter
from pathlib import Path
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--raw',type=Path,required=True);a=parser.parse_args();root=Path(__file__).resolve().parents[3];raw=a.raw.resolve();results=json.loads((raw/'run/results.json').read_text());records=[]
 for row in results:
  assignment=row['assignment'];folder=raw/'run'/('trial-%02d'%assignment['index']);initial=folder/'initial';diagnostics=[];commands={}
  for line in (folder/'events.jsonl').read_text().splitlines():
   try:e=json.loads(line)
   except ValueError:continue
   item=e.get('item',{})
   if item.get('type')=='command_execution' and item.get('id'):commands[item['id']]=item
  for item in commands.values():
   if item.get('exit_code') in (None,0):continue
   for line in item.get('aggregated_output','').splitlines():
    try:value=json.loads(line)
    except ValueError:continue
    if isinstance(value,dict) and isinstance(value.get('diagnostic'),dict):diagnostics.append(dict(command=item.get('command'),diagnostic=value['diagnostic']))
  checks=[]
  for i,check in enumerate(row['checks']):
   path=folder/('check-%02d'%i)/'result.json';value=json.loads(path.read_text()) if path.exists() else None;edited=[]
   for name in ('logic.lobo','policy.lobo'):
    candidate=path.parent/'source'/name
    if candidate.exists() and candidate.read_bytes()!=(initial/name).read_bytes():edited.append(name)
   feedback=check['feedback'];checks.append(dict(index=i,start_s=check['start_s'],end_s=check['end_s'],status=feedback['status'],edited_files=edited,build_s=value['build']['wall_s'] if value and value.get('build') else None))
  first=next((c['end_s'] for c in checks if c['status']=='pass' and c['edited_files']),None)
  texts=[x.get('command','') for x in commands.values()];records.append(dict(assignment=assignment,status=row['status'],candidate_correct=row['grading']['accepted'],timely_complete=row['accepted'],known_usage=row['usage'] is not None,command_items=row['command_tool_items'],checks=checks,first_edited_visible_pass_s=first,seconds_after_visible_pass=row['agent_wall_s']-first if first is not None else None,visible_build_s=sum(c['build_s'] or 0 for c in checks),direct_command_diagnostics=diagnostics,skill_path_in_commands=any('.agents/skills/lobo-development-v2/SKILL.md' in x for x in texts),external_skill_path_commands=[x for x in texts if '/home/luis/.codex/skills' in x or '/home/luis/.agents/skills' in x],all_commands=texts))
 groups=[]
 for arm in ('control','v2'):
  selected=[r for r in records if r['assignment']['arm']==arm];groups.append(dict(arm=arm,visible_status_counts=dict(Counter(c['status'] for r in selected for c in r['checks'])),direct_diagnostic_codes=dict(Counter(d['diagnostic']['code'] for r in selected for d in r['direct_command_diagnostics'])),skill_path_observed=sum(r['skill_path_in_commands'] for r in selected),timeouts_after_visible_pass=sum(r['status']=='wall_limit' and r['first_edited_visible_pass_s'] is not None for r in selected)))
 output=dict(label='Supplemental descriptive process audit, no score changes',records=records,groups=groups);(root/'benchmarks/results/2026-11-26-realistic-process-audit.json').write_text(json.dumps(output,indent=2)+'\n');print(json.dumps(groups))
