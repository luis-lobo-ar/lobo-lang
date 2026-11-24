"""Fresh development-frozen tasks for the response/reference/graph study."""
import importlib.util
import json
from pathlib import Path
import re

spec=importlib.util.spec_from_file_location('previous_context',Path(__file__).with_name('context.py'))
C=importlib.util.module_from_spec(spec);spec.loader.exec_module(C)
TOPICS=('state','ranges','nominal','propagation','callbacks','contracts')
NAMES={'ledger':{'valid_amount':'permitted','step':'transition'},'scheduler':{'ready':'available','fits':'affordable','choose':'select_job'},'frames':{'read_header':'header_at','fits':'length_ok','checksum':'digest'}}


def rename(text,family):return re.sub(r'\b('+'|'.join(NAMES[family])+r')\b',lambda m:NAMES[family][m[0]],text)


def format_logic(family,language,mutant=False):
    text=rename(C.BASE.reference(family,language),family)
    if family=='ledger':
        text=text.replace('<=1000000','<=83')
        if mutant:text=text.replace('value>=0','value>=1').replace('0<=value','1<=value')
    elif family=='scheduler':
        if mutant:text=text.replace('cost<=remaining','cost+3<=remaining')
    else:
        text=text.replace('total:I64=0','total:I64=13').replace('fold(0u8,','fold(13u8,').replace('sum(data)%256','(13+sum(data))%256')
        if mutant:text=text.replace('length<=limit','length<=limit && length!=3' if language!='python' else 'length<=limit and length!=3')
    return text


def format_sources(family,language):
    files=C.BASE.sources(family,language);suffix={'lobo':'lobo','rust':'rs','python':'py'}[language];files['logic.'+suffix]=format_logic(family,language)
    if language=='lobo':
        ids=json.loads(files['logic.lobo.ids.json']);ids['identities']={NAMES[family].get(k,k):v for k,v in ids['identities'].items()};files['logic.lobo.ids.json']=json.dumps(ids,indent=2)+'\n'
    return files


def format_inputs(family):
    return {
      'ledger':[{'initial':15,'events':[[0,4],[2,1],[1,0],[0,2],[3,0],[1,7]]},{'initial':2,'events':[[0,83],[0,1],[1,84],[1,83]]},{'initial':17,'events':[[2,8],[2,0],[1,3],[3,9],[1,17]]},{'initial':5,'events':[[1,6],[0,-3],[1,0],[0,83]]},{'initial':1,'events':[[0,84],[0,82],[1,83]]},{'initial':8,'events':[[0,2],[1,4],[0,3],[1,9]]}],
      'scheduler':[{'jobs':[{'id':2,'cost':1,'deps':[]},{'id':0,'cost':3,'deps':[]},{'id':1,'cost':1,'deps':[0]}],'budget':4},{'jobs':[{'id':0,'cost':2,'deps':[2]},{'id':2,'cost':1,'deps':[1]},{'id':1,'cost':0,'deps':[]}],'budget':3},{'jobs':[{'id':1,'cost':0,'deps':[1]},{'id':0,'cost':0,'deps':[]}],'budget':1},{'jobs':[{'id':0,'cost':6,'deps':[]},{'id':1,'cost':3,'deps':[]},{'id':2,'cost':0,'deps':[1]}],'budget':3},{'jobs':[{'id':1,'cost':1,'deps':[0]},{'id':0,'cost':1,'deps':[1]}],'budget':7},{'jobs':[{'id':1,'cost':2,'deps':[]},{'id':0,'cost':2,'deps':[]},{'id':2,'cost':1,'deps':[0,0]}],'budget':3}],
      'frames':[{'data':'000d','limit':4},{'data':'0301020313','limit':4},{'data':'01041102050618','limit':3},{'data':'01041102050619','limit':3},{'data':'05','limit':4},{'data':'0104110301','limit':4}]
    }[family]


