#!/usr/bin/env python3
"""Matched-language pilot: deterministic tasks, independent grading and retained attempts."""
import argparse, ast, hashlib, itertools, json, os, random, resource, selectors, shutil, signal, statistics, subprocess, tempfile, threading, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
PRIVATE=ROOT/'benchmarks/private-evidence/matched-study-20261123'
COMPILER=PRIVATE/'freeze/lobo'
LANGUAGES=('lobo','rust','python')
PHASES=('construction','comprehension','change')
FAMILIES=('ledger','scheduler','frames')
MODEL='gpt-6.1-sol'
DEADLINE=120
TOOL_LIMIT=16
CHECK_LIMIT=6

def digest(data):return hashlib.sha256(data).hexdigest()
def write_json(path,value):
    path=Path(path);tmp=path.with_suffix(path.suffix+'.tmp');tmp.write_text(json.dumps(value,indent=2)+'\n');tmp.replace(path)
def usage(events):
    found=[e['usage'] for e in events if e.get('type')=='turn.completed' and isinstance(e.get('usage'),dict)]
    return found[-1] if found else None

def schedule():
    blocks=list(itertools.product(range(2),range(3),range(3)));random.Random(20261123).shuffle(blocks);rows=[]
    for repeat,f,p in blocks:
        shift=(repeat+f+p)%3;order=list(LANGUAGES[shift:]+LANGUAGES[:shift])
        if repeat:order.reverse()
        for language in order:rows.append({'index':len(rows),'repeat':repeat,'family':FAMILIES[f],'phase':PHASES[p],'language':language})
    return rows

def oracle(family,args,changed=False):
    if family=='ledger':
        balance=args['initial'];peak=balance;rejected=0;locked=False;history=[];audit=[]
        for kind,amount in args['events']:
            if kind==2:locked=True
            elif kind==3:locked=False
            elif kind==4 and changed:
                if locked or not history:rejected+=1
                else:balance=history.pop()
            elif kind in (0,1):
                if not 0<=amount<=1000000 or (kind==1 and (locked or amount>balance)):rejected+=1
                else:history.append(balance);balance+=amount if kind==0 else -amount
            else:raise ValueError('invalid event')
            peak=max(peak,balance);audit.append(balance)
        return {'balance':balance,'rejected':rejected,'peak':peak,'audit':audit}
    if family=='scheduler':
        done=[];remaining=args['budget'];jobs=args['jobs']
        while True:
            choices=[j for j in jobs if j['id'] not in done and set(j['deps'])<=set(done) and j['cost']<=remaining]
            if not choices:break
            job=min(choices,key=lambda j:(-j['priority'],j['id']) if changed else (j['id'],))
            done.append(job['id']);remaining-=job['cost']
        return {'order':done,'spent':args['budget']-remaining,'blocked':len(jobs)-len(done)}
    data=bytes.fromhex(args['data']);frames=[];cursor=0
    while cursor<len(data):
        start=cursor;prefix=data[cursor];cursor+=1
        if changed and prefix==255:continue
        if changed and prefix==254:
            if len(data)-cursor<2:return {'frames':frames,'consumed':start,'error':1}
            length=int.from_bytes(data[cursor:cursor+2],'little');cursor+=2
        else:length=prefix
        if length>args['limit']:return {'frames':frames,'consumed':start,'error':3}
        if len(data)-cursor<length+1:return {'frames':frames,'consumed':start,'error':1}
        payload=data[cursor:cursor+length];check=data[cursor+length]
        if sum(payload)%256!=check:return {'frames':frames,'consumed':start,'error':2}
        frames.append(payload.hex());cursor+=length+1
    return {'frames':frames,'consumed':cursor,'error':0}

def cases(family,changed=False,hidden=False):
    rng=random.Random(71913 if hidden else 151);out=[]
    if family=='ledger':
        out=[{'initial':0,'events':[]},{'initial':4,'events':[(1,4),(1,0),(1,1)]},{'initial':0,'events':[(0,7),(2,0),(1,1),(0,2),(3,0),(1,8)]},{'initial':3,'events':[(0,-1),(1,-1),(0,1000000)]}]
        if changed:out+=[{'initial':0,'events':[(4,0),(0,5),(1,2),(4,0),(2,0),(4,0),(3,0),(4,0),(4,0)]}]
        for _ in range(90 if hidden else 3):
            events=[]
            for i in range(rng.randrange(1,25 if hidden else 9)):
                kind=rng.randrange(5 if changed else 4);events.append((kind,rng.choice([-2,0,1,2,7,31,1000000]) if kind<2 else 0))
            out.append({'initial':rng.randrange(0,20),'events':events})
    elif family=='scheduler':
        def job(i,c,r,p=0):return dict(id=i,cost=c,deps=r,**({'priority':p} if changed else {}))
        out=[{'jobs':[],'budget':0},{'jobs':[job(0,0,[]),job(1,1,[0])],'budget':1},{'jobs':[job(2,1,[]),job(0,4,[1]),job(1,2,[])],'budget':3},{'jobs':[job(0,1,[1]),job(1,1,[0])],'budget':10}]
        if changed:out+=[{'jobs':[job(0,1,[],0),job(1,1,[],5)],'budget':1}]
        for _ in range(90 if hidden else 3):
            n=rng.randrange(1,10 if hidden else 5);jobs=[]
            for i in range(n):jobs.append(job(i,rng.randrange(0,10),[rng.randrange(n) for _ in range(rng.randrange(4))],rng.randrange(-2,5)))
            rng.shuffle(jobs);out.append({'jobs':jobs,'budget':rng.randrange(0,30)})
    else:
        def frame(value):return bytes([len(value)])+value+bytes([sum(value)%256])
        out=[{'data':'','limit':0},{'data':'0000','limit':0},{'data':'02010102','limit':2},{'data':'02010103','limit':2},{'data':'02','limit':1},{'data':'0201','limit':2}]
        if changed:out+=[{'data':'fffe0200010102','limit':2},{'data':'fe01','limit':2},{'data':'ffff','limit':0}]
        for _ in range(90 if hidden else 3):
            limit=rng.randrange(0,9);blob=b''
            for i in range(rng.randrange(1,7)):
                value=bytes(rng.randrange(256) for _ in range(rng.randrange(0,10)))
                encoded=frame(value)
                if changed and rng.randrange(3)==0:encoded=b'\xfe'+len(value).to_bytes(2,'little')+encoded[1:]
                if changed and rng.randrange(4)==0:encoded=b'\xff'+encoded
                blob+=encoded
            if rng.randrange(4)==0:blob=blob[:-1]
            elif blob and rng.randrange(4)==0:blob=blob[:-1]+bytes([(blob[-1]+1)%256])
            out.append({'data':blob.hex(),'limit':limit})
    return out

# Templates contain only pure algorithms. The CLI adapters and data models are trusted fixtures.
LOBO_UTIL='''module util;
record Reverse<T> { pending:List<T>, output:List<T> }
fn reverse<T>(items:List<T>)->List<T> {
 let final=fold(i in List::length(items), state:Reverse<T> =Reverse {pending:items,output:List::empty<T>()}) {
  Reverse {pending:List::tail(state.pending),output:List::prepend(List::head(state.pending),state.output)}
 };
 final.output
}
identities {module:"study.util",Reverse:"study.util.Reverse",reverse:"study.util.reverse"}
'''

def models(family,language,changed=False):
    if language=='lobo':
        text={'ledger':'variant Event { Credit(I64), Debit(I64), Lock, Unlock'+(', Undo' if changed else '')+' }\nrecord Summary { balance:I64,rejected:I64,peak:I64,audit:List<I64> }',
        'scheduler':'record Job { id:I64,cost:I64,deps:List<I64>'+(',priority:I64' if changed else '')+' }\nrecord Summary {order:List<I64>,spent:I64,blocked:I64}',
        'frames':'record Summary {frames:List<Bytes>,consumed:I64,error:I64}'}[family]
        names=['Event','Summary'] if family=='ledger' else ['Job','Summary'] if family=='scheduler' else ['Summary']
        return 'module model;\n'+text+'\nidentities {module:"study.'+family+'.model",'+','.join(n+':"study.'+family+'.'+n+'"' for n in names)+'}\n'
    if language=='rust':
        return {'ledger':'#[derive(Clone)] pub enum Event {Credit(i64),Debit(i64),Lock,Unlock'+(',Undo' if changed else '')+'}\npub struct Summary {pub balance:i64,pub rejected:i64,pub peak:i64,pub audit:Vec<i64>}\n',
        'scheduler':'#[derive(Clone)] pub struct Job {pub id:i64,pub cost:i64,pub deps:Vec<i64>'+(',pub priority:i64' if changed else '')+'}\npub struct Summary {pub order:Vec<i64>,pub spent:i64,pub blocked:i64}\n',
        'frames':'pub struct Summary {pub frames:Vec<Vec<u8>>,pub consumed:i64,pub error:i64}\n'}[family]
    return 'from dataclasses import dataclass\n'+{'ledger':'from enum import IntEnum\nclass Kind(IntEnum):\n CREDIT=0\n DEBIT=1\n LOCK=2\n UNLOCK=3\n'+(' UNDO=4\n' if changed else '')+'@dataclass(frozen=True)\nclass Event:\n kind:Kind\n amount:int=0\n@dataclass\nclass Summary:\n balance:int\n rejected:int\n peak:int\n audit:list[int]\n',
    'scheduler':'@dataclass(frozen=True)\nclass Job:\n id:int\n cost:int\n deps:list[int]\n'+(' priority:int=0\n' if changed else '')+'@dataclass\nclass Summary:\n order:list[int]\n spent:int\n blocked:int\n',
    'frames':'@dataclass\nclass Summary:\n frames:list[bytes]\n consumed:int\n error:int\n'}[family]

