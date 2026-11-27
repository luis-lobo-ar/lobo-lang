"""Close and verify a local raw archive after all run and check writers have stopped."""
import argparse,hashlib,json,tarfile
from pathlib import Path
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--raw',type=Path,required=True);args=parser.parse_args();p=args.raw.resolve()
 assert (p/'run/environment-after.json').exists() and (p/'postchecks.json').exists()
 assert not list(p.rglob('auth.json')) and not any(f.is_symlink() for f in p.rglob('*'))
 sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
 files=sorted(f for f in p.rglob('*') if f.is_file());records=[dict(path=str(f.relative_to(p)),sha256=sha(f),bytes=f.stat().st_size) for f in files]
 archive=p.with_name(p.name+'-raw.tar.gz');manifest=p.with_name(p.name+'-raw-manifest.json');assert not archive.exists() and not manifest.exists()
 with tarfile.open(archive,'w:gz') as t:
  for f in files:t.add(f,arcname=str(f.relative_to(p)),recursive=False)
 with tarfile.open(archive,'r:gz') as t:
  assert len(t.getmembers())==len(records)
  for r in records:
   b=t.extractfile(r['path']).read();assert len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256']
 value=dict(format='lobo-v3-raw/1',archive=str(archive),sha256=sha(archive),bytes=archive.stat().st_size,members=len(records),all_member_hashes_verified=True,files=records,limits='Local ignored archive, not off-device backup. Authentication home was outside evidence and removed. Frozen compiler retained separately at registered path/hash. Final link/index checks retained outside closed directory.')
 manifest.write_text(json.dumps(value,indent=2)+'\n');print(json.dumps({k:v for k,v in value.items() if k!='files'}))
