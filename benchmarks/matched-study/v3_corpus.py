"""Fresh harder application changes; specification-derived independent oracles."""
import importlib.util,json,re
from pathlib import Path
HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location('v3_old_corpus',HERE/'realistic_corpus.py');OLD=importlib.util.module_from_spec(spec);spec.loader.exec_module(OLD)
TASKS=('feed','reservation')
EVENTS=[dict(id=9,kind='info',message='Tea "夜"',published=True),dict(id=3,kind='audit',message='Line\nBack\\',published=True),dict(id=7,kind='info',message='hidden',published=False),dict(id=1,kind='info',message='start',published=True),dict(id=12,kind='audit',message='',published=True),dict(id=9223372036854775807,kind='info',message='last',published=True)]
REQUIREMENTS={
'feed':'''Change the native event-feed HTTP controller and query policy. Preserve declarations/IDs. Exact GET and HEAD only; any other method =>405 {"error":"method not allowed"} before route/query validation. /health =>200 {"status":"ready"}. Only /api/events or /api/events?<parameters> is the feed route; other paths =>404 {"error":"not found"}. Feed query: optional after,limit,kind in ANY order, each at most once. Defaults after=0,limit=2,kind="" (all kinds). after is canonical decimal0..I64_MAX, limit canonical decimal1..3. No signs, leading zeros except literal0, overflow, percent decoding, empty query/segment/name/value, unknown or duplicate keys, trailing &, or extra '='; malformed feed query =>400 {"error":"bad query"}. kind is exactly info or audit when present. Paths are ASCII, at most128 characters. Only published model events with id>after and matching kind qualify. Sort by ascending id regardless of model order, then take at most limit. Return200 with exactly {items:[{id:<JSON integer>,kind:<original text>,message:<original text>}],count:<returned length>,next:<last returned id if MORE matching published events remain, otherwise JSON null>}. Count/next depend on filtering BEFORE pagination. Do not increment cursor (I64_MAX is valid); an exhausted page is empty,count0,nextnull. Preserve Unicode/escaping. HEAD has GET status/headers/representation byte length and no wire body; fixed transport owns this and Allow: GET, HEAD on405. read_query(path) must distinguish Ready(Page),Bad,Missing. Existing model/lib/app are read-only; JSON builders/helper rendering are supplied. The working legacy scaffold ignores query fields, emits model order and never emits a next cursor. Implement both modules; no transport change.''',
'reservation':'''Change the native inventory reservation service and cart policy. apply borrows Native::Database; never close it. Request has two lines (sku1,qty1),(sku2,qty2), plus opaque key. key length1..24 Unicode scalars, case-sensitive, quotes/NUL allowed. A zero-quantity line is ignored regardless of its SKU. Every nonzero line must have sku>0 and qty1..1000000. At least one line required. Normalize BEFORE ledger lookup: sort active lines by SKU, merge duplicate SKUs by summing quantities with total<=1000000; missing second canonical line is sku0,qty0. Guard overflow before addition. Invalid input =>Ok Receipt(false,false,"invalid"), with no change. Schema inventory(sku INTEGER PRIMARY KEY,available INTEGER NOT NULL CHECK(available>=0 AND available<=1000000)); requests(key TEXT PRIMARY KEY CHECK(key!='reject'),sku1 INTEGER,qty1 INTEGER,sku2 INTEGER,qty2 INTEGER). Key 'reject' is valid input but causes a deliberate REAL engine ledger INSERT constraint failure, which must propagate as the original SqlError and roll back reservations. Existing key with equal CANONICAL payload =>Ok(false,true,"replay") even if stocks would now be insufficient; unequal canonical payload=>Ok(false,false,"conflict"). For new key: missing any active SKU=>"missing"; otherwise strict integer decoding, then insufficient ANY requested quantity=>"stock" (both false,false). Missing takes priority over decode/stock by checking row presence first. Otherwise atomically decrement ALL canonical stocks, INSERT bound key+payload, commit, returnOk(true,false,"reserved"). On any SQL/decode error, roll back all changes and return the original error. No pending transaction on ANY return. All SQL values must be bound. Fixed app probes transaction state BEFORE close; Audit.clean must always be true. Host tests inspect inventory and ledger after EACH operation on the same file. Supplied helper params creates five bound cells in canonical ledger order; preserve helper/ID contracts. The legacy scaffold leaves duplicates/order unchanged, handles only the first stock line and has no idempotency/ledger; its early try propagation also leaves transaction open on an error. No schema edits, concurrency or dependencies. All scalar arguments may be any I64; input validation must avoid overflow.'''}
HTTP_MODEL='module model; record Event {id:I64,kind:Text,message:Text,published:Bool}\nfn events()->List<Event> {'
expr='List::empty<Event>()'
for event in reversed(EVENTS):expr='List::prepend(Event {id:'+str(event['id'])+',kind:'+json.dumps(event['kind'])+',message:'+json.dumps(event['message'],ensure_ascii=False)+',published:'+str(event['published']).lower()+'},'+expr+')'
HTTP_MODEL+=expr+'}\n'
HTTP_POLICY='''module policy;
record Page {after:I64,limit:I64,kind:Text}
variant Query {Ready(Page),Bad,Missing}
variant Number {Valid(I64),Invalid}
record Digits {value:I64,valid:Bool}
record Fields {after:I64,limit:I64,kind:Text,seen_after:Bool,seen_limit:Bool,seen_kind:Bool,valid:Bool}
record Scan {start:I64,fields:Fields}
fn natural(token:Text)->Number {
 let bytes=Text::bytes(token);
 if Bytes::length(bytes)==0 || (Bytes::length(bytes)>1 && Bytes::at(bytes,0)==48) {Invalid} else {
  let done=fold(i in Bytes::length(bytes),state:Digits=Digits {value:0,valid:true}) {
   let digit=Bytes::at(bytes,i)-48;
   if !state.valid || digit<0 || digit>9 || state.value>(9223372036854775807-digit)/10 {Digits {value:0,valid:false}} else {Digits {value:state.value*10+digit,valid:true}}
  };
  if done.valid {Valid(done.value)} else {Invalid}
 }
}
fn field_value(part:Text,fields:Fields)->Fields {
 let bytes=Text::bytes(part);
 let split=while(i:I64=0;i<Bytes::length(bytes) && Bytes::at(bytes,i)!=61) {i+1};
 if split==0 || split>=Bytes::length(bytes)-1 {Fields {...fields,valid:false}} else {
  let name=Text::slice(part,0,split);let value=Text::slice(part,split+1,Text::length(part)-split-1);
  if name=="kind" && !fields.seen_kind && (value=="info" || value=="audit") {Fields {...fields,kind:value,seen_kind:true}} else {
   if name=="after" && !fields.seen_after {
    match natural(value) {Invalid=>Fields {...fields,valid:false},Valid(n)=>Fields {...fields,after:n,seen_after:true}}
   } else {
    if name=="limit" && !fields.seen_limit {
     match natural(value) {Invalid=>Fields {...fields,valid:false},Valid(n)=>if n>=1 && n<=3 {Fields {...fields,limit:n,seen_limit:true}} else {Fields {...fields,valid:false}}}
    } else {Fields {...fields,valid:false}}
   }
  }
 }
}
fn read_query(path:Text)->Query {
 if path=="/api/events" {Ready(Page {after:0,limit:2,kind:""})} else {
  if Text::length(path)<12 || Text::slice(path,0,12)!="/api/events?" {Missing} else {
   let query=Text::slice(path,12,Text::length(path)-12);let bytes=Text::bytes(query);
   let initial=Fields {after:0,limit:2,kind:"",seen_after:false,seen_limit:false,seen_kind:false,valid:true};
   let done=fold(i in Bytes::length(bytes)+1,state:Scan=Scan {start:0,fields:initial}) {
    if i==Bytes::length(bytes) || Bytes::at(bytes,i)==38 {
     Scan {start:i+1,fields:field_value(Text::slice(query,state.start,i-state.start),state.fields)}
    } else {state}
   };
   if done.fields.valid {Ready(Page {after:done.fields.after,limit:done.fields.limit,kind:done.fields.kind})} else {Bad}
  }
 }
}
'''
HTTP_LOGIC='''module logic;
use model {Event,events}; use policy {Page,Query,read_query}; use http {Request,Response,json_response};
use json {Json,JsonArray,object,field,array,append};
record Items {values:JsonArray,count:I64,last:I64,more:Bool}
fn event_json(event:Event)->Json {Json::Object(field(field(field(object(),"id",Json::Integer(event.id)),"kind",Json::String(event.kind)),"message",Json::String(event.message)))}
fn insert(event:Event,sorted:List<Event>)->List<Event> {
 if List::length(sorted)==0 {List::prepend(event,sorted)} else {
  if event.id<List::head(sorted).id {List::prepend(event,sorted)} else {List::prepend(List::head(sorted),insert(event,List::tail(sorted)))}
 }
}
fn ordered(values:List<Event>)->List<Event> {
 if List::length(values)==0 {values} else {insert(List::head(values),ordered(List::tail(values)))}
}
fn page_json(page:Page)->Json {
 let values=ordered(events());
 let done=fold(i in List::length(values),state:Items=Items {values:array(),count:0,last:0,more:false}) {
  let event=List::head(fold(j in i,rest:List<Event> =values) {List::tail(rest)});
  if event.published && event.id>page.after && (page.kind=="" || event.kind==page.kind) {
   if state.count<page.limit {Items {values:append(state.values,event_json(event)),count:state.count+1,last:event.id,more:state.more}} else {Items {...state,more:true}}
  } else {state}
 };
 Json::Object(field(field(field(object(),"items",Json::Array(done.values)),"count",Json::Integer(done.count)),"next",if done.more {Json::Integer(done.last)} else {Json::Null}))
}
fn handle(request:Request)->Response {
 if request.method!="GET" && request.method!="HEAD" {json_response(405,json {"error":"method not allowed"})} else {
  if request.path=="/health" {json_response(200,json {"status":"ready"})} else {
   match read_query(request.path) {Missing=>json_response(404,json {"error":"not found"}),Bad=>json_response(400,json {"error":"bad query"}),Ready(page)=>json_response(200,page_json(page))}
  }
 }
}
'''
SQL_MODEL='module model; record Line {sku:I64,qty:I64} record Cart {first:Line,second:Line} record Receipt {applied:Bool,replay:Bool,reason:Text}\n'
SQL_POLICY='''module policy;
use model {Line,Cart}; use sql {Cell,params1};
variant Plan {Ready(Cart),Invalid}
fn plan(sku1:I64,qty1:I64,sku2:I64,qty2:I64,key:Text)->Plan {
 if Text::length(key)<1 || Text::length(key)>24 || qty1<0 || qty1>1000000 || qty2<0 || qty2>1000000 || (qty1>0 && sku1<=0) || (qty2>0 && sku2<=0) || (qty1==0 && qty2==0) {Invalid} else {
  let a=Line {sku:sku1,qty:qty1};let b=Line {sku:sku2,qty:qty2};let empty=Line {sku:0,qty:0};
  if qty1==0 {Ready(Cart {first:b,second:empty})} else {
   if qty2==0 {Ready(Cart {first:a,second:empty})} else {
    if sku1==sku2 {
     if qty1>1000000-qty2 {Invalid} else {Ready(Cart {first:Line {sku:sku1,qty:qty1+qty2},second:empty})}
    } else {if sku1<sku2 {Ready(Cart {first:a,second:b})} else {Ready(Cart {first:b,second:a})}}
   }
  }
 }
}
fn params(key:Text,cart:Cart)->List<Cell> {List::prepend(Cell::String(key),List::prepend(Cell::Integer(cart.first.sku),List::prepend(Cell::Integer(cart.first.qty),List::prepend(Cell::Integer(cart.second.sku),params1(Cell::Integer(cart.second.qty))))))}
'''
SQL_LOGIC='''module logic;
use model {Line,Cart,Receipt}; use policy {Plan,plan,params};
use sql {Cell,Row,SqlResult,SqlError,query,execute,begin,commit,rollback,params1,params2,integer_at,expect_columns};
fn receipt(applied:Bool,replay:Bool,reason:Text)->Receipt {Receipt {applied:applied,replay:replay,reason:reason}}
fn perform(db:Native::Database,cart:Cart,key:Text)->SqlResult<Receipt>
actions {db.execute,db.read,db.close}
{
 let seen=try query(db,"SELECT sku1,qty1,sku2,qty2 FROM requests WHERE key=?1",params1(Cell::String(key)));
 if List::length(seen)>0 {
  let row=List::head(seen);try expect_columns(row,4);
  let same=try integer_at(row,0)==cart.first.sku && try integer_at(row,1)==cart.first.qty && try integer_at(row,2)==cart.second.sku && try integer_at(row,3)==cart.second.qty;
  Ok(receipt(false,same,if same {"replay"} else {"conflict"}))
 } else {
  let first=try query(db,"SELECT available FROM inventory WHERE sku=?1",params1(Cell::Integer(cart.first.sku)));
  let second:SqlResult<List<Row>> =if cart.second.qty==0 {Ok(List::empty<Row>())} else {query(db,"SELECT available FROM inventory WHERE sku=?1",params1(Cell::Integer(cart.second.sku)))};
  let others=try second;
  if List::length(first)==0 || (cart.second.qty>0 && List::length(others)==0) {Ok(receipt(false,false,"missing"))} else {
   let available=try integer_at(List::head(first),0);
   let remaining:SqlResult<I64> =if cart.second.qty==0 {Ok(0)} else {integer_at(List::head(others),0)};
   let other_available=try remaining;
   if available<cart.first.qty || other_available<cart.second.qty {Ok(receipt(false,false,"stock"))} else {
    try execute(db,"UPDATE inventory SET available=available-?1 WHERE sku=?2",params2(Cell::Integer(cart.first.qty),Cell::Integer(cart.first.sku)));
    let updated=if cart.second.qty>0 {try execute(db,"UPDATE inventory SET available=available-?1 WHERE sku=?2",params2(Cell::Integer(cart.second.qty),Cell::Integer(cart.second.sku)));true} else {true};
    try execute(db,"INSERT INTO requests(key,sku1,qty1,sku2,qty2) VALUES(?1,?2,?3,?4,?5)",params(key,cart));
    Ok(receipt(true,false,"reserved"))
   }
  }
 }
}
fn apply(db:Native::Database,sku1:I64,qty1:I64,sku2:I64,qty2:I64,key:Text)->SqlResult<Receipt>
actions {db.execute,db.read,db.close,db.transaction}
{
 match plan(sku1,qty1,sku2,qty2,key) {
  Invalid=>Ok(receipt(false,false,"invalid")),
  Ready(cart)=> {
   try begin(db);let outcome=perform(db,cart,key);
   match outcome {Err(error)=>{let ignored=rollback(db);Err(error)},Ok(value)=>{try commit(db);Ok(value)}}
  }
 }
}
'''
SQL_APP='''module app;
use sql {SqlResult,open,close,begin,rollback}; use logic {apply}; use model {Receipt};
record Audit {outcome:SqlResult<Receipt>,clean:Bool}
fn main(sku1:I64,qty1:I64,sku2:I64,qty2:I64,key:Text)->SqlResult<Audit>
actions {native.authorize,db.open,db.close,db.transaction,db.execute,db.read}
{
 let db=try open(Native::sqlite_capability());let outcome=apply(db,sku1,qty1,sku2,qty2,key);
 let probe=begin(db);let clean=match probe {Err(error)=>false,Ok(done)=>{let ignored=rollback(db);true}};
 try close(db);Ok(Audit {outcome:outcome,clean:clean})
}
'''