def format_oracle(family,args,mutant=False):
    if family=='ledger':
        adjusted=json.loads(json.dumps(args));adjusted['events']=[[k,-1 if k<2 and not (int(mutant)<=a<=83) else a] for k,a in args['events']]
        return C.BASE.oracle(family,adjusted)
    if family=='scheduler':
        adjusted=dict(args,budget=args['budget']-(3 if mutant else 0));return C.BASE.oracle(family,adjusted)
    data=bytes.fromhex(args['data']);cursor=0;frames=[];error=0
    while cursor<len(data):
        n=data[cursor];end=cursor+n+2
        if n>args['limit'] or (mutant and n==3):error=3
        elif end>len(data):error=1
        elif (13+sum(data[cursor+1:end-1]))%256!=data[end-1]:error=2
        else:frames.append(data[cursor+1:end-1].hex());cursor=end;continue
        break
    return dict(frames=frames,consumed=cursor,error=error)


def format_correct(family):
    witness={'ledger':{'initial':17,'events':[[1,5],[1,13],[0,84]]},'scheduler':{'jobs':[{'id':0,'cost':2,'deps':[2]},{'id':1,'cost':7,'deps':[]},{'id':2,'cost':1,'deps':[]}],'budget':3},'frames':{'data':'0107140308090a28','limit':4}}[family]
    counter={'ledger':{'initial':3,'events':[[1,0]]},'scheduler':{'jobs':[{'id':0,'cost':2,'deps':[]},{'id':1,'cost':3,'deps':[]}],'budget':5},'frames':{'data':'0301020313','limit':4}}[family]
    return dict(predictions=[format_oracle(family,x) for x in format_inputs(family)],witness=witness,direct_calls={'ledger':['transition'],'scheduler':['select_job'],'frames':['header_at','length_ok','digest']}[family],impacted={'ledger':['permitted','transition','run'],'scheduler':['affordable','select_job','run'],'frames':['length_ok','run']}[family],fault_location={'ledger':'permitted','scheduler':'affordable','frames':'length_ok'}[family],counterexample=counter)


def format_target(family):return {'ledger':dict(balance=12,rejected=2,peak=17),'scheduler':dict(order=[2,0],spent=3,blocked=1),'frames':dict(frames=['07','08090a'],consumed=8,error=0)}[family]


def format_grade(family,answer):
    a=answer if isinstance(answer,dict) else {};pred=a.get('predictions');checks={}
    for i,x in enumerate(format_inputs(family)):checks['prediction_'+str(i)]=isinstance(pred,list) and len(pred)==6 and C.BASE.equal(pred[i],format_oracle(family,x))
    w=a.get('witness');checks['witness']=C.BASE.valid_input(family,w) and all(C.BASE.equal(format_oracle(family,w)[k],v) for k,v in format_target(family).items())
    wanted=format_correct(family)
    for key in ('direct_calls','impacted'):
        v=a.get(key);checks[key]=isinstance(v,list) and all(type(x)is str for x in v) and len(v)==len(wanted[key]) and set(v)==set(wanted[key])
    checks['fault_location']=a.get('fault_location')==wanted['fault_location'];v=a.get('counterexample');checks['counterexample']=C.BASE.valid_input(family,v) and format_oracle(family,v)!=format_oracle(family,v,True)
    return checks


def identities(source,prefix):
    names=re.findall(r'\b(?:fn|record|variant)\s+(\w+)',source)
    return json.dumps({'format':'lobo-identities/0.1','identities':{'module':prefix,**{n:prefix+'.'+n for n in names}}},indent=2)+'\n'


