#!/usr/bin/env python3
"""Inventory tracked code and lexical Lobo DX candidates; never rewrite source or grade agents."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT=Path(__file__).resolve().parents[1]
CODE={'.rs','.lobo','.c','.h','.cjs','.mjs','.py','.sh','.tla','.test','.cfg'}


def mask_lobo(source):
    return re.sub(r'//[^\r\n]*|"(?:[^"\\]|\\.)*"',lambda m:re.sub(r'[^\r\n]',' ',m[0]),source)


def closing(text,start):
    depth=0
    for i in range(start,len(text)):
        if text[i]=='{':depth+=1
        if text[i]=='}':
            depth-=1
            if depth==0:return i
    raise ValueError('unbalanced Lobo braces')


def lobo_functions(source):
    masked=mask_lobo(source);result=[]
    for match in re.finditer(r'\b(?:fn|function)\s+(\w+)',masked):
        start=match.end();actions=[]
        while True:
            brace=masked.index('{',start);end=closing(masked,brace)
            clause=re.search(r'(actions|requires|ensures)\s*$',masked[start:brace])
            if not clause:break
            if clause[1]=='actions':actions=sorted(re.findall(r'\b[a-z_]\w*\.\w+',masked[brace:end]))
            start=end+1
        body=masked[brace+1:end];words=Counter(re.findall(r'\b\w+\b',body))
        bindings=re.findall(r'\blet\s+(\w+)',body)
        result.append({'name':match[1],'line':masked.count('\n',0,match.start())+1,'lines':masked.count('\n',match.start(),end)+1,
          'actions':actions,'if_tokens':words['if'],'match_tokens':words['match'],'let_tokens':words['let'],
          'loop_tokens':words['while']+words['fold'],
          'forwarded_fields':len(re.findall(r'\b(\w+)\s*:\s*\w+\s*\.\s*\1\b(?!\s*[+*/%<>=!&|.-])',body)),
          'unused_binding_candidates':[name for name in bindings if words[name]==1],
          'cleanup_outcome_candidate':bool(re.search(r'let\s+outcome\s*=.*?;\s*let\s+\w+\s*=\s*try\s+[\w:.]*(?:close|finalize)\w*\(',body,re.S)),
          'ok_err_mapping_candidate':bool(re.search(r'Ok\(\w+\)\s*=>\s*Ok\(\w+\).*Err\(',body,re.S))})
    return result


def category(name):
    if name.startswith(('vendor/','tests/upstream/')):return 'upstream'
    if name.startswith(('tests/fixtures/','benchmarks/fixtures/','benchmarks/language-comparison/','examples/v1/')) or name in ['examples/mvc.lobo','examples/transfer.lobo']:return 'frozen_or_comparison'
    if name.startswith('tests/'):return 'tests'
    if name.endswith('.lobo'):return 'maintained_lobo'
    return 'implementation_and_tools'


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();args.out.mkdir(parents=True,exist_ok=False)
    names=subprocess.run(['git','ls-files','-z'],cwd=ROOT,capture_output=True,check=True).stdout.decode().split('\0')
    files=[];functions=[]
    for name in sorted(set(names)-{''}):
        path=ROOT/name
        if path.suffix not in CODE:continue
        data=path.read_bytes();text=data.decode('utf-8');group=category(name)
        row={'path':name,'category':group,'bytes':len(data),'lines':len(text.splitlines()),'sha256':hashlib.sha256(data).hexdigest(),'lines_over_160':sum(len(line)>160 for line in text.splitlines())}
        if path.suffix=='.lobo' and group!='upstream':
            found=lobo_functions(text);functions.extend({'path':name,'category':group,**f} for f in found);row['functions']=len(found)
        files.append(row)
    counts={c:{'files':sum(f['category']==c for f in files),'lines':sum(f['lines'] for f in files if f['category']==c)} for c in sorted(set(f['category'] for f in files))}
    maintained=[f for f in functions if f['category']=='maintained_lobo']
    result={'format':'lobo-dx-inventory/0.1','method':'lexical candidates, not AST complexity, bugs, productivity or token counts','head':subprocess.run(['git','rev-parse','HEAD'],cwd=ROOT,capture_output=True,text=True,check=True).stdout.strip(),'categories':counts,'maintained_lobo':{'files':sum(f['category']=='maintained_lobo' for f in files),'functions':len(maintained),'forwarded_field_candidates':sum(f['forwarded_fields'] for f in maintained),'unused_binding_candidates':sum(len(f['unused_binding_candidates']) for f in maintained),'cleanup_outcome_functions':sum(f['cleanup_outcome_candidate'] for f in maintained),'ok_err_mapping_functions':sum(f['ok_err_mapping_candidate'] for f in maintained),'functions_with_actions':sum(bool(f['actions']) for f in maintained),'max_actions':max((len(f['actions']) for f in maintained),default=0)},'files':files,'lobo_functions':functions}
    (args.out/'inventory.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ['files','lobo_functions']}))

if __name__=='__main__':main()