def sources(task):return OLD.sources('catalog' if task=='feed' else 'transfer')
def files(task,kind='starter'):
 if task=='feed':
  texts=dict(app=OLD.HTTP_APP,model=HTTP_MODEL,logic=HTTP_LOGIC,policy=HTTP_POLICY)
  if kind=='starter':
   texts['policy']=texts['policy'].replace(HTTP_POLICY[HTTP_POLICY.index('fn field_value'):HTTP_POLICY.index('fn read_query')],'fn field_value(part:Text,fields:Fields)->Fields {fields}\n').replace('if done.fields.valid {Ready(Page {after:done.fields.after,limit:done.fields.limit,kind:done.fields.kind})} else {Bad}','Ready(Page {after:0,limit:2,kind:""})')
   texts['logic']=texts['logic'].replace('if List::length(values)==0 {values} else {insert(List::head(values),ordered(List::tail(values)))}','values').replace('if done.more {Json::Integer(done.last)} else {Json::Null}','Json::Null')
  if kind=='duplicate-mutant':texts['policy']=texts['policy'].replace(' && !fields.seen_after','').replace(' && !fields.seen_limit','').replace(' && !fields.seen_kind','')
  if kind=='cursor-mutant':texts['logic']=texts['logic'].replace('if done.more {Json::Integer(done.last)} else {Json::Null}','if done.count>0 {Json::Integer(done.last)} else {Json::Null}')
 else:
  texts=dict(app=SQL_APP,model=SQL_MODEL,logic=SQL_LOGIC,policy=SQL_POLICY)
  if kind in ('starter','aggregate-mutant'):texts['policy']=texts['policy'].replace('if qty1>1000000-qty2 {Invalid} else {Ready(Cart {first:Line {sku:sku1,qty:qty1+qty2},second:empty})}','Ready(Cart {first:a,second:empty})')
  if kind=='starter':
   texts['policy']=texts['policy'].replace(SQL_POLICY[SQL_POLICY.index('fn plan'):SQL_POLICY.index('fn params')],'''fn plan(sku1:I64,qty1:I64,sku2:I64,qty2:I64,key:Text)->Plan {
 if sku1<=0 || qty1<1 || qty1>1000000 || qty2<0 || qty2>1000000 || Text::length(key)==0 {Invalid} else {Ready(Cart {first:Line {sku:sku1,qty:qty1},second:Line {sku:sku2,qty:qty2}})}
}
''')
   texts['logic']=texts['logic'].replace(SQL_LOGIC[SQL_LOGIC.index('fn perform'):SQL_LOGIC.index('fn apply')],'''fn perform(db:Native::Database,cart:Cart,key:Text)->SqlResult<Receipt> actions {db.execute,db.read,db.close} {
 let rows=try query(db,"SELECT available FROM inventory WHERE sku=?1",params1(Cell::Integer(cart.first.sku)));
 if List::length(rows)==0 {Ok(receipt(false,false,"missing"))} else {
  let available=try integer_at(List::head(rows),0);
  if available<cart.first.qty {Ok(receipt(false,false,"stock"))} else {
   try execute(db,"UPDATE inventory SET available=available-?1 WHERE sku=?2",params2(Cell::Integer(cart.first.qty),Cell::Integer(cart.first.sku)));
   Ok(receipt(true,false,"reserved"))
  }
 }
}
''').replace('try begin(db);let outcome=perform(db,cart,key);\n   match outcome {Err(error)=>{let ignored=rollback(db);Err(error)},Ok(value)=>{try commit(db);Ok(value)}}','try begin(db);let value=try perform(db,cart,key);try commit(db);Ok(value)')
  if kind=='cleanup-mutant':texts['logic']=texts['logic'].replace('let ignored=rollback(db);','let ignored=true;')
  if kind=='second-mutant':texts['logic']=texts['logic'].replace(' || other_available<cart.second.qty','')
 out=OLD.libraries('catalog' if task=='feed' else 'transfer')
 for component,text in texts.items():out[component+'.lobo']=text;out[component+'.lobo.ids.json']=OLD.identities(text,task,component,texts)
 out['BUNDLE.json']=json.dumps(sources(task),indent=2)+'\n';return out

