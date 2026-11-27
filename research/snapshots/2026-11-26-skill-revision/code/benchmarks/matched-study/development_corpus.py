"""Fresh development tasks, independent Python oracles and native calibration candidates."""
import json
import random
import re

TASKS=('splice','batch','dispatch')
UTIL='''module util;
record Reverse<T> {pending:List<T>,output:List<T>}
fn reverse<T>(items:List<T>)->List<T> {
 let final=fold(i in List::length(items),state:Reverse<T>=Reverse {pending:items,output:List::empty<T>()}) {
  Reverse {pending:List::tail(state.pending),output:List::prepend(List::head(state.pending),state.output)}
 };final.output
}
'''
MODELS={
'splice':'record Change {text:Text,removed:Text,scalar_delta:I64,byte_delta:I64}\nvariant Edit<T> {Ok(T),Err(I64)}',
'batch':'record Sample {raw:I64,scale:I64}\nrecord Summary {values:List<I64>,total:I64,maximum:I64}\nvariant Outcome<T> {Ok(T),Err(I64)}',
'dispatch':'record Entry {key:I64,value:I64,active:Bool}\nrecord Summary {kept:List<Entry>,dropped:I64}'}
REQUIREMENTS={
'splice':'''Implement run(text:Text,start:I64,count:I64,replacement:Text)->Edit<Change>. Replace exactly count Unicode scalars starting at start with replacement. Validate start first: negative or greater than scalar length returns Err(1). Then negative count or count greater than length-start returns Err(2). An empty replacement and an empty window at end are valid. On success return Ok(Change): text is prefix+replacement+suffix, removed is the original scalar window, scalar_delta is result scalar length minus original scalar length, byte_delta is result UTF-8 byte length minus original byte length. Preserve every scalar, including NUL/combining marks; no normalization. start/count may be any I64, so avoid overflowing addition. Texts have at most24 scalars.''',
'batch':'''Implement convert(sample:Sample)->Outcome<I64> and run(samples:List<Sample>)->Outcome<Summary>. convert rejects raw<0 with Err(11); otherwise scale<=0 gives Err(12); otherwise Ok(raw*scale). Check raw before scale. run converts samples in input order and returns the FIRST error unchanged, with no partial Summary. On success values contains all converted amounts in original order, total is their sum, maximum is their maximum (0 when empty). Each input list has at most16 samples, raw in[-10,40], scale in[-2,8]. Arithmetic fits I64. Keep both existing functions/IDs. Pure helper functions may be added.''',
'dispatch':'''Change run(entries:List<Entry>)->Summary. The supplied base algorithm filters inactive entries and retains active entries in input order. New behavior: only the LAST entry for each key matters. If it is inactive, that key contributes nothing (a tombstone). If active, retain that exact entry, in the order of its last position in the original input, not sorted by key. dropped equals original length minus number retained, including tombstones and superseded entries. Empty input succeeds. There are at most16 entries, keys in[-3,5], arbitrary I64 values. Keep the existing Cursor and run declarations/IDs; helpers may be added.'''}

def identities(source,prefix,imports=None):
    values={'module':prefix+'.module'}
    values.update({name:prefix+'.'+name for name in re.findall(r'\b(?:fn|record|variant)\s+(\w+)',source)})
    values.update(imports or {})
    return json.dumps(dict(format='lobo-identities/0.1',identities=values),indent=2)+'\n'

