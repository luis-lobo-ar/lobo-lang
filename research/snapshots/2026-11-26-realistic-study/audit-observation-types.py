"""Post-registration type-sensitivity audit; never rewrites primary scores or input data."""
from pathlib import Path
import json

def strict(expected,actual):
 if type(expected) is not type(actual):return False
 if isinstance(expected,dict):return expected.keys()==actual.keys() and all(strict(v,actual[k]) for k,v in expected.items())
 if isinstance(expected,list):return len(expected)==len(actual) and all(strict(x,y) for x,y in zip(expected,actual))
 return expected==actual

if __name__=='__main__':
 assert 1==True and not strict(1,True)
 assert not strict({'a':[0]},{'a':[False]}) and strict({'a':[0]},{'a':[0]})
 root=Path(__file__).resolve().parents[3];path=root/'benchmarks/results/2026-11-26-realistic-study.json';result=json.loads(path.read_text());rows=[]
 for trial in result['trials']:
  for i,case in enumerate(trial['grading']['cases']):
   typed=strict(case['expected'],case['actual']);rows.append(dict(index=trial['assignment']['index'],scenario=i,primary=case['ok'],strict_type_match=typed))
 output=dict(label='Post-registration sensitivity audit, not primary score change',reason='Python equality can equate booleans with integers; additionally verify exact nested JSON types.',property_checks=3,observed_scenarios=len(rows),changed_case_outcomes=sum(r['primary']!=r['strict_type_match'] for r in rows),rows=rows)
 (root/'benchmarks/results/2026-11-26-realistic-type-audit.json').write_text(json.dumps(output,indent=2)+'\n')
 print(json.dumps({k:v for k,v in output.items() if k!='rows'}))
