"""Fresh, multi-module application changes; independent HTTP and SQLite state oracles."""
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
TASKS=('catalog','transfer')
LIBS={'http':'libraries/http-server/server.lobo','io':'libraries/http-server/transport.lobo','helpers':'libraries/http/http.lobo','json':'libraries/json/json.lobo','access':'libraries/native/permissions.lobo','sql':'libraries/sql/sql.lobo'}
PRODUCTS=[dict(id=2,name='Tea "夜"',stock=3,enabled=True),dict(id=7,name='Line\nBack\\',stock=0,enabled=True),dict(id=11,name='hidden',stock=9,enabled=False)]
REQUIREMENTS={
'catalog':'''Extend the working native HTTP catalog application by editing logic.lobo and policy.lobo (and their identity sidecars). Preserve existing declarations/IDs. Only exact GET and HEAD are allowed; other methods return405 before routing. GET /health returns200 {"status":"ready"}. GET /api/products returns200 {"items":[enabled products in original model order],"total":N}. GET /api/products/available includes only enabled products with stock>0. Each serialized product has exactly id (JSON number), name (escaped original text), stock (JSON number). GET /api/products/<id> returns that enabled product, otherwise404 {"error":"not found"}. IDs must be canonical positive decimal1..I64_MAX: no leading zero, sign, overflow, query, extra slash or percent decoding. Routes are exact and case-sensitive. Unsupported method returns405 {"error":"method not allowed"}. HEAD has the same status/headers/representation length as GET but no wire body; supplied HTTP transport owns that behavior and Allow: GET, HEAD on405. Use the existing JSON/HTTP libraries; do not hand-build JSON or change transport/model. policy.read_id takes the entire path and returns Found(id) only for a valid detail path. Inputs have at most128 ASCII path characters. The fixed server serves a bounded number of real loopback TCP requests. The baseline deliberately includes disabled entries, ignores the available filter and accepts leading zeros.''',
 'transfer':'''Extend the working SQLite transfer application with durable idempotency and input policy. Edit logic.lobo and policy.lobo and their sidecars; preserve existing declarations/IDs. apply(db,source_id,target_id,amount,key) returns SqlResult<Receipt> and borrows the database (never close it). Validate BEFORE ledger lookup: IDs>0 and different; amount1..1000000; key length1..24 Unicode scalars, opaque/case-sensitive including quotes and NUL. Invalid returns Ok{applied:false,replay:false,reason:"invalid"}. Schema: accounts(id INTEGER PRIMARY KEY,balance INTEGER NOT NULL CHECK(balance>=0 AND balance<=1000000)); requests(key TEXT PRIMARY KEY,source_id INTEGER,target_id INTEGER,amount INTEGER). Query requests by bound key. An exact existing payload returns Ok(false,true,"replay"), without another transfer; a different payload for the key returns Ok(false,false,"conflict"). For a new key: either missing account =>"missing", insufficient source balance=>"funds" (both false,false); otherwise atomically debit source, credit target and INSERT requests, then return Ok(true,false,"applied"). All SQL parameters must be bound, including key; no dynamic SQL interpolation. On SQL or typed-row decode failure, preserve the original SqlError and roll back ALL changes. No pending transaction on ANY return (replay, rejection, error or success). The provided wrapper owns begin/commit/rollback, captures outcomes before cleanup, and must remain correct. The fixed app probes for an open transaction before closing the borrowed DB and returns Audit{outcome,clean}; clean must always be true. Tests reuse a real DB file across requests and inspect accounts/requests after EACH call. A target balance exceeding1000000 must produce the real engine Constraint error and leave debit/ledger unchanged. Existing params helper builds key,source,target,amount parameters. Baseline transfers correctly but ignores the ledger/key-length rule. No concurrency, schema changes or dependencies are needed.'''}

def libraries(task):
 out={}
 for alias in (('http','io','helpers','json','access') if task=='catalog' else ('sql',)):
  path=ROOT/LIBS[alias];out['lib/'+alias+'.lobo']=path.read_text()
  side=Path(str(path)+'.ids.json')
  if side.exists():out['lib/'+alias+'.lobo.ids.json']=side.read_text()
 return out

