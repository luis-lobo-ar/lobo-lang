"""Fresh locate/coalesce tasks. No scored examples appear in either skill."""
import importlib.util
from pathlib import Path
import json,random,re
spec=importlib.util.spec_from_file_location('revision_base_corpus',Path(__file__).with_name('development_corpus.py'));OLD=importlib.util.module_from_spec(spec);spec.loader.exec_module(OLD)
UTIL=OLD.UTIL;identities=OLD.identities;TASKS=('locate','coalesce')
MODELS={'locate':'record Hit {index:I64,prefix:Text,suffix:Text,match_bytes:I64}\nvariant Search<T> {Found(T),Missing,Invalid(I64)}','coalesce':'record Span {start:I64,end:I64}\nrecord Summary {spans:List<Span>,points:I64,rejected:I64}'}
REQUIREMENTS={
'locate':'''Implement run(text:Text,needle:Text,start:I64)->Search<Hit>. Search for the FIRST occurrence of the exact scalar sequence needle at an index >=start. Validate start first: outside0..scalar_length(text) gives Invalid(start), even if needle cannot fit. Otherwise no occurrence gives Missing. An empty needle matches exactly at start, including end. Found carries Hit: index is scalar index, prefix is all original text before the hit, suffix is text after the entire matched needle, match_bytes is UTF-8 byte length of needle. Preserve Unicode scalars, NUL and combining marks; no normalization/case folding. text and needle have at most24 scalars; start may be any I64. Guard ranges without overflowing addition. Only run is required; helpers may be added.''',
'coalesce':'''Change run(spans:List<Span>)->Summary. The supplied correct base filters invalid spans and sums each retained inclusive length separately. New behavior: coalesce overlapping OR ADJACENT valid inclusive integer spans into maximal spans, preserving sorted order. Input is sorted by start, at most16 spans; every endpoint lies in0..50. A span with end<start is invalid: count it once in rejected and ignore it entirely, without closing or splitting a pending valid span. A point span start==end is valid. Summary.spans is the maximal coalesced list; points is sum(end-start+1) over that output, counting each covered integer once. Empty input returns empty spans,0points,0rejected. Preserve the existing Cursor/run declarations and IDs, but internal Cursor fields may be changed and helpers added.'''}

def logic(task,kind):
 names=re.findall(r'\b(?:record|variant)\s+(\w+)',MODELS[task]);imports='use model {'+','.join(names)+'};\nuse util {reverse};\n'
 if task=='locate':
  starter='fn run(text:Text,needle:Text,start:I64)->Search<Hit> {Missing}\n'
  reference='''fn run(text:Text,needle:Text,start:I64)->Search<Hit> {
 let size=Text::length(text);
 if start<0 || start>size {Invalid(start)} else {
  let width=Text::length(needle);
  if width>size-start {Missing} else {
   let found=fold(i in size-start-width+1,pick:I64 = -1) {
    if pick>=0 {pick} else {
     let at=start+i;
     if Text::slice(text,at,width)==needle {at} else {-1}
    }
   };
   if found<0 {Missing} else {Found(Hit {index:found,prefix:Text::slice(text,0,found),suffix:Text::slice(text,found+width,size-found-width),match_bytes:Bytes::length(Text::bytes(needle))})}
  }
 }
}
'''
  mutant=reference.replace('if pick>=0 {pick} else {','if false {pick} else {').replace('{at} else {-1}','{at} else {pick}')
 else:
  starter='''record Cursor {pending:List<Span>,kept:List<Span>,points:I64,rejected:I64}
fn run(spans:List<Span>)->Summary {
 let final=fold(i in List::length(spans),cursor:Cursor=Cursor {pending:spans,kept:List::empty<Span>(),points:0,rejected:0}) {
  let item=List::head(cursor.pending);
  if item.end<item.start {Cursor {...cursor,pending:List::tail(cursor.pending),rejected:cursor.rejected+1}} else {
   Cursor {...cursor,pending:List::tail(cursor.pending),kept:List::prepend(item,cursor.kept),points:cursor.points+item.end-item.start+1}
  }
 };
 Summary {spans:reverse(final.kept),points:final.points,rejected:final.rejected}
}
'''
  reference='''record Cursor {pending:List<Span>,kept:List<Span>,open:Bool,low:I64,high:I64,rejected:I64}
record Total {pending:List<Span>,points:I64}
fn run(spans:List<Span>)->Summary {
 let final=fold(i in List::length(spans),cursor:Cursor=Cursor {pending:spans,kept:List::empty<Span>(),open:false,low:0,high:0,rejected:0}) {
  let item=List::head(cursor.pending);
  if item.end<item.start {Cursor {...cursor,pending:List::tail(cursor.pending),rejected:cursor.rejected+1}} else {
   if !cursor.open {Cursor {...cursor,pending:List::tail(cursor.pending),open:true,low:item.start,high:item.end}} else {
    if item.start<=cursor.high+1 {Cursor {...cursor,pending:List::tail(cursor.pending),high:if item.end>cursor.high {item.end} else {cursor.high}}} else {
     Cursor {...cursor,pending:List::tail(cursor.pending),kept:List::prepend(Span {start:cursor.low,end:cursor.high},cursor.kept),low:item.start,high:item.end}
    }
   }
  }
 };
 let backwards=if final.open {List::prepend(Span {start:final.low,end:final.high},final.kept)} else {final.kept};
 let output=reverse(backwards);
 let total=fold(j in List::length(output),state:Total=Total {pending:output,points:0}) {
  let segment=List::head(state.pending);
  Total {pending:List::tail(state.pending),points:state.points+segment.end-segment.start+1}
 };
 Summary {spans:output,points:total.points,rejected:final.rejected}
}
'''
  mutant=reference.replace('item.start<=cursor.high+1','item.start<=cursor.high')
 return 'module logic;\n'+imports+{'starter':starter,'reference':reference,'mutant':mutant}[kind]

