"""Read-only audits and complete candidate preservation after all participant writers finish."""
from pathlib import Path
import hashlib,importlib.util,json,shutil,re
R=Path.cwd();P=R/'benchmarks/private-evidence/development-study-20261124';run=P/'run'
spec=importlib.util.spec_from_file_location('devpost',R/'benchmarks/matched-study/development.py');s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
assert (run/'environment-after.json').exists()
reg=json.loads((P/'registration/manifest.json').read_text());rows=json.loads((run/'results.json').read_text());old=json.loads((P/'prior-paper-pins.json').read_text());idx=json.loads((R/'research/local-evidence.json').read_text())
assert idx['entries']==old['entries'] and all(s.sha(R/f['path'])==f['sha256'] for f in old['pins'])
assert all(s.sha(R/f)==h for f,h in reg['code'].items()) and s.hashed(s.materials())==reg['material_hashes']
audits=[]
for row in rows:
 a=row['assignment'];folder=run/('trial-%02d'%a['index']);workspace=Path(row['workspace']);initial={str(f.relative_to(folder/'initial')):f.read_text() for f in (folder/'initial').rglob('*') if f.is_file()}
 assert s.hashed(initial)==reg['inputs'][a['family']+'/'+a['arm']]['files']
 changed=[]
 for f in workspace.rglob('*'):
  if f.is_file() and not f.is_symlink():
   name=str(f.relative_to(workspace))
   if name not in initial or hashlib.sha256(initial[name].encode()).hexdigest()!=s.sha(f):
    destination=folder/'candidate-final'/name;destination.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,destination);changed.append(dict(path=name,sha256=s.sha(f),bytes=f.stat().st_size))
 def signatures(source):
  return {name:re.sub(r'\s+','',decl) for name,decl in re.findall(r'fn\s+(run|convert)\s*(\([^)]*\)\s*->\s*[^{{]+)\{',source)}
 required=signatures(initial['logic.lobo']);actual_signatures=signatures((folder/'heldout/source/logic.lobo').read_text());assert required==actual_signatures
 result=row['grading'];cases=result['cases'];assert result['passed']==sum(c['ok'] for c in cases)
 assert result['total']==len(s.C.cases(a['family'],True))
 for i,c in enumerate(cases):
  assert c['input']==s.C.cases(a['family'],True)[i]
  assert c['expected']==s.C.oracle(a['family'],c['input'])
  assert c['ok']==s.B.equal(c['expected'],c['actual'])
 if cases:assert len(cases)==result['total'] and result['accepted']==all(c['ok'] for c in cases)
 events=[]
 for line in (folder/'events.jsonl').read_text().splitlines():
  try:events.append(json.loads(line))
  except ValueError:pass
 items={e['item']['id']:e['item'] for e in events if isinstance(e.get('item'),dict) and 'id' in e['item']}
 commands=[v for v in items.values() if v.get('type')=='command_execution'];authoring=[]
 for item in commands:
  if 'identities' not in item.get('command','') or item.get('exit_code') in (0,None):continue
  for line in item.get('aggregated_output','').splitlines():
   try:observed=json.loads(line)
   except ValueError:continue
   diagnostic=observed.get('diagnostic')
   if isinstance(diagnostic,dict):authoring.append(dict(command=item['command'],exit_code=item['exit_code'],diagnostic=diagnostic))
 assert len(commands)==row['command_tool_items']
 audits.append(dict(index=a['index'],case_labels_recomputed=True,required_source_signatures_preserved=True,native_observations=len(cases),candidate_changes_preserved=changed,nonzero_command_exits=sum(v.get('exit_code') not in (0,None) for v in commands),authoring_diagnostic_errors=authoring))
assert not list(P.rglob('auth.json'))
summary=dict(date='2026-11-24',prior_entries=len(old['entries']),prior_pins=len(old['pins']),all_prior_entries_pins_unchanged=True,registered_code_pins=len(reg['code']),all_registered_code_material_hashes_match=True,compiler_sha256=s.sha(s.B.COMPILER),audits=audits,credentials_present_in_evidence=False,limits='Recomputes labels from captured native outputs and Python oracles; does not rerun native execution or models.')
(P/'postchecks.json').write_text(json.dumps(summary,indent=2)+'\n');print('All captured outputs/labels, registered inputs and prior pins agree; candidate changes retained.')