def signature(family,language):
    return {'lobo':{'ledger':'events:List<Event>,initial:I64','scheduler':'jobs:List<Job>,budget:I64','frames':'data:Bytes,limit:I64'},'rust':{'ledger':'events:&[Event],initial:i64','scheduler':'jobs:&[Job],budget:i64','frames':'data:&[u8],limit:i64'},'python':{'ledger':'events, initial','scheduler':'jobs, budget','frames':'data, limit'}}[language][family]

def ids_for(family,source):
    import re
    names=re.findall(r'\b(?:fn|record|variant)\s+(\w+)',source)
    ids={'module':'study.'+family+'.logic','model.Summary':'study.'+family+'.Summary','util.reverse':'study.util.reverse'}
    for name in ['Event'] if family=='ledger' else ['Job'] if family=='scheduler' else []:ids['model.'+name]='study.'+family+'.'+name
    ids.update({name:'study.'+family+'.'+name for name in names})
    return {'format':'lobo-identities/0.1','identities':ids}

REFERENCE_LOBO={}
REFERENCE_RUST={}
REFERENCE_PYTHON={}

def reference_lobo(family,changed=False):
    imports='use model {Summary'+(',Event' if family=='ledger' else ',Job' if family=='scheduler' else '')+'};\nuse util {reverse};\n'
    if family=='ledger':
        extra=',history:List<I64>' if changed else ''
        push=',history:List::prepend(state.balance,state.history)' if changed else ''
        undo=',Undo=> if state.locked || List::length(state.history)==0 {State {...state,rejected:state.rejected+1}} else {State {...state,balance:List::head(state.history),history:List::tail(state.history)}}' if changed else ''
        code='''record State {balance:I64,rejected:I64,peak:I64,locked:Bool,audit:List<I64>EXTRA}
record Cursor {pending:List<Event>,state:State}
fn valid_amount(value:I64)->Bool {value>=0 && value<=1000000}
fn step(state:State,event:Event)->State {
 let next=match event {
  Credit(amount)=> if valid_amount(amount) {State {...state,balance:state.balance+amount PUSH}} else {State {...state,rejected:state.rejected+1}},
  Debit(amount)=> if !valid_amount(amount) || state.locked || amount>state.balance {State {...state,rejected:state.rejected+1}} else {State {...state,balance:state.balance-amount PUSH}},
  Lock=> State {...state,locked:true}, Unlock=> State {...state,locked:false}UNDO
 };
 State {...next,peak:if next.balance>state.peak {next.balance} else {state.peak},audit:List::prepend(next.balance,state.audit)}
}
fn run(events:List<Event>,initial:I64)->Summary {
 let final=fold(i in List::length(events),cursor:Cursor=Cursor {pending:events,state:State {balance:initial,rejected:0,peak:initial,locked:false,audit:List::empty<I64>()INIT}}) {
  Cursor {pending:List::tail(cursor.pending),state:step(cursor.state,List::head(cursor.pending))}
 };
 Summary {balance:final.state.balance,rejected:final.state.rejected,peak:final.state.peak,audit:reverse(final.state.audit)}
}
'''.replace('EXTRA',extra).replace('PUSH',push).replace('UNDO',undo).replace('INIT',',history:List::empty<I64>()' if changed else '')
    elif family=='scheduler':
        better='item.priority>old.priority || (item.priority==old.priority && item.id<old.id)' if changed else 'item.id<old.id'
        code='''variant Pick {None,Some(Job)}
record Scan {pending:List<I64>,yes:Bool}
record Search {pending:List<Job>,best:Pick}
record State {done:List<I64>,remaining:I64,stopped:Bool}
fn contains(items:List<I64>,value:I64)->Bool {
 let final=fold(i in List::length(items),cursor:Scan=Scan {pending:items,yes:false}) {Scan {pending:List::tail(cursor.pending),yes:cursor.yes || List::head(cursor.pending)==value}};
 final.yes
}
fn ready(job:Job,done:List<I64>)->Bool {
 let final=fold(i in List::length(job.deps),cursor:Scan=Scan {pending:job.deps,yes:true}) {Scan {pending:List::tail(cursor.pending),yes:cursor.yes && contains(done,List::head(cursor.pending))}};
 final.yes
}
fn fits(cost:I64,remaining:I64)->Bool {cost<=remaining}
fn choose(jobs:List<Job>,done:List<I64>,remaining:I64)->Pick {
 let final=fold(i in List::length(jobs),cursor:Search=Search {pending:jobs,best:None}) {
  let item=List::head(cursor.pending);
  let best=if contains(done,item.id) || !ready(item,done) || !fits(item.cost,remaining) {cursor.best} else {
   match (cursor.best) {None=> Pick::Some(item),Some(old)=> if BETTER {Pick::Some(item)} else {cursor.best}}
  };
  Search {pending:List::tail(cursor.pending),best:best}
 };
 final.best
}
fn run(jobs:List<Job>,budget:I64)->Summary {
 let final=while(state:State=State {done:List::empty<I64>(),remaining:budget,stopped:false};!state.stopped) {
  match choose(jobs,state.done,state.remaining) {None=> State {...state,stopped:true},Some(job)=> State {...state,done:List::prepend(job.id,state.done),remaining:state.remaining-job.cost}}
 };
 Summary {order:reverse(final.done),spent:budget-final.remaining,blocked:List::length(jobs)-List::length(final.done)}
}
'''.replace('BETTER',better)
    else:
        header='''let prefix=Bytes::at(data,start);
 if prefix==255 {Header {length:-1,payload:start+1,error:0}} else {if prefix==254 {
  if Bytes::length(data)-start<3 {Header {length:0,payload:start,error:1}} else {Header {length:Bytes::at(data,start+1)+256*Bytes::at(data,start+2),payload:start+3,error:0}}
 } else {Header {length:prefix,payload:start+1,error:0}}}''' if changed else 'Header {length:Bytes::at(data,start),payload:start+1,error:0}'
        padding='if header.length<0 {State {...state,cursor:header.payload}} else {' if changed else ''
        code='''record Header {length:I64,payload:I64,error:I64}
record State {frames:List<Bytes>,cursor:I64,error:I64}
fn read_header(data:Bytes,start:I64)->Header {HEADER}
fn fits(length:I64,limit:I64)->Bool {length<=limit}
fn checksum(data:Bytes)->I64 {fold(i in Bytes::length(data),total:I64=0) {(total+Bytes::at(data,i))%256}}
fn run(data:Bytes,limit:I64)->Summary {
 let final=while(state:State=State {frames:List::empty<Bytes>(),cursor:0,error:0};state.cursor<Bytes::length(data) && state.error==0) {
  let header=read_header(data,state.cursor);
  if header.error!=0 {State {...state,error:header.error}} else {PADDINGif !fits(header.length,limit) {State {...state,error:3}} else {
   if Bytes::length(data)-header.payload<header.length+1 {State {...state,error:1}} else {
    let payload=Bytes::slice(data,header.payload,header.length);
    if checksum(payload)!=Bytes::at(data,header.payload+header.length) {State {...state,error:2}} else {State {...state,frames:List::prepend(payload,state.frames),cursor:header.payload+header.length+1}}
   }
  }}CLOSE_PADDING
 };
 Summary {frames:reverse(final.frames),consumed:final.cursor,error:final.error}
}
'''.replace('HEADER',header).replace('CLOSE_PADDING','}' if changed else '').replace('PADDING',padding)
    return 'module logic;\n'+imports+code