def logic(task,kind):
    imports='use model {'+','.join(re.findall(r'\b(?:record|variant)\s+(\w+)',MODELS[task]))+'};\nuse util {reverse};\n'
    if task=='splice':
        stub='fn run(text:Text,start:I64,count:I64,replacement:Text)->Edit<Change> {Err(1)}\n'
        ref='''fn run(text:Text,start:I64,count:I64,replacement:Text)->Edit<Change> {
 let size=Text::length(text);
 if start<0 || start>size {Err(1)} else {
  if count<0 || count>size-start {Err(2)} else {
   let output=Text::concat(Text::concat(Text::slice(text,0,start),replacement),Text::slice(text,start+count,size-start-count));
   Ok(Change {text:output,removed:Text::slice(text,start,count),scalar_delta:Text::length(replacement)-count,byte_delta:Bytes::length(Text::bytes(output))-Bytes::length(Text::bytes(text))})
  }
 }
}
'''
        mutant=ref.replace('count>size-start','count>=size-start')
    elif task=='batch':
        stub='fn convert(sample:Sample)->Outcome<I64> {Ok(sample.raw)}\nfn run(samples:List<Sample>)->Outcome<Summary> {Ok(Summary {values:List::empty<I64>(),total:0,maximum:0})}\n'
        ref='''record Cursor {pending:List<Sample>,output:List<I64>,total:I64,maximum:I64}
fn convert(sample:Sample)->Outcome<I64> {
 if sample.raw<0 {Err(11)} else {if sample.scale<=0 {Err(12)} else {Ok(sample.raw*sample.scale)}}
}
fn run(samples:List<Sample>)->Outcome<Summary> {
 let final=fold(i in List::length(samples),cursor:Cursor=Cursor {pending:samples,output:List::empty<I64>(),total:0,maximum:0}) {
  let amount=try convert(List::head(cursor.pending));
  Cursor {pending:List::tail(cursor.pending),output:List::prepend(amount,cursor.output),total:cursor.total+amount,maximum:if amount>cursor.maximum {amount} else {cursor.maximum}}
 };
 Ok(Summary {values:reverse(final.output),total:final.total,maximum:final.maximum})
}
'''
        mutant=ref.replace('reverse(final.output)','final.output')
    else:
        stub='''record Cursor {pending:List<Entry>,kept:List<Entry>}
fn run(entries:List<Entry>)->Summary {
 let final=fold(i in List::length(entries),cursor:Cursor=Cursor {pending:entries,kept:List::empty<Entry>()}) {
  let item=List::head(cursor.pending);
  Cursor {pending:List::tail(cursor.pending),kept:if item.active {List::prepend(item,cursor.kept)} else {cursor.kept}}
 };Summary {kept:reverse(final.kept),dropped:List::length(entries)-List::length(final.kept)}
}
'''
        ref='''record Cursor {pending:List<Entry>,kept:List<Entry>}
record Seen {pending:List<Entry>,found:Bool}
fn run(entries:List<Entry>)->Summary {
 let final=fold(i in List::length(entries),cursor:Cursor=Cursor {pending:entries,kept:List::empty<Entry>()}) {
  let item=List::head(cursor.pending);
  let later=List::tail(cursor.pending);
  let scanned=fold(j in List::length(later),search:Seen=Seen {pending:later,found:false}) {
   Seen {pending:List::tail(search.pending),found:search.found || List::head(search.pending).key==item.key}
  };
  Cursor {pending:later,kept:if item.active && !scanned.found {List::prepend(item,cursor.kept)} else {cursor.kept}}
 };Summary {kept:reverse(final.kept),dropped:List::length(entries)-List::length(final.kept)}
}
'''
        mutant=ref.replace('item.active && !scanned.found','item.active')
    return 'module logic;\n'+imports+{'starter':stub,'reference':ref,'mutant':mutant}[kind]

def files(task,kind='starter'):
    model='module model;\n'+MODELS[task]+'\n';source=logic(task,kind)
    imports={'model.'+n:'skilldev.'+task+'.model.'+n for n in re.findall(r'\b(?:record|variant)\s+(\w+)',model)}|{'util.reverse':'skilldev.util.reverse'}
    return {'model.lobo':model,'model.lobo.ids.json':identities(model,'skilldev.'+task+'.model'),'util.lobo':UTIL,'util.lobo.ids.json':identities(UTIL,'skilldev.util'),'logic.lobo':source,'logic.lobo.ids.json':identities(source,'skilldev.'+task,imports)}