def library_ids(alias):
 path=ROOT/LIBS[alias];side=Path(str(path)+'.ids.json')
 values=json.loads(side.read_text())['identities'] if side.exists() else dict(re.findall(r'([\w.]+)\s*:\s*"([^"]+)"',path.read_text().split('identities {')[-1]))
 values={k:v for k,v in values.items() if '.' not in k}
 if alias=='http':values['Response']='example.web.response'
 return values

def identities(text,task,component,alltext):
 values={'module':f'realistic.{task}.{component}'}
 for name in re.findall(r'\b(?:fn|record|variant)\s+(\w+)',text):values[name]=values['module']+'.'+name
 for group,items in re.findall(r'use\s+(\w+)\s*\{([^}]+)\}',text):
  ids=library_ids(group) if group in LIBS else {name:f'realistic.{task}.{group}.{name}' for name in re.findall(r'\b(?:fn|record|variant)\s+(\w+)',alltext[group])}
  for name in items.replace(' ','').split(','):
   name=name.strip()
   if name:values[group+'.'+name]=ids[name]
 return json.dumps(dict(format='lobo-identities/0.1',identities=values),indent=2)+'\n'

HTTP_MODEL='module model;\nrecord Product {id:I64,name:Text,stock:I64,enabled:Bool}\nfn catalog()->List<Product> {'
expression='List::empty<Product>()'
for product in reversed(PRODUCTS):
 expression='List::prepend(Product {id:'+str(product['id'])+',name:'+json.dumps(product['name'],ensure_ascii=False)+',stock:'+str(product['stock'])+',enabled:'+str(product['enabled']).lower()+'},'+expression+')'