def reference_rust(family,changed=False):
    if family=='ledger':
        return '''use crate::model::{Event,Summary};
pub fn valid_amount(value:i64)->bool {value>=0 && value<=1000000}
struct State {balance:i64,rejected:i64,peak:i64,locked:bool,audit:Vec<i64> EXTRA}
fn step(mut state:State,event:&Event)->State {
 match *event {
  Event::Credit(amount)=> {if valid_amount(amount) {PUSH state.balance+=amount;} else {state.rejected+=1;}},
  Event::Debit(amount)=> {if !valid_amount(amount) || state.locked || amount>state.balance {state.rejected+=1;} else {PUSH state.balance-=amount;}},
  Event::Lock=>state.locked=true,Event::Unlock=>state.locked=false UNDO
 }
 state.peak=state.peak.max(state.balance);state.audit.push(state.balance);state
}
pub fn run(events:&[Event],initial:i64)->Summary {
 let mut state=State {balance:initial,rejected:0,peak:initial,locked:false,audit:Vec::new() INIT};
 for event in events {state=step(state,event);}
 Summary {balance:state.balance,rejected:state.rejected,peak:state.peak,audit:state.audit}
}
'''.replace('EXTRA',',history:Vec<i64>' if changed else '').replace('PUSH','state.history.push(state.balance);' if changed else '').replace('INIT',',history:Vec::new()' if changed else '').replace('UNDO',',Event::Undo=> {if state.locked || state.history.is_empty() {state.rejected+=1;} else {state.balance=state.history.pop().unwrap();}}' if changed else '')
    if family=='scheduler':
        return '''use crate::model::{Job,Summary};
fn ready(job:&Job,done:&[i64])->bool {job.deps.iter().all(|x|done.contains(x))}
fn fits(cost:i64,remaining:i64)->bool {cost<=remaining}
fn choose<'a>(jobs:&'a [Job],done:&[i64],remaining:i64)->Option<&'a Job> {
 let mut best:Option<&Job>=None;
 for item in jobs {if !done.contains(&item.id) && ready(item,done) && fits(item.cost,remaining) {
  if best.map_or(true,|old| BETTER) {best=Some(item);}
 }} best
}
pub fn run(jobs:&[Job],budget:i64)->Summary {
 let mut done=Vec::new();let mut remaining=budget;
 while let Some(job)=choose(jobs,&done,remaining) {done.push(job.id);remaining-=job.cost;}
 Summary {blocked:jobs.len() as i64-done.len() as i64,spent:budget-remaining,order:done}
}
'''.replace('BETTER','item.priority>old.priority || (item.priority==old.priority && item.id<old.id)' if changed else 'item.id<old.id')
    return '''use crate::model::Summary;
struct Header {length:i64,payload:usize,error:i64}
fn read_header(data:&[u8],start:usize)->Header {HEADER}
fn fits(length:i64,limit:i64)->bool {length<=limit}
fn checksum(data:&[u8])->u8 {data.iter().fold(0u8,|sum,b|sum.wrapping_add(*b))}
pub fn run(data:&[u8],limit:i64)->Summary {
 let mut frames=Vec::new();let mut cursor=0;let mut error=0;
 while cursor<data.len() && error==0 {
  let header=read_header(data,cursor);
  if header.error!=0 {error=header.error;} PADDINGelse if !fits(header.length,limit) {error=3;}
  else if data.len()-header.payload<(header.length+1) as usize {error=1;}
  else {let end=header.payload+header.length as usize;let payload=&data[header.payload..end];
   if checksum(payload)!=data[end] {error=2;} else {frames.push(payload.to_vec());cursor=end+1;}
  }
 }
 Summary {frames,consumed:cursor as i64,error}
}
'''.replace('HEADER','''let prefix=data[start] as i64;
 if prefix==255 {Header {length:-1,payload:start+1,error:0}} else if prefix==254 {
  if data.len()-start<3 {Header {length:0,payload:start,error:1}} else {Header {length:data[start+1] as i64+256*data[start+2] as i64,payload:start+3,error:0}}
 } else {Header {length:prefix,payload:start+1,error:0}}''' if changed else 'Header {length:data[start] as i64,payload:start+1,error:0}').replace('PADDING','else if header.length<0 {cursor=header.payload;} ' if changed else '')

def reference_python(family,changed=False):
    if family=='ledger':
        return '''from model import Event,Kind,Summary

def valid_amount(value):
 return 0<=value<=1000000

def step(state,event):
 balance,rejected,peak,locked,audit EXTRA=state
 if event.kind==Kind.LOCK:locked=True
 elif event.kind==Kind.UNLOCK:locked=False
UNDO elif event.kind in (Kind.CREDIT,Kind.DEBIT):
  amount=event.amount
  if not valid_amount(amount) or (event.kind==Kind.DEBIT and (locked or amount>balance)):rejected+=1
  else:
   PUSH
   balance+=amount if event.kind==Kind.CREDIT else -amount
 peak=max(peak,balance)
 return balance,rejected,peak,locked,audit+[balance] RETURN

def run(events,initial):
 state=(initial,0,initial,False,[] INIT)
 for event in events:state=step(state,event)
 return Summary(*state[:3],state[4])
'''.replace('EXTRA',',history' if changed else '').replace('PUSH','history=history+[balance]' if changed else 'pass').replace('RETURN',',history' if changed else '').replace('INIT',',[]' if changed else '').replace('UNDO',' elif event.kind==Kind.UNDO:\n  if locked or not history:rejected+=1\n  else:balance,history=history[-1],history[:-1]\n' if changed else '')
    if family=='scheduler':
        return '''from model import Job,Summary

def ready(job,done):
 return all(x in done for x in job.deps)

def fits(cost,remaining):
 return cost<=remaining

def choose(jobs,done,remaining):
 best=None
 for item in jobs:
  if item.id not in done and ready(item,done) and fits(item.cost,remaining):
   if best is None or BETTER:best=item
 return best

def run(jobs,budget):
 done=[];remaining=budget
 while True:
  job=choose(jobs,done,remaining)
  if job is None:break
  done.append(job.id);remaining-=job.cost
 return Summary(done,budget-remaining,len(jobs)-len(done))
'''.replace('BETTER','item.priority>best.priority or (item.priority==best.priority and item.id<best.id)' if changed else 'item.id<best.id')
    return '''from model import Summary

def read_header(data,start):
 HEADER

def fits(length,limit):
 return length<=limit

def checksum(data):
 return sum(data)%256

def run(data,limit):
 frames=[];cursor=0;error=0
 while cursor<len(data) and error==0:
  length,payload,header_error=read_header(data,cursor)
  if header_error:error=header_error
  PADDINGelif not fits(length,limit):error=3
  elif len(data)-payload<length+1:error=1
  else:
   value=data[payload:payload+length]
   if checksum(value)!=data[payload+length]:error=2
   else:frames.append(value);cursor=payload+length+1
 return Summary(frames,cursor,error)
'''.replace('HEADER','''prefix=data[start]
 if prefix==255:return -1,start+1,0
 if prefix==254:
  if len(data)-start<3:return 0,start,1
  return data[start+1]+256*data[start+2],start+3,0
 return prefix,start+1,0''' if changed else 'return data[start],start+1,0').replace('PADDING','elif length<0:cursor=payload\n  ' if changed else '')

def reference(family,language,changed=False):return {'lobo':reference_lobo,'rust':reference_rust,'python':reference_python}[language](family,changed)

def lobo_app(family):
    extra=',Event' if family=='ledger' else ',Job' if family=='scheduler' else ''
    sig=signature(family,'lobo');args={'ledger':'events,initial','scheduler':'jobs,budget','frames':'data,limit'}[family]
    varied={'ledger':'events,initial+i%3','scheduler':'jobs,budget+i%3','frames':'data,limit+i%3'}[family]
    total={'ledger':'value.balance+value.rejected+value.peak+sum_list(value.audit)','scheduler':'value.spent+value.blocked+sum_list(value.order)','frames':'value.consumed+value.error+sum_frames(value.frames)'}[family]
    text='''module app;
use model {SummaryEXTRA};
use logic {run};
record SumCursor {pending:List<I64>,total:I64}
record FrameCursor {pending:List<Bytes>,total:I64}
fn sum_list(items:List<I64>)->I64 {let final=fold(i in List::length(items),state:SumCursor=SumCursor {pending:items,total:0}) {SumCursor {pending:List::tail(state.pending),total:state.total+List::head(state.pending)}}; final.total}
fn sum_bytes(data:Bytes)->I64 {fold(i in Bytes::length(data),total:I64=0) {total+Bytes::at(data,i)}}
fn sum_frames(items:List<Bytes>)->I64 {let final=fold(i in List::length(items),state:FrameCursor=FrameCursor {pending:items,total:0}) {FrameCursor {pending:List::tail(state.pending),total:state.total+Bytes::length(List::head(state.pending))+sum_bytes(List::head(state.pending))}}; final.total}
fn digest_summary(value:Summary)->I64 {TOTAL}
fn entry(SIG)->Summary {logic.run(ARGS)}
fn bench(SIG,repeats:I64)->I64 {fold(i in repeats,total:I64=0) {total+digest_summary(logic.run(VARIED))}}
'''.replace('EXTRA',extra).replace('TOTAL',total).replace('SIG',sig).replace('ARGS',args).replace('VARIED',varied)
    ids={'module':'study.'+family+'.app','model.Summary':'study.'+family+'.Summary','logic.run':'study.'+family+'.run'}
    if extra:ids['model.'+extra[1:]]='study.'+family+'.'+extra[1:]
    for n in ['SumCursor','FrameCursor','sum_list','sum_bytes','sum_frames','digest_summary','entry','bench']:ids[n]='study.'+family+'.app.'+n
    ids['entry']='study.'+family+'.entry';ids['bench']='study.'+family+'.bench'
    return text,{'format':'lobo-identities/0.1','identities':ids}

