"""Retain all private study artifacts after writers finish; verify every archived byte."""
import hashlib,json,tarfile
from pathlib import Path
p=Path(__file__).resolve().parent
assert (p/'run/environment-after.json').is_file()
assert (p/'development/cargo-exit.txt').read_text().strip()=='0'
sha=lambda b:hashlib.sha256(b).hexdigest()
files=[]
for source in sorted(p.rglob('*')):
 assert not source.is_symlink(),source
 if source.is_file():files.append(dict(path=source.relative_to(p).as_posix(),sha256=sha(source.read_bytes()),bytes=source.stat().st_size))
archive=p.parent/(p.name+'-raw.tar.gz');manifest=p.parent/(p.name+'-raw-manifest.json')
assert not archive.exists() and not manifest.exists()
with tarfile.open(archive,'w:gz') as t:
 for f in files:t.add(p/f['path'],arcname=f['path'],recursive=False)
with tarfile.open(archive,'r:gz') as t:
 members=t.getmembers();assert len(members)==len(files)
 for member,expected in zip(members,files):
  assert member.isfile() and member.name==expected['path']
  data=t.extractfile(member).read();assert sha(data)==expected['sha256'] and len(data)==expected['bytes']
receipt=dict(archive=str(archive),archive_sha256=sha(archive.read_bytes()),archive_bytes=archive.stat().st_size,members=len(files),all_member_hashes_verified=True,files=files,limits='Retained prior frozen compiler remains at the path/hash in registration; it is not duplicated in this raw archive. Final public-index/link checks are retained separately after archive closure.')
manifest.write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:v for k,v in receipt.items() if k!='files'}))