def http_oracle(c):
 method,path=c['method'],c['path'];status=404;body={'error':'not found'}
 if method not in ('GET','HEAD'):status=405;body={'error':'method not allowed'}
 elif path=='/health':status=200;body={'status':'ready'}
 elif path=='/api/events' or path.startswith('/api/events?'):
  params={};valid=True
  if '?' in path:
   for part in path.split('?',1)[1].split('&'):
    if part.count('=')!=1:valid=False;break
    k,v=part.split('=')
    if k not in ('after','limit','kind') or k in params:valid=False;break
    params[k]=v
  after=params.get('after','0');limit=params.get('limit','2');kind=params.get('kind','')
  valid=valid and bool(re.fullmatch(r'0|[1-9][0-9]*',after)) and int(after)<=2**63-1 and bool(re.fullmatch('[1-3]',limit)) and (('kind' not in params) or kind in ('info','audit'))
  if not valid:status=400;body={'error':'bad query'}
  else:
   available=sorted((e for e in EVENTS if e['published'] and e['id']>int(after) and (not kind or e['kind']==kind)),key=lambda e:e['id']);page=available[:int(limit)];status=200;body=dict(items=[{k:e[k] for k in ('id','kind','message')} for e in page],count=len(page),next=page[-1]['id'] if len(available)>len(page) else None)
 return dict(status=status,body=body,head_empty=True,length_ok=True,content_type='application/json',allow='GET, HEAD' if status==405 else None)