def rust_app(family,changed=False):
    parse={'ledger':'''let initial:i64=a[1].parse().unwrap();let events:Vec<model::Event>=a[2].split(';').filter(|s|!s.is_empty()).map(|s| {let v:Vec<_>=s.split(':').collect();let n:i64=v[1].parse().unwrap();match v[0] {"0"=>model::Event::Credit(n),"1"=>model::Event::Debit(n),"2"=>model::Event::Lock,"3"=>model::Event::Unlock UNDO,_=>panic!("bad event")}}).collect();'''.replace('UNDO',',"4"=>model::Event::Undo' if changed else ''),
    'scheduler':'''let budget:i64=a[1].parse().unwrap();let jobs:Vec<model::Job>=a[2].split(';').filter(|s|!s.is_empty()).map(|s| {let v:Vec<_>=s.split(':').collect();model::Job {id:v[0].parse().unwrap(),cost:v[1].parse().unwrap(),deps:v[2].split(',').filter(|s|!s.is_empty()).map(|s|s.parse().unwrap()).collect() PRIORITY}}).collect();'''.replace('PRIORITY',',priority:v[3].parse().unwrap()' if changed else ''),
    'frames':'''let limit:i64=a[1].parse().unwrap();let data:Vec<u8>=(0..a[2].len()).step_by(2).map(|i|u8::from_str_radix(&a[2][i..i+2],16).unwrap()).collect();'''}[family]
    args={'ledger':'&events,initial','scheduler':'&jobs,budget','frames':'&data,limit'}[family]
    varied={'ledger':'std::hint::black_box(&events),initial+i%3','scheduler':'std::hint::black_box(&jobs),budget+i%3','frames':'std::hint::black_box(&data),limit+i%3'}[family]
    output={'ledger':'''println!("{{\\"balance\\":{},\\"rejected\\":{},\\"peak\\":{},\\"audit\\":{:?}}}",v.balance,v.rejected,v.peak,v.audit);''',
    'scheduler':'''println!("{{\\"order\\":{:?},\\"spent\\":{},\\"blocked\\":{}}}",v.order,v.spent,v.blocked);''',
    'frames':'''let strings:Vec<String>=v.frames.iter().map(|b|format!("\\"{}\\"",b.iter().map(|x|format!("{:02x}",x)).collect::<String>())).collect();println!("{{\\"frames\\":[{}],\\"consumed\\":{},\\"error\\":{}}}",strings.join(","),v.consumed,v.error);'''}[family]
    total={'ledger':'v.balance+v.rejected+v.peak+v.audit.iter().sum::<i64>()','scheduler':'v.spent+v.blocked+v.order.iter().sum::<i64>()','frames':'v.consumed+v.error+v.frames.iter().map(|b|b.len() as i64+b.iter().map(|x|*x as i64).sum::<i64>()).sum::<i64>()'}[family]
    return 'mod model;mod logic;\nfn main(){let a:Vec<String>=std::env::args().collect();'+parse+'\nif a.len()>3 {let repeats:i64=a[4].parse().unwrap();let mut total=0i64;for i in 0..repeats {let v=logic::run('+varied+');total+=std::hint::black_box('+total+');}println!("{}",total);} else {let v=logic::run('+args+');'+output+'}}\n'

def python_app(family,changed=False):
    parse={'ledger':"initial=int(a[1]);events=[model.Event(model.Kind(int(s.split(':')[0])),int(s.split(':')[1])) for s in a[2].split(';') if s]",
    'scheduler':"budget=int(a[1]);jobs=[]\nfor s in a[2].split(';'):\n if s:\n  v=s.split(':');jobs.append(model.Job(int(v[0]),int(v[1]),[int(x) for x in v[2].split(',') if x]"+(",int(v[3])" if changed else '')+"))",
    'frames':"limit=int(a[1]);data=bytes.fromhex(a[2])"}[family]
    args={'ledger':'events,initial','scheduler':'jobs,budget','frames':'data,limit'}[family]
    varied={'ledger':'events,initial+i%3','scheduler':'jobs,budget+i%3','frames':'data,limit+i%3'}[family]
    total={'ledger':'v.balance+v.rejected+v.peak+sum(v.audit)','scheduler':'v.spent+v.blocked+sum(v.order)','frames':'v.consumed+v.error+sum(len(b)+sum(b) for b in v.frames)'}[family]
    return 'import sys,json\nfrom dataclasses import asdict\nimport model,logic\na=sys.argv\n'+parse+'\nif len(a)>3:\n total=0\n for i in range(int(a[4])):\n  v=logic.run('+varied+');total+='+total+'\n print(total)\nelse:\n v=logic.run('+args+');value=asdict(v)\n'+(" value['frames']=[b.hex() for b in v.frames]\n" if family=='frames' else '')+' print(json.dumps(value))\n'

def sources(family,language,changed=False,starter=False):
    code=reference(family,language,False if starter else changed)
    if language=='lobo':
        app,ids=lobo_app(family)
        return {'model.lobo':models(family,language,changed),'util.lobo':LOBO_UTIL,'logic.lobo':code,'logic.lobo.ids.json':json.dumps(ids_for(family,code),indent=2)+'\n','app.lobo':app,'app.lobo.ids.json':json.dumps(ids,indent=2)+'\n'}
    suffix='rs' if language=='rust' else 'py'
    return {'model.'+suffix:models(family,language,changed),'logic.'+suffix:code,'app.'+suffix:rust_app(family,changed) if language=='rust' else python_app(family,changed)}

def editable(language):return ['logic.lobo','logic.lobo.ids.json'] if language=='lobo' else ['logic.rs'] if language=='rust' else ['logic.py']

def stub(family,language):
    if language=='lobo':
        text='module logic;\nuse model {Summary'+(',Event' if family=='ledger' else ',Job' if family=='scheduler' else '')+'};\nuse util {reverse};\nfn run('+signature(family,language)+')->Summary {'+{'ledger':'Summary {balance:initial,rejected:0,peak:initial,audit:List::empty<I64>()}','scheduler':'Summary {order:List::empty<I64>(),spent:0,blocked:List::length(jobs)}','frames':'Summary {frames:List::empty<Bytes>(),consumed:0,error:0}'}[family]+'}\n'
        return {'logic.lobo':text,'logic.lobo.ids.json':json.dumps(ids_for(family,text),indent=2)+'\n'}
    if language=='rust':return {'logic.rs':'use crate::model::{Summary'+(',Event' if family=='ledger' else ',Job' if family=='scheduler' else '')+'};\npub fn run('+signature(family,language)+')->Summary {'+{'ledger':'Summary {balance:initial,rejected:0,peak:initial,audit:Vec::new()}','scheduler':'Summary {order:Vec::new(),spent:0,blocked:jobs.len() as i64}','frames':'Summary {frames:Vec::new(),consumed:0,error:0}'}[family]+'}\n'}
    return {'logic.py':'from model import Summary'+(',Event,Kind' if family=='ledger' else ',Job' if family=='scheduler' else '')+'\ndef run('+signature(family,language)+'):\n return '+{'ledger':'Summary(initial,0,initial,[])','scheduler':'Summary([],0,len(jobs))','frames':'Summary([],0,0)'}[family]+'\n'}

def lobo_args(family,args):
    if family=='ledger':
        tokens=['Count:'+str(len(args['events']))]
        for kind,amount in args['events']:
            tokens.append('Case:'+['Credit','Debit','Lock','Unlock','Undo'][kind])
            if kind<2:tokens.append('I64:'+str(amount))
        return ['List:'+','.join(tokens),'I64:'+str(args['initial'])]
    if family=='scheduler':
        tokens=['Count:'+str(len(args['jobs']))]
        for job in args['jobs']:
            tokens.extend(['I64:'+str(job['id']),'I64:'+str(job['cost']),'Count:'+str(len(job['deps']))]);tokens.extend('I64:'+str(x) for x in job['deps'])
            if 'priority' in job:tokens.append('I64:'+str(job['priority']))
        return ['List:'+','.join(tokens),'I64:'+str(args['budget'])]
    return ['Bytes:'+args['data'],'I64:'+str(args['limit'])]

def other_args(family,args):
    if family=='ledger':return [str(args['initial']),';'.join(f'{k}:{a}' for k,a in args['events'])]
    if family=='scheduler':return [str(args['budget']),';'.join(str(j['id'])+':'+str(j['cost'])+':'+','.join(map(str,j['deps']))+(':'+str(j['priority']) if 'priority' in j else '') for j in args['jobs'])]
    return [str(args['limit']),args['data']]

def untyped(value):
    kind=value['type'];body=value['value']
    if kind=='I64':return int(body)
    if kind in ['Bool','Text','Bytes']:return body
    if kind=='Record':return {k:untyped(v) for k,v in body['fields'].items()}
    if kind=='List':return [untyped(x) for x in body['elements']]
    raise ValueError('unsupported result '+str(value))