def files(task,kind='starter'):
 model='module model;\n'+MODELS[task]+'\n';source=logic(task,kind)
 imports={'model.'+n:'skilldev.'+task+'.model.'+n for n in re.findall(r'\b(?:record|variant)\s+(\w+)',model)}|{'util.reverse':'skilldev.util.reverse'}
 return {'model.lobo':model,'model.lobo.ids.json':identities(model,'skilldev.'+task+'.model'),'util.lobo':UTIL,'util.lobo.ids.json':identities(UTIL,'skilldev.util'),'logic.lobo':source,'logic.lobo.ids.json':identities(source,'skilldev.'+task,imports)}

def oracle(task,a):
 if task=='locate':
  t,n,start=a['text'],a['needle'],a['start']
  if not 0<=start<=len(t):return dict(case='Invalid',payload=start)
  found=t.find(n,start)
  if found<0:return dict(case='Missing',payload=None)
  return dict(case='Found',payload=dict(index=found,prefix=t[:found],suffix=t[found+len(n):],match_bytes=len(n.encode())))
 out=[];rejected=0
 for start,end in a['spans']:
  if end<start:rejected+=1;continue
  if out and start<=out[-1]['end']+1:out[-1]['end']=max(out[-1]['end'],end)
  else:out.append(dict(start=start,end=end))
 return dict(spans=out,points=sum(s['end']-s['start']+1 for s in out),rejected=rejected)

def cases(task,hidden=False):
 rng=random.Random(20261126+(0 if task=='locate' else 1))
 if task=='locate':
  out=[dict(text='éababa',needle='aba',start=0),dict(text='éababa',needle='aba',start=2),dict(text='',needle='',start=0),dict(text='ab',needle='z',start=-1),dict(text='ab',needle='z',start=0)]
  if hidden:
   out += [dict(text='a'*24,needle='aa',start=23),dict(text='x',needle='',start=2**63-1),dict(text='x',needle='',start=-2**63),dict(text='a\0e\u0301',needle='\0e',start=0),dict(text='a'*24,needle='a'*24,start=0),dict(text='🐺é',needle='é',start=1)]
   for _ in range(59):
    t=''.join(rng.choice(['a','b','é','🐺','\0','\u0301']) for _ in range(rng.randrange(25)));n=rng.choice(['','a','é','🐺',t[:rng.randrange(len(t)+1)],'zz'])
    out.append(dict(text=t,needle=n,start=rng.choice([-1,0,len(t),max(0,len(t)-1),len(t)+1,rng.randrange(len(t)+1)])))
 else:
  out=[dict(spans=[]),dict(spans=[[0,2],[3,3],[7,8]]),dict(spans=[[0,8],[1,3],[2,1]]),dict(spans=[[1,0],[3,3]]),dict(spans=[[0,2],[1,0],[2,4]])]
  if hidden:
   out += [dict(spans=[[0,0],[1,1],[50,50]]),dict(spans=[[0,50]]),dict(spans=[[i,i] for i in range(16)]),dict(spans=[[0,50]]*16),dict(spans=[[i,0] for i in range(1,17)])]
   for _ in range(60):out.append(dict(spans=sorted([[rng.randrange(51),rng.randrange(51)] for _ in range(rng.randrange(17))])))
 return out

def arguments(task,a):
 if task=='locate':return ['Text:'+a['text'].encode().hex(),'Text:'+a['needle'].encode().hex(),'I64:'+str(a['start'])]
 return ['List:Count:'+str(len(a['spans']))+''.join(',I64:'+str(start)+',I64:'+str(end) for start,end in a['spans'])]