def cases(task,hidden=False):
 if task=='feed':
  public=[('GET','/health'),('GET','/api/events'),('GET','/api/events?after=3&limit=2'),('GET','/api/events?kind=audit&limit=1'),('HEAD','/api/events?after=12')]
  private=[('GET',p) for p in ['/api/events?limit=1','/api/events?limit=3','/api/events?after=0&limit=2','/api/events?limit=2&after=0','/api/events?after=9&kind=audit&limit=3','/api/events?kind=info&after=1&limit=1','/api/events?after=9223372036854775807','/api/events?after=9223372036854775806&limit=1','/api/events?after=01','/api/events?after=00','/api/events?after=-1','/api/events?after=+1','/api/events?after=9223372036854775808','/api/events?limit=0','/api/events?limit=4','/api/events?limit=02','/api/events?','/api/events?after=1&','/api/events?&limit=2','/api/events?limit=2&&after=1','/api/events?after=1&after=1','/api/events?limit=1&limit=2','/api/events?kind=info&kind=info','/api/events?unknown=1','/api/events?kind=','/api/events?kind=Info','/api/events?after=%31','/api/events?after=1=2','/api/events/','/health?x=1']]
  private += [('POST','/api/events?limit=bad'),('PUT','/unknown'),('HEAD','/api/events?kind=info'),('HEAD','/api/events?limit=0'),('HEAD','/api/events?kind=audit&after=3')]
  return [dict(method=m,path=p) for m,p in (public+private if hidden else public)]
 def c(ops,inventory=None):return dict(inventory=inventory if inventory is not None else [[1,10],[2,5],[3,0]],operations=ops)
 public=[c([[1,2,2,1,'a'],[2,1,1,2,'a']]),c([[1,2,1,3,'b'],[1,5,0,0,'b']]),c([[1,1,2,6,'c']]),c([[1,0,2,0,'d']]),c([[1,2,2,1,'e'],[1,2,2,2,'e']])]
 private=[c([[1,2,2,1,'reject']]),c([[1,2,2,1,'decode']],[[1,10],[2,'bad']]),c([[1,11,9,1,'priority']]),c([[1,11,2,1,'stock']]),c([[1,2,3,1,'zero']]),c([[0,0,2,2,'ignore']]),c([[2,2,-9223372036854775808,0,'ignore']]),c([[1,2,1,9223372036854775807,'overflow']]),c([[1,600000,1,600000,'sum']]),c([[1,-1,2,1,'neg']]),c([[0,1,2,1,'sku']]),c([[1,2,2,1,'']]),c([[1,2,2,1,'夜'*24]]),c([[1,2,2,1,'夜'*25]]),c([[1,2,2,1,"x' OR 1=1--"],[2,1,1,2,"x' OR 1=1--"]]),c([[1,1,2,1,'x\0a'],[1,1,2,1,'x\0b']]),c([[1,10,0,0,'all'],[1,10,0,0,'all']]),c([[1,1,2,1,'K'],[1,1,2,1,'k']]),c([[1,2,2,1,'x'],[1,0,2,0,'x']]),c([[1,2,2,1,'x'],[2,2,1,1,'x']]),c([[2,1,0,0,'single'],[0,0,2,1,'single']]),c([[1,1000000,1,0,'max']],[[1,1000000]]),c([[1,1,2,1,'decode-missing']],[[1,'bad']]),c([[1,2,2,1,'reject'],[1,2,2,1,'good']]),c([[1,2,2,1,'a'],[1,2,2,1,'b'],[2,1,1,2,'a']])]
 return public+private if hidden else public