def build(directory,language,destination):
    if language=='lobo':command=[str(COMPILER),'compile',str(directory/'app.lobo'),'--source',str(directory/'model.lobo'),'--source',str(directory/'util.lobo'),'--source',str(directory/'logic.lobo'),'-o',str(destination)]
    elif language=='rust':command=['rustc','--edition=2021','-C','opt-level=3','-C','overflow-checks=yes','-C','panic=abort','-C','strip=symbols',str(directory/'app.rs'),'-o',str(destination)]
    else:command=['python3','-m','py_compile',str(directory/'app.py'),str(directory/'model.py'),str(directory/'logic.py')]
    p=subprocess.run(command,cwd=directory,capture_output=True,text=True,timeout=35)
    return {'ok':p.returncode==0,'command':command,'exit_code':p.returncode,'stdout':p.stdout,'stderr':p.stderr}

def execute(directory,language,family,args,binary,repeats=None):
    if language=='lobo':
        command=[str(binary),'--entry','study.'+family+('.bench' if repeats is not None else '.entry'),'--fuel','100000000','--heap-cells','4000000']
        for arg in lobo_args(family,args)+(['I64:'+str(repeats)] if repeats is not None else []):command+=['--arg',arg]
    else:command=([str(binary)] if language=='rust' else ['python3',str(directory/'app.py')])+other_args(family,args)+(['--bench',str(repeats)] if repeats is not None else [])
    p=subprocess.run(command,cwd=directory,capture_output=True,text=True,timeout=10)
    if p.returncode:raise ValueError(json.dumps({'command':command,'exit_code':p.returncode,'stdout':p.stdout,'stderr':p.stderr})[:4096])
    value=json.loads(p.stdout)
    return untyped(value['value']) if language=='lobo' else value

def save_sources(folder,files):
    folder.mkdir(parents=True,exist_ok=True)
    for name,text in files.items():(folder/name).write_text(text)

def validate_references(out):
    out.mkdir(parents=True,exist_ok=False);records=[]
    for family,language,changed in itertools.product(FAMILIES,LANGUAGES,[False,True]):
        folder=out/(family+'-'+language+('-changed' if changed else '-base'));save_sources(folder,sources(family,language,changed));binary=folder/'program';compiled=build(folder,language,binary);row={'family':family,'language':language,'changed':changed,'build':compiled,'cases':[]};records.append(row);write_json(out/'results.json',records)
        if not compiled['ok']:raise RuntimeError(compiled)
        for args in cases(family,changed,True):
            result=execute(folder,language,family,args,binary);expected=oracle(family,args,changed);row['cases'].append({'input':args,'actual':result,'expected':expected,'ok':result==expected})
            if result!=expected:write_json(out/'results.json',records);raise ValueError((family,language,changed,args,result,expected))
        write_json(out/'results.json',records)
    return records

COMMON_GUIDE='''Use only the supplied task directory. Read TASK.txt, GUIDE.txt and the model/algorithm files. Change only the listed editable files. The app and models are trusted read-only adapters. No dependencies, network, file IO, subprocesses or unsafe code in the algorithm. Standard in-memory collections are allowed. Host-injected instruction/skill reads are permitted when required, but other trials, repository code, private graders and reference solutions are outside scope. Do not inspect the compiler binary or its parent directory. Use python3 check.py to submit a visible check; the host compiles and grades a snapshot, at most six submissions. No direct compiler/test invocation is needed. Initial stubs may compile but are deliberately incorrect. Use checks before and after editing. Build artifacts belong to the host. End with a concise result. All inputs and arithmetic fit signed 64-bit values under the stated bounds; results must preserve exact integers. No output or external side effects in run().'''
LOBO_GUIDE='''Lobo 1.11.2 is a native language. There is no projection/interpreter step. model.lobo declares the immutable DTOs. logic.lobo and its adjacent logic.lobo.ids.json are editable; app/model/util sources are fixed. Each block returns its final expression without a trailing semicolon. Use fn f(x:I64)->I64 {x+1}, let x=...; if condition {a} else {b}, &&/||/!, checked I64 arithmetic, and ==/!= comparisons. Names cannot shadow other visible names or function/type declarations. No assignment, break, continue, return keyword or for statement.
Records: record Pair {left:I64,right:I64}; Pair {left:1,right:2}; pair.left; Pair {...pair,left:3}. Variants: match event {Credit(n)=> expression,Debit(n)=> expression,Lock=> expression,Unlock=> expression}. Every case must occur. Expected types infer case owners. Parenthesize complex match subjects: match (cursor.best) {...}.
Lists: List<T>, List::empty<I64>(), List::prepend(value,list), List::length(list), List::head(list), List::tail(list). Head/tail require nonempty. Prepend reverses order; the supplied util.reverse<T> restores order and may infer T. Lists are persistent; no indexing, append or mutation operation.
Counted loops carry one typed value: fold(i in n,total:I64=0) {total+i}. The index runs 0..n-1; zero iterations return the initial state. For multiple variables, carry a record: fold(i in List::length(items),state:Cursor=Cursor {pending:items,total:0}) {Cursor {pending:List::tail(state.pending),total:state.total+List::head(state.pending)}}. State-driven loop: while(state:State=initial; condition_using_state) {new_state}. To stop early, carry a Bool and test it in the loop condition. Finite execution fuel/heap apply; do not use deep recursive traversal.
Bytes: Bytes::length(data), Bytes::at(data,index) returns 0..255, Bytes::slice(data,start,count), Bytes::empty(), Bytes::push(data,value). Guard bounds before at/slice. List<Bytes> is supported. Remainder uses %, division truncates toward zero. Ordinary expected failures are values in the declared DTO, not runtime faults.
New functions/records/variants need identity entries. Existing IDs must stay stable. After adding declarations, run the explicit authoring command below (it only updates the managed identity map). Then use python3 check.py.
'''
RUST_GUIDE='''Rust edition 2021, standard library only. logic.rs exports pub fn run with the signature in TASK.txt; app.rs and model.rs are fixed. Import DTOs with use crate::model::{Event,Summary} or {Job,Summary}. Event payload variants are Event::Credit(i64), Event::Debit(i64), Event::Lock, Event::Unlock; the change model also has Event::Undo. Job has public id:i64, cost:i64, deps:Vec<i64>; the change model also has priority:i64. Summary fields are public and listed in model.rs. Use Vec<T>, slices &[T], .iter(), .len(), .contains(), push/pop, pattern matching and ordinary helpers as needed. Constructors use Summary {field:value,...}. No serde or external crates are available. The host builds optimized machine code with rustc --edition=2021 -C opt-level=3 -C overflow-checks=yes -C panic=abort -C strip=symbols. Build/check work is included in your wall budget. Submit with python3 check.py rather than modifying the wrapper. No file/network/process IO or unsafe code in logic.rs.'''
PYTHON_GUIDE='''Python 3.12, standard library only. logic.py exports run with the signature in TASK.txt; app.py and model.py are fixed. Import DTOs from model. Ledger uses Event(kind:Kind,amount:int), with Kind.CREDIT/DEBIT/LOCK/UNLOCK (and UNDO in the change model). Scheduler uses Job(id:int,cost:int,deps:list[int]) and change adds priority:int. Summary dataclasses and their field order are in model.py. Input bytes are bytes objects, not strings; the trusted wrapper handles hexadecimal input/output. Use in-memory lists, tuples, sets, dictionaries and ordinary helpers as needed. Keep exact integer results; no float conversion. Imports from model, dataclasses, enum, typing, collections, functools, itertools and math are allowed. No imports for file/network/process IO, open/eval/exec or dynamic import. The host py-compiles and executes the program. Submit with python3 check.py; no application prints or wrapper changes.'''

