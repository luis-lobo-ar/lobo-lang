from pathlib import Path
import hashlib,importlib.util,json,re
R=Path.cwd();P=R/'benchmarks/private-evidence/skill-revision-20261126';assert (P/'run/environment-after.json').exists()
spec=importlib.util.spec_from_file_location('revision_post',R/'benchmarks/matched-study/skill_revision.py');s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
reg=json.loads((P/'registration/manifest.json').read_text());prior=json.loads((P/'prior-paper-pins.json').read_text());index=json.loads((R/'research/local-evidence.json').read_text());assert index['entries']==prior['entries']
assert all(s.sha(R/f['path'])==f['sha256'] for f in prior['pins']);assert all(s.sha(R/f)==h for f,h in reg['code'].items());assert s.hashed(s.D.materials())==reg['material_hashes']
rows=json.loads((P/'run/results.json').read_text());checks=[]
def signature(text):
 m=re.search(r'fn\s+run\s*\(([^)]*)\)\s*->\s*([^\{]+)\{',text)
 if not m:return None
 return dict(parameters=[re.sub(r'\s+','',x.split(':',1)[1]) for x in m[1].split(',')],output=re.sub(r'\s+','',m[2]))
for row in rows:
 a=row['assignment'];folder=P/'run'/('trial-%02d'%a['index']);initial=(folder/'initial/logic.lobo').read_text();final=(folder/'heldout/source/logic.lobo').read_text();g=row['grading']
 for c in g['cases']:assert c['expected']==s.C.oracle(a['family'],c['input']) and c['ok']==s.B.equal(c['expected'],c['actual'])
 assert g['passed']==sum(c['ok'] for c in g['cases'])
 checks.append(dict(index=a['index'],required_source_signature_matches=signature(initial)==signature(final),native_observations=len(g['cases']),accepted=row['accepted'],candidate_native_passes=g['accepted'],status=row['status']))
assert not list(P.rglob('auth.json'))
value=dict(date='2026-11-26',prior_entries=len(prior['entries']),prior_pins=len(prior['pins']),all_prior_entries_pins_unchanged=True,registered_code_pins=len(reg['code']),all_registered_code_materials_match=True,compiler_sha256=s.sha(s.B.COMPILER),checks=checks,credential_files_in_evidence=False,limits='Read-only native-label and source-signature audit; original grades and unknown usage unchanged.')
(P/'postchecks.json').write_text(json.dumps(value,indent=2)+'\n');print(json.dumps(dict(attempts=len(rows),native_observations=sum(x['native_observations'] for x in checks),all_signature_matches=all(x['required_source_signature_matches'] for x in checks))))