HTTP_MODEL+=expression+'}\n'
HTTP_APP='''module app;
use access {restrict}; use io {server_permissions,serve}; use http {Request,Response}; use logic {handle};
fn main(port:I64,max_requests:I64)->Native::I64Result
actions {native.authorize,clock.read,net.listen,net.accept,net.read,net.write,net.close}
requires {port>=1 && port<=65535 && max_requests>=1 && max_requests<=128}
{let cap=try restrict(Native::capability(),server_permissions());serve(cap,port,max_requests,handle)}
'''
HTTP_POLICY='''module policy;
variant Key {Found(I64),Missing}
record Digits {value:I64,valid:Bool}
fn read_id(path:Text)->Key {
 let prefix="/api/products/";
 if Text::length(path)<=Text::length(prefix) || Text::slice(path,0,Text::length(prefix))!=prefix {Missing} else {
  let token=Text::slice(path,Text::length(prefix),Text::length(path)-Text::length(prefix));
  let bytes=Text::bytes(token);
  let parsed=fold(i in Bytes::length(bytes),state:Digits=Digits {value:0,valid:true}) {
   let digit=Bytes::at(bytes,i)-48;
   if !state.valid || digit<0 || digit>9 || state.value>(9223372036854775807-digit)/10 {Digits {value:0,valid:false}} else {Digits {value:state.value*10+digit,valid:true}}
  };
  if parsed.valid && parsed.value>0 && Bytes::at(bytes,0)!=48 {Found(parsed.value)} else {Missing}
 }
}
'''
HTTP_LOGIC='''module logic;
use model {Product,catalog}; use policy {Key,read_id}; use http {Request,Response,json_response,error_response};
use json {Json,JsonArray,object,field,array,append};
record Items {values:JsonArray,count:I64}
fn product_json(product:Product)->Json {
 let a=field(object(),"id",Json::Integer(product.id));
 let b=field(a,"name",Json::String(product.name));
 Json::Object(field(b,"stock",Json::Integer(product.stock)))
}
fn products_json(available:Bool)->Json {
 let products=catalog();
 let done=fold(i in List::length(products),state:Items=Items {values:array(),count:0}) {
  let item=List::head(fold(j in i,rest:List<Product> =products) {List::tail(rest)});
  if item.enabled && (!available || item.stock>0) {Items {values:append(state.values,product_json(item)),count:state.count+1}} else {state}
 };
 Json::Object(field(field(object(),"items",Json::Array(done.values)),"total",Json::Integer(done.count)))
}
fn handle(request:Request)->Response {
 if request.method!="GET" && request.method!="HEAD" {json_response(405,json {"error":"method not allowed"})} else {
  if request.path=="/health" {json_response(200,json {"status":"ready"})} else {
   if request.path=="/api/products" || request.path=="/api/products/available" {json_response(200,products_json(request.path=="/api/products/available"))} else {
    match read_id(request.path) {
     Missing=>json_response(404,json {"error":"not found"}),
     Found(id)=> {
      let products=catalog();
      let found=fold(i in List::length(products),pick:List<Product> =List::empty<Product>()) {
       let item=List::head(fold(j in i,rest:List<Product> =products) {List::tail(rest)});
       if item.id==id && item.enabled {List::prepend(item,pick)} else {pick}
      };
      if List::length(found)==0 {json_response(404,json {"error":"not found"})} else {json_response(200,product_json(List::head(found)))}
     }
    }
   }
  }
 }
}
'''
SQL_MODEL='module model; record Receipt {applied:Bool,replay:Bool,reason:Text}\n'
SQL_APP='''module app;
use sql {SqlResult,open,close,begin,rollback}; use logic {apply}; use model {Receipt};
record Audit {outcome:SqlResult<Receipt>,clean:Bool}
fn main(source_id:I64,target_id:I64,amount:I64,key:Text)->SqlResult<Audit>
actions {native.authorize,db.open,db.close,db.transaction,db.execute,db.read}
{
 let db=try open(Native::sqlite_capability());
 let outcome=apply(db,source_id,target_id,amount,key);
 let probe=begin(db);
 let clean=match probe {Err(error)=>false,Ok(done)=>{let ignored=rollback(db);true}};
 try close(db);
 Ok(Audit {outcome:outcome,clean:clean})
}
'''
SQL_POLICY='''module policy;
use sql {Cell,params1};
fn valid(source_id:I64,target_id:I64,amount:I64,key:Text)->Bool {source_id>0 && target_id>0 && source_id!=target_id && amount>=1 && amount<=1000000 && Text::length(key)>=1 && Text::length(key)<=24}
fn params(key:Text,source_id:I64,target_id:I64,amount:I64)->List<Cell> {List::prepend(Cell::String(key),List::prepend(Cell::Integer(source_id),List::prepend(Cell::Integer(target_id),params1(Cell::Integer(amount)))))}
'''
SQL_LOGIC='''module logic;
use model {Receipt}; use policy {valid,params};
use sql {Cell,Row,SqlResult,SqlError,query,execute,begin,commit,rollback,no_params,params1,params2,integer_at,text_at,expect_columns};
fn receipt(applied:Bool,replay:Bool,reason:Text)->Receipt {Receipt {applied:applied,replay:replay,reason:reason}}
fn perform(db:Native::Database,source_id:I64,target_id:I64,amount:I64,key:Text)->SqlResult<Receipt>
actions {db.execute,db.read,db.close}
{
 let seen=try query(db,"SELECT source_id,target_id,amount FROM requests WHERE key=?1",params1(Cell::String(key)));
 if List::length(seen)>0 {
  let old=List::head(seen);try expect_columns(old,3);
  let same=try integer_at(old,0)==source_id && try integer_at(old,1)==target_id && try integer_at(old,2)==amount;
  Ok(receipt(false,same,if same {"replay"} else {"conflict"}))
 } else {
  let from=try query(db,"SELECT balance FROM accounts WHERE id=?1",params1(Cell::Integer(source_id)));
  let dest=try query(db,"SELECT balance FROM accounts WHERE id=?1",params1(Cell::Integer(target_id)));
  if List::length(from)==0 || List::length(dest)==0 {Ok(receipt(false,false,"missing"))} else {
   let balance=try integer_at(List::head(from),0);
   let target_balance=try integer_at(List::head(dest),0);
   if balance<amount {Ok(receipt(false,false,"funds"))} else {
    try execute(db,"UPDATE accounts SET balance=balance-?1 WHERE id=?2",params2(Cell::Integer(amount),Cell::Integer(source_id)));
    try execute(db,"UPDATE accounts SET balance=balance+?1 WHERE id=?2",params2(Cell::Integer(amount),Cell::Integer(target_id)));
    try execute(db,"INSERT INTO requests(key,source_id,target_id,amount) VALUES(?1,?2,?3,?4)",params(key,source_id,target_id,amount));
    Ok(receipt(true,false,"applied"))
   }
  }
 }
}
fn apply(db:Native::Database,source_id:I64,target_id:I64,amount:I64,key:Text)->SqlResult<Receipt>
actions {db.execute,db.read,db.close,db.transaction}
{
 if !valid(source_id,target_id,amount,key) {Ok(receipt(false,false,"invalid"))} else {
  try begin(db);
  let outcome=perform(db,source_id,target_id,amount,key);
  match outcome {
   Err(error)=>{let cleaned=rollback(db);Err(error)},
   Ok(value)=>{try commit(db);Ok(value)}
  }
 }
}
'''