BASE_REQUIREMENTS={
'ledger':'''Implement run(events,initial)->Summary(balance,rejected,peak,audit). Events have kinds Credit(amount), Debit(amount), Lock and Unlock. Start at initial balance, unlocked, rejected=0, peak=initial, empty audit. Credit/Debit amounts are valid iff 0<=amount<=1000000. Invalid amounts reject once and leave the balance unchanged. A valid credit always succeeds, even locked. A debit succeeds iff unlocked and amount<=balance; otherwise reject once. Zero-value credits/debits succeed under the same rules. Lock/Unlock are idempotent and never reject. After EVERY event append the current balance to audit, including rejects and locks. Peak is the maximum balance ever observed, including initial. Inputs: 0<=initial<=1000000, at most32 events, amounts between -1000000 and1000001; Lock/Unlock amounts are ignored. No overflow is possible.''',
'scheduler':'''Implement run(jobs,budget)->Summary(order,spent,blocked). Job fields id,cost,deps. Jobs have unique IDs exactly0..n-1, arrive in arbitrary order,0<=n<=24,0<=cost<=64,0<=budget<=512. deps contains valid IDs, may repeat and may contain self-dependencies or cycles. Start with no completed jobs. Repeatedly choose the LOWEST ID among unfinished jobs whose every dependency is completed and whose cost<=remaining budget; execute it, append its ID to order, subtract its cost. A zero-cost job can run at zero budget. If no eligible fitting job remains, stop. spent is total executed cost. blocked counts EVERY unfinished job, whether blocked by dependencies or budget. Do not stop merely because a lower-ID job is unaffordable.''',
'frames':'''Implement run(data,limit)->Summary(frames,consumed,error). data is bytes of length<=512;0<=limit<=64. Each frame is one unsigned length byte n, followed by n payload bytes, followed by one checksum byte equal to sum(payload)%256. Empty payload is valid and needs checksum0. Parse from left to right, retaining payloads in input order. At a frame start: first reject n>limit with error3 EVEN IF payload/checksum is missing; otherwise missing payload/checksum is error1; otherwise wrong checksum is error2. Stop on the first error. consumed is the offending frame's START offset; retain only earlier valid frames. With no error, consumed=len(data),error=0. Empty input succeeds. frames is a list of bytes values, not decoded text.'''}
CHANGE_REQUIREMENTS={
'ledger':'''New Event Undo (amount ignored) is in the supplied model. Undo restores the balance immediately before the most recent SUCCESSFUL Credit/Debit, and removes that event from undo history. Successful zero-value events also enter history. Rejected events and Lock/Unlock do NOT enter history. Undo rejects once without popping history if locked or history is empty. Undo itself never enters history. A successful Undo does NOT decrease peak, which is maximum EVER observed. Append balance to audit for Undo too. Preserve every original behavior.''',
'scheduler':'''The supplied Job model adds integer priority in[-1000,1000]. Among eligible fitting jobs choose HIGHEST priority, then LOWEST ID on a tie. Readiness and budget rules are unchanged. A higher-priority unaffordable job must not prevent a lower-priority fitting job from running. Preserve all original behaviors for equal priorities.''',
'frames':'''Two new header forms: byte255 is standalone padding: consume it, produce no frame and continue (even limit0). Byte254 begins an extended header: the next TWO bytes are an unsigned little-endian payload length, followed by payload and the normal checksum. Missing either length byte gives error1 at the marker's offset; once both length bytes exist, check length>limit BEFORE payload availability. Nonminimal extended lengths including0 are permitted. Ordinary length bytes0..253 retain their previous rules. On any error consumed is the START of the offending header, including marker254; preceding padding remains consumed. Preserve all earlier valid frames.'''}

def guide(language):
    text=COMMON_GUIDE+'\n\n'+{'lobo':LOBO_GUIDE,'rust':RUST_GUIDE,'python':PYTHON_GUIDE}[language]
    if language=='lobo':text+='\n'+str(COMPILER)+' identities sync logic.lobo --source model.lobo --source util.lobo\n'
    assert len(text.encode())<=8000
    return text

def prediction_inputs(family):
    if family=='ledger':return [
        {'initial':0,'events':[(0,5),(1,2),(0,1)]},
        {'initial':3,'events':[(2,0),(1,0),(0,2),(3,0),(1,5)]},
        {'initial':1,'events':[(0,-1),(1,2),(1,1)]},
        {'initial':0,'events':[(0,0),(1,0),(2,0),(3,0)]},
        {'initial':7,'events':[(1,0),(2,0),(0,0),(1,1)]},
        {'initial':0,'events':[(0,1000000),(0,1),(1,1000001)]}]
    if family=='scheduler':return [
        {'jobs':[{'id':0,'cost':2,'deps':[1]},{'id':1,'cost':1,'deps':[]}],'budget':3},
        {'jobs':[{'id':2,'cost':1,'deps':[0]},{'id':0,'cost':2,'deps':[]},{'id':1,'cost':1,'deps':[]}],'budget':2},
        {'jobs':[{'id':0,'cost':0,'deps':[0]},{'id':1,'cost':0,'deps':[]}],'budget':0},
        {'jobs':[{'id':0,'cost':1,'deps':[1]},{'id':1,'cost':1,'deps':[2]},{'id':2,'cost':1,'deps':[0]}],'budget':10},
        {'jobs':[{'id':0,'cost':5,'deps':[]},{'id':1,'cost':2,'deps':[]}],'budget':2},
        {'jobs':[{'id':0,'cost':1,'deps':[]},{'id':1,'cost':1,'deps':[0,0]},{'id':2,'cost':0,'deps':[1]}],'budget':2}]
    return [{'data':'','limit':0},{'data':'0000','limit':0},{'data':'02010203','limit':2},{'data':'02010204','limit':2},{'data':'03','limit':2},{'data':'0101010201','limit':2}]

def valid_input(family,args):
    integer=lambda x:type(x) is int
    if not isinstance(args,dict):return False
    try:
        if family=='ledger':
            return set(args)=={'initial','events'} and integer(args['initial']) and 0<=args['initial']<=1000000 and isinstance(args['events'],list) and len(args['events'])<=32 and all(isinstance(e,list) and len(e)==2 and integer(e[0]) and 0<=e[0]<4 and integer(e[1]) and -1000000<=e[1]<=1000001 for e in args['events'])
        if family=='scheduler':
            jobs=args['jobs'];n=len(jobs)
            return set(args)=={'jobs','budget'} and integer(args['budget']) and 0<=args['budget']<=512 and isinstance(jobs,list) and n<=24 and sorted(j['id'] for j in jobs)==list(range(n)) and all(set(j)=={'id','cost','deps'} and integer(j['id']) and integer(j['cost']) and 0<=j['cost']<=64 and isinstance(j['deps'],list) and len(j['deps'])<=24 and all(integer(d) and 0<=d<n for d in j['deps']) for j in jobs)
        import re
        return set(args)=={'data','limit'} and isinstance(args['data'],str) and len(args['data'])%2==0 and len(args['data'])<=1024 and re.fullmatch('[0-9a-fA-F]*',args['data']) is not None and integer(args['limit']) and 0<=args['limit']<=64
    except (KeyError,TypeError,ValueError):return False

def strict_oracle(family,args):
    args=json.loads(json.dumps(args))
    if family=='ledger':args['events']=[(k,-1 if k<2 and a==1000000 else a) for k,a in args['events']]
    elif family=='scheduler':args['budget']-=1
    else:args['limit']-=1
    return oracle(family,args)

def equal(a,b):return type(a) is type(b) and (set(a)==set(b) and all(equal(a[k],b[k]) for k in a) if isinstance(a,dict) else len(a)==len(b) and all(equal(x,y) for x,y in zip(a,b)) if isinstance(a,list) else a==b)

def comprehension_questions(family):
    change={'ledger':'valid_amount changes its upper bound from1000000 to999999','scheduler':'fits changes cost<=remaining to cost<remaining','frames':'fits changes length<=limit to length<limit'}[family]
    target={'ledger':'balance=7,rejected=1,peak=9 (any audit)','scheduler':'order=[1,0],spent=3,blocked=0','frames':'frames=["01","02"],consumed=6,error=0'}[family]
    return ('Analyze the supplied BASE program without executing it. Return ONE JSON object with keys: predictions (the SIX full normalized Summary objects for the inputs below, in order); witness (any valid BASE input object yielding '+target+'); direct_calls (names of application functions directly invoked by run, ignoring constructors/builtins/standard-library methods and util.reverse); impacted (names of functions in logic whose return values can change for identical direct arguments if '+change+'; ignore reachability alone); fault_location (the predicate function responsible for that boundary policy); counterexample (any valid BASE input distinguishing the original program from that strict-policy mutant). Bytes in answers are lowercase hexadecimal strings. Use plain function names without module prefixes. No tools/target execution. Inputs:\n'+json.dumps(prediction_inputs(family)))

def grade_comprehension(family,answer):
    answer=answer if isinstance(answer,dict) else {};pred=answer.get('predictions');checks={}
    for i,args in enumerate(prediction_inputs(family)):checks['prediction_'+str(i)]=isinstance(pred,list) and len(pred)==6 and equal(pred[i],oracle(family,args))
    witness=answer.get('witness');checks['witness']=False
    if valid_input(family,witness):
        value=oracle(family,witness)
        checks['witness']=all(value[k]==v for k,v in {'ledger':{'balance':7,'rejected':1,'peak':9},'scheduler':{'order':[1,0],'spent':3,'blocked':0},'frames':{'frames':['01','02'],'consumed':6,'error':0}}[family].items())
    def names(value,wanted):return isinstance(value,list) and len(value)==len(wanted) and all(isinstance(x,str) for x in value) and set(value)==set(wanted)
    checks['direct_calls']=names(answer.get('direct_calls'),{'ledger':['step'],'scheduler':['choose'],'frames':['read_header','fits','checksum']}[family])
    checks['impacted']=names(answer.get('impacted'),{'ledger':['valid_amount','step','run'],'scheduler':['fits','choose','run'],'frames':['fits','run']}[family])
    checks['fault_location']=answer.get('fault_location')==('valid_amount' if family=='ledger' else 'fits')
    counter=answer.get('counterexample');checks['counterexample']=valid_input(family,counter) and oracle(family,counter)!=strict_oracle(family,counter)
    return {'accepted':all(checks.values()),'passed':sum(checks.values()),'total':11,'checks':checks}