def micro(topic,form):
    n=5+form;seed=4+3*form;trigger=2+form
    if topic=='state':
        head='record Cursor { pending: List<I64>, total: I64 }\nrecord State { index: I64, total: I64 }\n'
        bodies=[f'fold(i in {n}, total:I64={seed}) {{ total+i+1 }}',f'fold(i in 0, total:I64={seed}) {{ 1/0 }}',f'let final=while(state:State=State {{index:0,total:{seed}}};state.index<{n}) {{ State {{index:state.index+1,total:state.total+state.index}} }}; final.total',f'List::head(List::prepend({n},List::prepend({seed},List::empty<I64>())))',f'List::head(List::tail(List::prepend({seed},List::prepend({n},List::empty<I64>()))))',f'let final=fold(i in 3,cursor:Cursor=Cursor {{pending:List::prepend({n},List::prepend({seed},List::prepend(2,List::empty<I64>()))),total:0}}) {{Cursor {{pending:List::tail(cursor.pending),total:10*cursor.total+List::head(cursor.pending)}}}}; final.total']
        expected=[seed+n*(n+1)//2,seed,seed+n*(n-1)//2,n,n,100*n+10*seed+2]
        admissions=[('fn candidate()->I64 {List::head(List::empty<I64>())}',True),('fn candidate()->I64 {fold(i in 0,total:I64=0) {true}}',False),('fn candidate()->I64 {while(state:I64=0;false) {1/0}}',True),('fn candidate()->I64 {fold(i in 2,total:I64=0) {let total=1;total}}',False)]
    elif topic=='ranges':
        head='';text='"Aé猫Z!"';start=1+form;count=2+form
        bodies=[f'Text::length({text})+{form}',f'Bytes::length(Text::bytes({text}))+{2*form}',f'Text::at({text},{start})',f'Bytes::length(Bytes::slice(Text::bytes({text}),{start},{count}))',f'Bytes::at(Bytes::slice(Text::bytes({text}),{start},{count}),0)',f'Text::length(Text::slice({text},1,{count}))']
        expected=[5+form,8+2*form,ord('Aé猫Z!'[start]),count,'Aé猫Z!'.encode()[start],count]
        admissions=[('fn candidate()->I64 {Bytes::length(Bytes::slice(Bytes::empty(),0,true))}',False),('fn candidate()->I64 {Text::length(Text::from_bytes(Bytes::empty()))}',True),('fn candidate()->I64 {Bytes::at(Bytes::empty(),7)}',True),('fn candidate()->I64 {Bytes::length(Text::bytes(Bytes::empty()))}',False)]
    elif topic=='nominal':
        head=f'record Box<T> {{value:T}}\nrecord Duo {{left:I64,right:I64}}\nvariant Maybe<T> {{Missing,Found(T)}}\nfn take(item:Maybe<I64>)->I64 {{match item {{Missing=>-{seed},Found(value)=>value+{seed}}}}}\nfn identity<T>(value:T)->T {{value}}\n'
        bodies=[f'let original=Duo {{left:{seed},right:{n}}}; let updated=Duo {{...original,left:{seed+2}}}; 10*updated.left+updated.right',f'let original=Duo {{left:{seed},right:{n}}}; let copied=Duo {{...original}}; copied.right',f'take(Found({n}))','take(Missing)',f'let boxed=Box {{value:{n}}}; identity(boxed.value)',f'let wrapped:Maybe<Box<I64>>=Found(Box {{value:{seed}}}); match wrapped {{Missing=>0,Found(inner)=>inner.value}}']
        expected=[10*(seed+2)+n,n,n+seed,-seed,n,seed]
        admissions=[('fn candidate()->I64 {let item=Missing;1}',False),('fn candidate()->I64 {let item:Maybe<I64>=Missing;take(item)}',True),('record Same {left:I64,right:I64} fn candidate()->Duo {Same {left:1,right:2}}',False),('fn candidate()->Box<I64> {Box {value:3}}',True)]
    elif topic=='propagation':
        head=f'variant Result<T> {{Ok(T),Err(I64)}}\nfn gate(value:I64)->Result<I64> {{if value=={trigger} {{Err({seed})}} else {{Ok(value+1)}}}}\nfn decode(item:Result<I64>)->I64 {{match item {{Ok(value)=>value,Err(code)=>-code}}}}\nfn work(start:I64)->Result<I64> {{let value=try gate(start);Ok(value*3)}}\nfn looped()->Result<I64> {{let amount=fold(i in {trigger+2},total:I64=0) {{let value=try gate(i);total+value}};Ok(amount)}}\nfn nested()->Result<I64> {{if true {{let value=try gate({trigger});Ok(value)}} else {{Ok(8)}}}}\n'
        bodies=['decode(work(0))',f'decode(work({trigger}))','decode(looped())',f'decode(work({trigger}))+11','decode(nested())',f'let item:Result<I64>=Ok({n});decode(item)']
        expected=[3,-seed,-seed,11-seed,-seed,n]
        admissions=[('fn candidate()->I64 {try gate(0)}',False),('fn candidate()->Result<I64> {let value=try gate(0);Ok(value)}',True),('fn candidate()->Result<I64> {Ok(try(gate(0)))}',False),(f'fn candidate()->Result<I64> requires {{false}} {{Err({seed})}}',True)]
    elif topic=='callbacks':
        head=f'fn plus(value:I64)->I64 {{value+{seed}}}\nfn apply(callback:Fn(I64)->I64,value:I64)->I64 {{callback(value)}}\nfn observed(value:I64)->I64 actions {{study.read}} {{value*2}}\nfn invoke(callback:Fn(I64) actions {{study.read}}->I64,value:I64)->I64 actions {{study.read}} {{callback(value)}}\nfn combine(context:I64,value:I64)->I64 {{context-value}}\nfn with_context(context:I64,callback:Fn(I64,I64)->I64,value:I64)->I64 {{callback(context,value)}}\n'
        bodies=[f'apply(plus,{n})',f'let callback:Fn(I64)->I64=&plus;callback({seed})',f'invoke(observed,{n})',f'let callback:Fn(I64) actions {{study.read}}->I64=&observed;callback({seed})',f'with_context({seed},combine,{n})',f'apply(plus,apply(plus,{n}))']
        expected=[n+seed,2*seed,2*n,2*seed,seed-n,n+2*seed]
        admissions=[('fn candidate()->I64 {invoke(observed,1)}',False),('fn candidate()->I64 actions {study.read} {invoke(observed,1)}',True),('fn candidate()->Fn(I64) actions {study.read}->I64 {&plus}',False),('fn candidate()->I64 {let callback:Fn(I64) actions {study.read}->I64=&observed;1}',True)]
    else:
        head=f'fn guarded(value:I64)->I64 requires {{value>=0}} ensures {{result>=value}} {{value+{seed}}}\nfn consistent(value:I64)->I64 ensures {{result==value+{seed}}} {{value+{seed}}}\n'
        bodies=[f'guarded({n})',f'if false {{guarded(-1)}} else {{{seed}}}',f'if false && guarded(-1)>0 {{0}} else {{{n}}}',f'fold(i in 0,total:I64={n}) {{guarded(-1)}}',f'consistent({seed})',f'if true || 1/0==0 {{{n+1}}} else {{0}}']
        expected=[n+seed,seed,n,n,2*seed,n+1]
        admissions=[('fn candidate()->I64 requires {1} {1}',False),('fn candidate()->I64 ensures {false} {1}',True),('fn bad<T>(value:T)->I64 {value+1} fn candidate()->I64 {1}',False),('fn candidate()->I64 {if true {3} else {1/0}}',True)]
    if form:
        admissions={
          'state':[('fn candidate()->I64 {List::length(List::tail(List::empty<I64>()))}',True),('fn candidate()->I64 {fold(i in true,total:I64=0) {total+i}}',False),('fn candidate()->I64 {while(state:I64=0;true) {state+1}}',True),('fn candidate(value:I64)->I64 {let value=3;value}',False)],
          'ranges':[('fn candidate()->I64 {Text::length(Bytes::empty())}',False),('fn candidate()->I64 {Bytes::length(Bytes::concat(Bytes::empty(),Bytes::empty()))}',True),('fn candidate()->I64 {Text::length(Text::slice("a",0,2))}',True),('fn candidate()->I64 {Bytes::length(Bytes::push(Bytes::empty(),"x"))}',False)],
          'nominal':[('fn candidate()->Maybe<I64> {Found(Box {value:3})}',False),('fn candidate()->Maybe<Box<I64>> {Found(Box {value:9})}',True),('fn candidate()->Duo {let original=Box {value:1};Duo {...original}}',False),('fn candidate()->I64 {let item:Maybe<I64>=Missing();take(item)}',True)],
          'propagation':[('variant Other {Ok(I64),Err(Text)} fn candidate()->Other {Ok(try gate(0))}',False),('fn candidate()->Result<I64> {try gate(0);Ok(7)}',True),('fn candidate()->Result<I64> ensures {(try result)>0} {Ok(1)}',False),('fn try(value:I64)->I64 {value+2} fn candidate()->I64 {try(3)}',True)],
          'callbacks':[('fn candidate()->Fn(I64)->I64 {&observed}',False),('fn candidate()->Fn(I64) actions {study.read}->I64 {&observed}',True),('fn candidate()->I64 actions {study.write} {invoke(observed,1)}',False),('fn candidate()->I64 actions {study.read,study.write} {invoke(observed,1)}',True)],
          'contracts':[('fn leaf()->I64 actions {study.audit} {1} fn candidate()->I64 actions {study.audit} requires {leaf()>0} {1}',False),('fn candidate()->I64 requires {false} {1/0}',True),('fn candidate()->I64 {if false {missing()} else {1}}',False),('fn candidate()->I64 ensures {result==2} {3}',True)]
        }[topic]
    entries='\n'.join('fn p'+str(i)+'()->I64'+(' actions {study.read}' if topic=='callbacks' and i in (2,3) else '')+' {'+body+'}' for i,body in enumerate(bodies))
    source='module questions;\n'+head+entries+'\n'
    result=dict(source=source,head='module questions;\n'+head,expected=expected,admissions=[dict(source='module questions;\n'+head+code,accepted=ok) for code,ok in admissions])
    if form:
        # Distinct instances in stage B, including static-admission snippets with unchanged type laws.
        declarations=set(re.findall(r'\b(?:fn|record|variant)\s+(\w+)',source+'\n'+'\n'.join(a['source'] for a in result['admissions'])))
        names={name:name+'_b' for name in declarations if not re.fullmatch('p[0-5]',name) and name!='try'}
        transform=lambda text:re.sub(r'\b('+'|'.join(names)+r')\b',lambda m:names[m[0]],text)
        result['source']=transform(result['source']);result['head']=transform(result['head'])
        for admission in result['admissions']:admission['source']=transform(admission['source'])
    return result


def micro_correct(topic,form):
    case=micro(topic,form);return dict(predictions=case['expected'],admissions=[a['accepted'] for a in case['admissions']])


def micro_mutant(topic,form):
    source=micro(topic,form)['source'];n=5+form;seed=4+3*form;start=1+form;count=2+form
    before,after,entry={
      'state':(f'fold(i in {n}, total:I64={seed})',f'fold(i in {n+1}, total:I64={seed})',0),
      'ranges':(f'Bytes::slice(Text::bytes("Aé猫Z!"),{start},{count})',f'Bytes::slice(Text::bytes("Aé猫Z!"),{start},{count+1})',3),
      'nominal':(f'left:{seed+2}',f'left:{seed+3}',0),
      'propagation':('Ok(value*3)','Ok(value*4)',0),
      'callbacks':(f'value+{seed}',f'value+{seed}+1',0),
      'contracts':(f'value+{seed}',f'value+{seed}+1',0)}[topic]
    if before not in source:raise ValueError('mutation target missing')
    return source.replace(before,after,1),entry
