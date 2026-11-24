"""Close the compact pilot's local raw evidence and verify every compressed member."""
from pathlib import Path
import hashlib, json, tarfile
p=Path(__file__).resolve().parent
root=p.parents[2]
assert (p/'run/environment-after.json').is_file()
reg=json.loads((p/'registration/manifest.json').read_text())
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
assert all(sha(root/f)==h for f,h in reg['code'].items())
compiler=root/'benchmarks/private-evidence/matched-study-20261123/freeze/lobo'
assert sha(compiler)==reg['compiler_sha256']
prior=root/'benchmarks/results/2026-11-24-learning-verification.json'
assert json.loads(prior.read_text())['checks']['cargo']==dict(suites=59,passed=312,failed=0,ignored=0)
assert json.loads((p/'development/final-audit.json').read_text())['all_old_entries_and_pins_match']
files=sorted(f for f in p.rglob('*') if f.is_file())
assert not any(f.is_symlink() for f in p.rglob('*'))
records=[dict(path=str(f.relative_to(p)),sha256=sha(f),bytes=f.stat().st_size) for f in files]
archive=p.with_name(p.name+'-raw.tar.gz')
manifest=p.with_name(p.name+'-raw-manifest.json')
assert not archive.exists() and not manifest.exists()
with tarfile.open(archive,'w:gz') as tf:
 for f in files:tf.add(f,arcname=str(f.relative_to(p)),recursive=False)
with tarfile.open(archive,'r:gz') as tf:
 assert len(tf.getmembers())==len(records)
 for r in records:
  data=tf.extractfile(r['path']).read()
  assert len(data)==r['bytes'] and hashlib.sha256(data).hexdigest()==r['sha256']
value=dict(format='lobo-compact-raw-preservation/1',archive=str(archive),archive_sha256=sha(archive),archive_bytes=archive.stat().st_size,members=len(records),all_member_hashes_verified=True,files=records,limits='Local ignored archive, not an off-device backup. Frozen compiler is retained separately at its registered path/hash. Final public-index/link checks are retained outside this closed raw directory. Reused earlier compiler suite; no new cargo run.')
manifest.write_text(json.dumps(value,indent=2)+'\n')
print(json.dumps({k:v for k,v in value.items() if k!='files'}))