def sources(task):return ['app.lobo','logic.lobo','policy.lobo','model.lobo']+['lib/'+a+'.lobo' for a in (('http','io','helpers','json','access') if task=='catalog' else ('sql',))]
def files(task,kind='starter'):
 if task=='catalog':
  texts=dict(app=HTTP_APP,model=HTTP_MODEL,logic=HTTP_LOGIC,policy=HTTP_POLICY)
  if kind=='starter':texts['logic']=texts['logic'].replace('item.enabled && (!available || item.stock>0)','true').replace('item.id==id && item.enabled','item.id==id');texts['policy']=texts['policy'].replace(' && Bytes::at(bytes,0)!=48','')
  if kind=='mutant':texts['policy']=texts['policy'].replace(' && Bytes::at(bytes,0)!=48','')
 else:
  texts=dict(app=SQL_APP,model=SQL_MODEL,logic=SQL_LOGIC,policy=SQL_POLICY)
  if kind in ('starter','mutant'):
   texts['logic']=texts['logic'].replace('if List::length(seen)>0','if false').replace('    try execute(db,"INSERT INTO requests(key,source_id,target_id,amount) VALUES(?1,?2,?3,?4)",params(key,source_id,target_id,amount));','')
  if kind=='starter':texts['policy']=texts['policy'].replace(' && Text::length(key)>=1 && Text::length(key)<=24','')
  if kind=='cleanup-mutant':texts['logic']=texts['logic'].replace('let cleaned=rollback(db);','let cleaned=true;')
 out=libraries(task)
 for component,text in texts.items():out[component+'.lobo']=text;out[component+'.lobo.ids.json']=identities(text,task,component,texts)
 out['BUNDLE.json']=json.dumps(sources(task),indent=2)+'\n'
 return out

def http_oracle(case):
 method,path=case['method'],case['path'];status=404;body={'error':'not found'}
 if method not in ('GET','HEAD'):status=405;body={'error':'method not allowed'}
 elif path=='/health':status=200;body={'status':'ready'}
 elif path in ('/api/products','/api/products/available'):
  items=[{k:p[k] for k in ('id','name','stock')} for p in PRODUCTS if p['enabled'] and (path=='/api/products' or p['stock']>0)];status=200;body=dict(items=items,total=len(items))
 elif re.fullmatch(r'/api/products/[1-9][0-9]*',path) and int(path.rsplit('/',1)[1])<=2**63-1:
  p=next((p for p in PRODUCTS if p['id']==int(path.rsplit('/',1)[1]) and p['enabled']),None)
  if p:status=200;body={k:p[k] for k in ('id','name','stock')}
 return dict(status=status,body=body,head_empty=True,length_ok=True,content_type='application/json',allow='GET, HEAD' if status==405 else None)