def oracle(task,a):
    if task=='splice':
        text,start,count,replacement=(a[k] for k in ('text','start','count','replacement'))
        if not 0<=start<=len(text):return dict(case='Err',payload=1)
        if not 0<=count<=len(text)-start:return dict(case='Err',payload=2)
        output=text[:start]+replacement+text[start+count:]
        return dict(case='Ok',payload=dict(text=output,removed=text[start:start+count],scalar_delta=len(output)-len(text),byte_delta=len(output.encode())-len(text.encode())))
    if task=='batch':
        values=[]
        for raw,scale in a['samples']:
            if raw<0:return dict(case='Err',payload=11)
            if scale<=0:return dict(case='Err',payload=12)
            values.append(raw*scale)
        return dict(case='Ok',payload=dict(values=values,total=sum(values),maximum=max(values,default=0)))
    last={key:i for i,(key,value,active) in enumerate(a['entries'])}
    kept=[dict(key=key,value=value,active=active) for i,(key,value,active) in enumerate(a['entries']) if last[key]==i and active]
    return dict(kept=kept,dropped=len(a['entries'])-len(kept))

def cases(task,hidden=False):
    rng=random.Random(20261126+TASKS.index(task));out=[]
    if task=='splice':
        out=[dict(text='Ké🐺Z',start=1,count=2,replacement='λ'),dict(text='',start=0,count=0,replacement='x'),dict(text='hi',start=-1,count=-1,replacement=''),dict(text='hi',start=1,count=2,replacement=''),dict(text='ab',start=2,count=0,replacement='!')]
        if hidden:
            for _ in range(65):
                t=''.join(rng.choice(['a','é','🐺','\u0301','\0','Ω']) for _ in range(rng.randrange(9)))
                out.append(dict(text=t,start=rng.choice([-2,0,1,len(t),len(t)+1,2**63-1,-2**63]),count=rng.choice([-1,0,1,2,len(t),2**63-1]),replacement=rng.choice(['','Q','🐺é','\0'])))
    elif task=='batch':
        out=[dict(samples=[]),dict(samples=[[2,3],[1,4],[0,8]]),dict(samples=[[-1,0]]),dict(samples=[[2,0],[-1,2]]),dict(samples=[[1,1],[4,2]])]
        if hidden:
            out += [dict(samples=[[rng.randrange(41),rng.randrange(1,9)] for _ in range(rng.randrange(12))]) for _ in range(35)]
            out += [dict(samples=[[rng.randrange(-3,10),rng.randrange(-2,5)] for _ in range(rng.randrange(12))]) for _ in range(30)]
    else:
        out=[dict(entries=[]),dict(entries=[[1,3,True],[1,4,True]]),dict(entries=[[1,3,True],[2,8,True],[1,0,False]]),dict(entries=[[3,6,False],[3,7,True]]),dict(entries=[[2,7,True],[1,8,True],[2,9,True]])]
        if hidden:
            out += [dict(entries=[[rng.randrange(-3,6),rng.randrange(-100,101),bool(rng.randrange(2))] for _ in range(rng.randrange(17))]) for _ in range(65)]
    return out

def arguments(task,a):
    if task=='splice':return ['Text:'+a['text'].encode().hex(),'I64:'+str(a['start']),'I64:'+str(a['count']),'Text:'+a['replacement'].encode().hex()]
    values=a['samples' if task=='batch' else 'entries'];kind='Sample' if task=='batch' else 'Entry';atoms=[]
    for item in values:
        atom='I64:'+str(item[0])+',I64:'+str(item[1])
        if task=='dispatch':atom+=',Bool:'+str(item[2]).lower()
        atoms.append(atom)
    return ['List:Count:'+str(len(values))+(','+','.join(atoms) if atoms else '')]
