import json,subprocess
from pathlib import Path
compiler='/home/luis/dev/lobo-ar/lobo-lang/benchmarks/private-evidence/matched-study-20261123/freeze/lobo'
sources=json.loads(Path('BUNDLE.json').read_text())
for name in ('policy.lobo','logic.lobo'):
 command=[compiler,'identities','sync',name]
 for source in sources:
  if source not in ('app.lobo',name) and not(name=='policy.lobo' and source=='logic.lobo'):command+=['--source',source]
 p=subprocess.run(command)
 if p.returncode:raise SystemExit(p.returncode)