def cases(task,hidden=False):
 if task=='catalog':
  public=[dict(method=m,path=p) for m,p in [('GET','/health'),('GET','/api/products'),('GET','/api/products/available'),('GET','/api/products/02'),('HEAD','/api/products/2')]]
  private=[dict(method=m,path=p) for m,p in [('GET','/api/products/2'),('GET','/api/products/7'),('GET','/api/products/11'),('GET','/api/products/1'),('GET','/api/products/'),('GET','/api/products/0'),('GET','/api/products/-2'),('GET','/api/products/+2'),('GET','/api/products/2/'),('GET','/api/products/2?x=1'),('GET','/api/products/%32'),('GET','/api/products/9223372036854775808'),('GET','/api/products/9223372036854775807'),('GET','/api/products/0002'),('GET','/api/Products'),('GET','/health/'),('POST','/health'),('POST','/unknown'),('PUT','/api/products'),('HEAD','/health'),('HEAD','/api/products'),('HEAD','/api/products/available'),('HEAD','/api/products/11'),('GET','/api/products/available/'),('GET','/api/products/2a')]]
  return public+private if hidden else public
 def c(ops,accounts=None):return dict(accounts=accounts or [[1,10],[2,0]],operations=ops)
 public=[c([[1,2,3,'a'],[1,2,3,'a'],[1,2,4,'a']]),c([[1,2,11,'b']]),c([[1,9,1,'c']]),c([[1,2,1,'']]),c([[1,2,2,'cap']],[[1,10],[2,999999]])]
 private=[c([[1,2,1,"x' OR 1=1--"],[1,2,1,"x' OR 1=1--"]]),c([[1,2,1,'夜'*24]]),c([[1,2,1,'夜'*25]]),c([[1,2,1,'x\0y'],[1,2,1,'x\0z']]),c([[1,2,10,'all'],[1,2,10,'all']]),c([[1,2,1,'K'],[1,2,2,'k']]),c([[1,2,3,'x'],[2,1,3,'x']]),c([[1,2,3,'x'],[1,2,0,'x']]),c([[0,2,1,'x'],[1,1,1,'y']]),c([[1,2,-1,'x'],[1,2,1000001,'y']]),c([[9,2,1,'x'],[1,2,1,'x']]),c([[1,2,11,'x'],[1,2,2,'x']]),c([[1,2,2,'a'],[1,2,3,'b'],[1,2,2,'a']]),c([[1,2,1,'cap']],[[1,10],[2,1000000]]),c([[1,2,1,'decode']],[[1,'bad'],[2,0]])]
 return public+private if hidden else public

def sql_oracle(case):
 accounts=dict(case['accounts']);ledger={};out=[]
 for source,target,amount,key in case['operations']:
  reason='applied';error=None
  if source<=0 or target<=0 or source==target or not 1<=amount<=1000000 or not 1<=len(key)<=24:reason='invalid'
  elif key in ledger:reason='replay' if ledger[key]==[source,target,amount] else 'conflict'
  elif source not in accounts or target not in accounts:reason='missing'
  elif not isinstance(accounts[source],int) or not isinstance(accounts[target],int):error=('Decode',0)
  elif accounts[source]<amount:reason='funds'
  elif accounts[target]+amount>1000000:error=('Constraint',275)
  else:accounts[source]-=amount;accounts[target]+=amount;ledger[key]=[source,target,amount]
  reply=dict(case='Err',payload=dict(kind=dict(case=error[0],payload=None),backend_code=error[1])) if error else dict(case='Ok',payload=dict(applied=reason=='applied',replay=reason=='replay',reason=reason))
  out.append(dict(reply=reply,clean=True,accounts=sorted([[k,v] for k,v in accounts.items()]),requests=sorted([[k]+v for k,v in ledger.items()])))
 return out

def oracle(task,case):return http_oracle(case) if task=='catalog' else sql_oracle(case)