def sql_oracle(c):
 inventory=dict(c['inventory']);ledger={};out=[]
 for sku1,qty1,sku2,qty2,key in c['operations']:
  reason='reserved';error=None;cart={};valid=1<=len(key)<=24
  for sku,qty in ((sku1,qty1),(sku2,qty2)):
   if qty==0:continue
   if sku<=0 or not 1<=qty<=1000000:valid=False
   cart[sku]=cart.get(sku,0)+qty
  valid=valid and bool(cart) and all(qty<=1000000 for qty in cart.values());lines=sorted(cart.items());payload=[*lines[0],*(lines[1] if len(lines)>1 else (0,0))] if lines else []
  if not valid:reason='invalid'
  elif key in ledger:reason='replay' if ledger[key]==payload else 'conflict'
  elif any(sku not in inventory for sku in cart):reason='missing'
  elif any(type(inventory[sku]) is not int for sku in cart):error=('Decode',0)
  elif any(inventory[sku]<qty for sku,qty in cart.items()):reason='stock'
  elif key=='reject':error=('Constraint',275)
  else:
   for sku,qty in cart.items():inventory[sku]-=qty
   ledger[key]=payload
  reply=dict(case='Err',payload=dict(kind=dict(case=error[0],payload=None),backend_code=error[1])) if error else dict(case='Ok',payload=dict(applied=reason=='reserved',replay=reason=='replay',reason=reason))
  out.append(dict(reply=reply,clean=True,inventory=sorted([[k,v] for k,v in inventory.items()]),requests=sorted([[k]+v for k,v in ledger.items()])))
 return out

def oracle(task,c):return http_oracle(c) if task=='feed' else sql_oracle(c)
