import importlib.util,json
from pathlib import Path
root=Path('/home/luis/dev/lobo-ar/lobo-lang')
spec=importlib.util.spec_from_file_location('learning',root/'benchmarks/matched-study/learning.py');s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
primary=json.loads((root/'benchmarks/results/2026-11-24-learning-study.json').read_text());replays=[]
for row in primary['trials']:
 a=row['assignment']
 if a['stage']!='format':continue
 folder=s.PRIVATE/'run'/('trial-%02d'%a['index']);answer=s.contract.decode((folder/'last-message.txt').read_text())
 if not isinstance(answer,dict):continue
 for key in ('witness','counterexample'):
  args=answer.get(key)
  if not s.C.BASE.valid_input(a['family'],args):continue
  for language in s.C.BASE.LANGUAGES:
   for mutant in ([False,True] if key=='counterexample' else [False]):
    source=s.PRIVATE/'validation-registered'/(a['family']+'-'+language+('-mutant' if mutant else ''));actual=s.C.BASE.execute(source,language,a['family'],args,source/'program');expected=s.corpus.format_oracle(a['family'],args,mutant)
    record=dict(assignment=a,input_role=key,input=args,language=language,mutant=mutant,actual=actual,expected=expected,ok=s.C.BASE.equal(actual,expected),source_folder=str(source));replays.append(record)
    if not record['ok']:raise ValueError(record)
p=root/'benchmarks/results/2026-11-24-learning-observed-input-replays.json';s.C.BASE.write_json(p,dict(date='2026-11-24',primary_sha256=s.C.sha(root/'benchmarks/results/2026-11-24-learning-study.json'),scope='All admissible submitted format-stage witness/counterexample inputs, including unsuccessful attempts; no property projection or score changes',replays=replays,all_agree=all(x['ok'] for x in replays)));print('Concrete observed-input replays:',len(replays),'all agree')