def correct_comprehension(family):
    witness={'ledger':{'initial':9,'events':[[1,2],[1,8]]},'scheduler':{'jobs':[{'id':0,'cost':2,'deps':[1]},{'id':1,'cost':1,'deps':[]}],'budget':3},'frames':{'data':'010101010202','limit':2}}[family]
    counter={'ledger':{'initial':0,'events':[[0,1000000]]},'scheduler':{'jobs':[{'id':0,'cost':0,'deps':[]}],'budget':0},'frames':{'data':'0000','limit':0}}[family]
    return {'predictions':[oracle(family,a) for a in prediction_inputs(family)],'witness':witness,'direct_calls':{'ledger':['step'],'scheduler':['choose'],'frames':['read_header','fits','checksum']}[family],'impacted':{'ledger':['valid_amount','step','run'],'scheduler':['fits','choose','run'],'frames':['fits','run']}[family],'fault_location':'valid_amount' if family=='ledger' else 'fits','counterexample':counter}


def mutant(family,language,changed=False):
    text=reference(family,language,changed)
    if family=='ledger':
        return text.replace('<=1000000','<1000000')
    return text.replace('<=remaining','<remaining') if family=='scheduler' else text.replace('<=limit','<limit')


def integrity(folder,fixed,language):
    failures=[]
    for name,text in fixed.items():
        path=folder/name
        if path.is_symlink():failures.append('symlink:'+name)
        elif not path.is_file():failures.append('missing:'+name)
        elif name not in editable(language) and path.read_text()!=text:failures.append('modified:'+name)
        elif path.stat().st_size>65536:failures.append('oversize:'+name)
    if language=='lobo':
        try:
            initial=json.loads(fixed['logic.lobo.ids.json'])['identities'];current=json.loads((folder/'logic.lobo.ids.json').read_text())['identities']
            failures += ['changed_identity:'+key for key,value in initial.items() if current.get(key)!=value]
        except (ValueError,KeyError,OSError,TypeError):failures.append('invalid_identity_file')
    allowed=set(fixed)|{'TASK.txt','GUIDE.txt','check.py','.check-io','AGENTS.md'}
    failures += ['unexpected:'+p.name for p in folder.iterdir() if p.name not in allowed]
    return failures


def policy(code,language):
    errors=[]
    if language=='python':
        try:
            tree=ast.parse(code)
            allowed={'model','dataclasses','enum','typing','collections','functools','itertools','math','__future__'}
            for node in ast.walk(tree):
                if isinstance(node,ast.Import):
                    errors += ['import:'+a.name for a in node.names if a.name.split('.')[0] not in allowed]
                if isinstance(node,ast.ImportFrom) and (node.module or '').split('.')[0] not in allowed:errors.append('import:'+str(node.module))
                if isinstance(node,ast.Call) and isinstance(node.func,ast.Name) and node.func.id in {'open','eval','exec','compile','__import__','input'}:errors.append('call:'+node.func.id)
                if isinstance(node,ast.Attribute) and node.attr.startswith('__'):errors.append('dunder:'+node.attr)
        except SyntaxError:pass # compilation produces the syntax diagnostic
    elif language=='rust':
        import re
        for word in ['unsafe','std::fs','std::process','std::net','include!','include_str!','include_bytes!','env!']:
            if word in re.sub(r'\s+','',code):errors.append('forbidden:'+word)
    return sorted(set(errors))


CHECK_CLIENT='''import json,time,uuid
from pathlib import Path
root=Path('.check-io');root.mkdir(exist_ok=True)
key=uuid.uuid4().hex
(root/(key+'.request.json')).write_text('{}')
response=root/(key+'.response.json')
end=time.monotonic()+40
while not response.exists() and time.monotonic()<end:time.sleep(.05)
if not response.exists():raise SystemExit('controller timeout')
print(response.read_text())
'''


def assess(folder,fixed,row,out,hidden=False):
    out.mkdir(parents=True,exist_ok=False)
    issues=integrity(folder,fixed,row['language'])
    suffix={'lobo':'lobo','rust':'rs','python':'py'}[row['language']]
    code=(folder/('logic.'+suffix)).read_text() if (folder/('logic.'+suffix)).is_file() else ''
    issues+=policy(code,row['language'])
    files=dict(fixed)
    for name in editable(row['language']):
        path=folder/name
        if path.is_file() and not path.is_symlink() and path.stat().st_size<=65536:files[name]=path.read_text()
    save_sources(out/'source',files)
    result={'issues':issues,'build':None,'cases':[],'accepted':False}
    if issues:write_json(out/'result.json',result);return result
    try:
        compiled=build(out/'source',row['language'],out/'program');result['build']=compiled
        if compiled['ok']:
            for args in cases(row['family'],row['phase']=='change',hidden):
                expected=oracle(row['family'],args,row['phase']=='change')
                try:actual=execute(out/'source',row['language'],row['family'],args,out/'program');error=None
                except Exception as exc:actual=None;error=str(exc)[:4096]
                result['cases'].append({'input':args,'expected':expected,'actual':actual,'error':error,'ok':equal(expected,actual)})
        result['accepted']=bool(result['cases']) and all(c['ok'] for c in result['cases'])
    except Exception as exc:result['infrastructure_error']=repr(exc)
    write_json(out/'result.json',result);return result


def visible_feedback(result):
    if result['issues']:return {'status':'scope_error','issues':result['issues']}
    if result.get('infrastructure_error'):return {'status':'infrastructure_error','message':result['infrastructure_error']}
    if not result['build']['ok']:return {'status':'compile_error','diagnostic':(result['build']['stdout']+result['build']['stderr'])[-5000:]}
    failures=[c for c in result['cases'] if not c['ok']]
    return {'status':'pass' if result['accepted'] else 'fail','passed':sum(c['ok'] for c in result['cases']),'total':len(result['cases']),'failures':failures[:3]}


def fixtures(row):
    family,language,phase=row['family'],row['language'],row['phase']
    files=sources(family,language,phase=='change',starter=phase=='change')
    if phase=='construction':files.update(stub(family,language))
    return files


def task_text(row):
    language,family,phase=row['language'],row['family'],row['phase']
    text='Task family: '+family+'. Phase: '+phase+'.\n'+BASE_REQUIREMENTS[family]+'\n\nRequired signature: run('+signature(family,language)+').\n'
    if phase=='change':text+='\n'+CHANGE_REQUIREMENTS[family]+'\nStart from the supplied correct base algorithm. The model already includes the new field/case.\n'
    if phase=='comprehension':text+='\n'+comprehension_questions(family)+'\nUse NO tools or execution; reason from the supplied source. End with ONLY the requested JSON object, no Markdown.\n'
    else:text+='\nEditable files: '+', '.join(editable(language))+'. Check with python3 check.py. No other files may be changed.\n'
    return text


def command(folder,row,last):
    args=['codex','exec','--ignore-user-config','--ephemeral','--json','--sandbox','read-only' if row['phase']=='comprehension' else 'workspace-write','--skip-git-repo-check','--disable','multi_agent','--disable','multi_agent_v2','--disable','apps','--disable','plugins','--disable','memories','-C',str(folder),'-m',MODEL,'-c','model_reasoning_effort="high"','-c','approval_policy="never"','--output-last-message',str(last)]
    if row['phase']=='comprehension':args+=['--disable','shell_tool']
    return args+['-']


def parse_answer(text):
    try:
        if text.startswith('```') and text.rstrip().endswith('```'):text='\n'.join(text.strip().splitlines()[1:-1])
        return json.loads(text)
    except (ValueError,TypeError):return None


