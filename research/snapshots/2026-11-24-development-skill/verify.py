"""Exercise the development skill's native red/green workflow without an LLM trial."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler',type=Path,required=True)
    parser.add_argument('--skill',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();compiler=args.compiler.resolve();skill=args.skill.resolve();out=args.out.resolve()
    out.mkdir(parents=True,exist_ok=False);records=[]
    def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
    def save():
        (out/'results.json').write_text(json.dumps(dict(compiler=str(compiler),compiler_sha256=sha(compiler),skill=str(skill),skill_sha256=sha(skill),commands=records),indent=2)+'\n')
    def run(command):
        p=subprocess.run([str(x) for x in command],capture_output=True,text=True,timeout=40)
        row=dict(command=[str(x) for x in command],exit_code=p.returncode,stdout=p.stdout,stderr=p.stderr);records.append(row);save();return row
    def success(command):
        row=run(command);assert row['exit_code']==0,row;return row
    references=[]
    for target in re.findall(r'\]\(([^)#]+)',skill.read_text()):
        path=(skill.parent/target).resolve();assert path.is_file(),str(path)
        references.append(dict(target=target,resolved=str(path),sha256=sha(path)))
    (out/'references.json').write_text(json.dumps(references,indent=2)+'\n')
    helper=out/'helpers.lobo';source=out/'program.lobo'
    helper.write_text('module helpers; fn fee(value:I64)->I64 {value+8}\n')
    source.write_text('''module development_probe;
use helpers {fee};
record Item {quantity:I64,total:I64}
variant Outcome<T> {Ok(T),Err(I64)}
fn base(amount:I64)->Outcome<Item> {
  if amount<0 {Err(9)} else {Ok(Item {quantity:2,total:amount})}
}
fn adjust(amount:I64)->Outcome<Item> {
  let item=try base(amount);
  Ok(Item {...item,total:helpers.fee(item.total)})
}
fn total(amount:I64)->I64 {
  match adjust(amount) {Ok(item)=>item.quantity*100+item.total,Err(code)=>-code}
}
''')
    success([compiler,'identities','init',helper]);success([compiler,'identities','init',source,'--source',helper])
    metadata=[Path(str(f)+'.ids.json') for f in (helper,source)];before={str(f):sha(f) for f in metadata}
    entry=json.loads(metadata[1].read_text())['identities']['total']
    success([compiler,'check',source,'--source',helper]);success([compiler,'compile',source,'--source',helper,'-o',out/'red'])
    red=success([out/'red','--entry',entry,'--arg','I64:0'])
    actual=json.loads(red['stdout'])['value'];assert actual==dict(type='I64',value='208'),actual
    assert actual!=dict(type='I64',value='207')
    (out/'tdd-red.json').write_text(json.dumps(dict(requirement='Nonnegative totals receive fee7; quantity stays2. Negative inputs propagate Err9.',input=0,expected=207,actual=208,behavioral_test_passed=False,intentional_defect_detected=True),indent=2)+'\n')
    (out/'helpers-red.lobo.txt').write_text(helper.read_text());helper.write_text('module helpers; fn fee(value:I64)->I64 {value+7}\n')
    success([compiler,'check',source,'--source',helper]);success([compiler,'compile',source,'--source',helper,'-o',out/'green'])
    for value,expected in ((0,207),(5,212),(10,217),(-1,-9),(-100,-9)):
        row=success([out/'green','--entry',entry,'--arg','I64:'+str(value)])
        result=json.loads(row['stdout']);assert result.get('status')=='ok' and result.get('value')==dict(type='I64',value=str(expected)),row
    assert before=={str(f):sha(f) for f in metadata}
    summary=dict(skill_references=len(references),all_references_exist=True,intentional_behavioral_failure_detected=True,native_green_cases=5,stable_metadata_unchanged=True,explicit_dependency_compiled=True,compiler_sha256=sha(compiler),agent_trials=0,limits='Deterministic instruction/tool smoke check; no measured agent compliance, productivity or token benefit.')
    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))


if __name__=='__main__':main()