def run_trial(row,base):
    out=base/('trial-%02d'%row['index']);out.mkdir(parents=True,exist_ok=False)
    write_json(out/'assignment.json',row)
    folder=Path(tempfile.mkdtemp(prefix='lobo-matched-'));files=fixtures(row);save_sources(folder,files)
    extras={'TASK.txt':task_text(row),'GUIDE.txt':guide(row['language']),'check.py':CHECK_CLIENT,'AGENTS.md':'This is an isolated matched experiment. Follow TASK.txt and GUIDE.txt. Only the declared algorithm files are editable. Do not inspect other directories.\n'}
    save_sources(folder,extras);files=files|extras;save_sources(out/'initial',files)
    prompt='Complete TASK.txt following GUIDE.txt. The limits are 120 seconds, 16 command-execution tool items and 6 visible check submissions. All build/check work counts in wall time. Do not delegate.\n'
    if row['phase']=='comprehension':
        prompt='Source-only comprehension. Do not use tools.\n'+extras['TASK.txt']+'\n'+extras['GUIDE.txt']+'\n'+'\n'.join('FILE '+name+'\n'+text for name,text in files.items() if '.ids.' not in name and not name.startswith('app.'))
    (out/'prompt.txt').write_text(prompt);mailbox=folder/'.check-io';mailbox.mkdir();stop=threading.Event();checks=[];serviced=set()
    def controller():
        while not stop.is_set():
            for request in sorted(mailbox.glob('*.request.json')):
                if request.name in serviced:continue
                serviced.add(request.name)
                if len(checks)>=CHECK_LIMIT:response={'status':'check_limit','limit':CHECK_LIMIT}
                else:
                    checks.append({'index':len(checks),'start_s':time.monotonic()-start})
                    try:result=assess(folder,files,row,out/('check-%02d'%(len(checks)-1)));response=visible_feedback(result)
                    except Exception as exc:response={'status':'infrastructure_error','message':repr(exc)}
                    checks[-1]['end_s']=time.monotonic()-start;checks[-1]['feedback']=response
                write_json(mailbox/request.name.replace('.request.json','.response.json'),response)
            stop.wait(.05)
    args=command(folder,row,out/'last-message.txt');write_json(out/'command.json',args)
    env=dict(os.environ);env.pop('CODEX_THREAD_ID',None);env.pop('CODEX_INTERNAL_ORIGINATOR_OVERRIDE',None)
    start=time.monotonic();thread=threading.Thread(target=controller,daemon=True);thread.start()
    result={'assignment':row,'command':args,'deadline_s':DEADLINE,'tool_limit':TOOL_LIMIT,'check_limit':CHECK_LIMIT,'started_unix':time.time(),'load_before':os.getloadavg(),'status':'unknown'}
    events=[];tools={};reason=None
    try:
        with (out/'events.jsonl').open('w') as log,(out/'stderr.txt').open('w') as stderr:
            process=subprocess.Popen(args,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,cwd=folder,env=env,text=True,start_new_session=True,bufsize=1)
            process.stdin.write(prompt);process.stdin.close()
            selector=selectors.DefaultSelector();selector.register(process.stdout,selectors.EVENT_READ)
            while True:
                elapsed=time.monotonic()-start
                if elapsed>DEADLINE:reason='wall_limit';break
                if len(tools)>TOOL_LIMIT:reason='tool_limit';break
                ready=selector.select(.1)
                if ready:
                    line=process.stdout.readline()
                    if not line:
                        if process.poll() is not None:break
                        continue
                    log.write(line);log.flush()
                    try:event=json.loads(line);events.append(event)
                    except ValueError:continue
                    item=event.get('item',{})
                    if item.get('type')=='command_execution':tools[item.get('id',str(len(tools)))]=item
                elif process.poll() is not None:break
            if process.poll() is None:
                os.killpg(process.pid,signal.SIGTERM)
                try:process.wait(timeout=3)
                except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait()
            # Drain buffered terminal usage and final events after process completion.
            for line in process.stdout:
                log.write(line)
                try:events.append(json.loads(line))
                except ValueError:pass
            result.update(exit_code=process.returncode,status=reason or ('completed' if process.returncode==0 else 'agent_error'))
    except Exception as exc:result.update(status='infrastructure_error',infrastructure_error=repr(exc))
    finally:
        result['agent_wall_s']=time.monotonic()-start;stop.set();thread.join(timeout=45)
        result.update(usage=usage(events),command_tool_items=len(tools),checks=checks,load_after=os.getloadavg())
        last=(out/'last-message.txt').read_text() if (out/'last-message.txt').exists() else ''
        if row['phase']=='comprehension':result['grading']=grade_comprehension(row['family'],parse_answer(last))
        else:result['grading']=assess(folder,files,row,out/'heldout',hidden=True)
        result['accepted']=result['status']=='completed' and result['grading']['accepted'] and len(checks)<=CHECK_LIMIT and len(tools)<=TOOL_LIMIT
        result['workspace']=str(folder);write_json(out/'result.json',result)
    return result


def validate_mutants(out):
    out.mkdir(parents=True,exist_ok=False);rows=[]
    for family,language,changed in itertools.product(FAMILIES,LANGUAGES,[False,True]):
        row={'family':family,'language':language,'phase':'change' if changed else 'construction'};folder=out/(family+'-'+language+str(changed));files=sources(family,language,changed);files['logic.'+{'lobo':'lobo','rust':'rs','python':'py'}[language]]=mutant(family,language,changed);save_sources(folder,files)
        # Keep controller's own assessment directory outside candidate workspace.
        result=assess(folder,files,row,out/(folder.name+'-assessment'),True)
        killed=result['build'] is not None and result['build']['ok'] and any(not c['ok'] for c in result['cases'])
        rows.append({'family':family,'language':language,'changed':changed,'killed_by_behavior':killed});write_json(out/'results.json',rows)
        if not killed:raise ValueError(rows[-1])
    return rows


def register(out):
    out.mkdir(parents=True,exist_ok=False)
    manifest={'source_commit':'75a551fc0aec1fbc94f1fddb2a0f6a59ddb3f122','model':MODEL,'reasoning':'high','deadline_s':DEADLINE,'command_tool_items':TOOL_LIMIT,'visible_checks':CHECK_LIMIT,'hard_token_cap':None,'cache_policy':'provider managed; cached tokens included in input','seed':20261123,'schedule':schedule(),'files':{},'guides':{}}
    for row in schedule():
        name='%02d'%row['index'];trial=out/name;save_sources(trial,fixtures(row)|{'TASK.txt':task_text(row),'GUIDE.txt':guide(row['language'])})
    for path in sorted(out.rglob('*')):
        if path.is_file():manifest['files'][str(path.relative_to(out))]={'sha256':digest(path.read_bytes()),'bytes':path.stat().st_size}
    for language in LANGUAGES:manifest['guides'][language]={'bytes':len(guide(language).encode()),'sha256':digest(guide(language).encode())}
    manifest['harness_sha256']=digest(Path(__file__).read_bytes());write_json(out/'manifest.json',manifest)
    return manifest


def run_all(base):
    registration=json.loads((PRIVATE/'registration/manifest.json').read_text())
    if registration['harness_sha256']!=digest(Path(__file__).read_bytes()):raise ValueError('harness changed after registration')
    base.mkdir(parents=True,exist_ok=False);results=[];infrastructure_streak=0
    for row in registration['schedule']:
        print('START',row,flush=True);result=run_trial(row,base);results.append(result)
        write_json(base/'results.json',results);print('END',row['index'],result['status'],result['accepted'],result['usage'],flush=True)
        infra=result['status'] in {'infrastructure_error','agent_error'} or result['grading'].get('infrastructure_error')
        infrastructure_streak=infrastructure_streak+1 if infra else 0
        if infrastructure_streak>=3:
            write_json(base/'stopped.json',{'reason':'three consecutive infrastructure failures','remaining':registration['schedule'][len(results):]});break


def transport_canaries(out):
    out.mkdir(parents=True,exist_ok=False);records=[]
    for language in LANGUAGES:
        folder=Path(tempfile.mkdtemp(prefix='lobo-transport-canary-'));save_sources(folder,{'check.py':CHECK_CLIENT,'AGENTS.md':'This is an isolated transport check. Follow the user prompt. Do not inspect other directories.\n'})
        box=folder/'.check-io';box.mkdir();trial=out/language;trial.mkdir();stop=threading.Event();seen=[]
        def service():
            while not stop.is_set():
                for path in box.glob('*.request.json'):
                    response=path.with_name(path.name.replace('.request.json','.response.json'))
                    if not response.exists():seen.append(path.name);write_json(response,{'status':'pass','nonce':'mailbox-731'})
                stop.wait(.05)
        thread=threading.Thread(target=service,daemon=True);thread.start()
        row={'phase':'construction','language':language};args=command(folder,row,trial/'last-message.txt');write_json(trial/'command.json',args)
        prompt='Transport canary for the '+language+' arm. Run python3 check.py exactly once in this directory and reply with only the nonce returned by the host. Do not inspect other directories or solve any programming task.'
        started=time.monotonic()
        try:
            p=subprocess.run(args,input=prompt,text=True,capture_output=True,timeout=90,cwd=folder)
            (trial/'events.jsonl').write_text(p.stdout);(trial/'stderr.txt').write_text(p.stderr);events=[]
            for line in p.stdout.splitlines():
                try:events.append(json.loads(line))
                except ValueError:pass
            last=(trial/'last-message.txt').read_text() if (trial/'last-message.txt').exists() else ''
            row.update(exit_code=p.returncode,wall_s=time.monotonic()-started,usage=usage(events),mailbox_requests=len(seen),accepted=p.returncode==0 and len(seen)==1 and 'mailbox-731' in last)
        finally:stop.set();thread.join(timeout=2)
        records.append(row);write_json(out/'results.json',records)
        if not row['accepted']:raise ValueError(row)
    return records


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('operation',choices=['register','run','validate','mutants','canary']);parser.add_argument('--out',type=Path,required=True);args=parser.parse_args()
    functions={'register':register,'run':run_all,'validate':validate_references,'mutants':validate_mutants,'canary':transport_canaries}
    functions[args.operation](args.out.resolve())

if __name__=='__main__':main()
