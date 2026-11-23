import assert from 'node:assert/strict'
import test from 'node:test'
import { createRequire } from 'node:module'
import { readFile } from 'node:fs/promises'
const require = createRequire(import.meta.url)
const navigation = require('../src/navigation.cjs')

test('typed MVC JSON calls and constructors navigate to the supplied helper module', async () => {
  const mvc = await readFile(new URL('../../../examples/mvc.lobo', import.meta.url), 'utf8')
  const json = await readFile(new URL('../../../libraries/json/json.lobo', import.meta.url), 'utf8')
  const http = await readFile(new URL('../../../libraries/http/http.lobo', import.meta.url), 'utf8')
  const sources = [{name:'mvc',text:mvc},{name:'json',text:json},{name:'http',text:http}]
  for (const [needle, declaration] of [['field(object()', 'field('], ['object(),', 'object('], ['json_response(200', 'json_response('], ['Object(field', 'Object(JsonObject)']]) {
    const result = navigation.definitionInBundle(sources, 'mvc', mvc.indexOf(needle))
    assert.equal(result?.source,'json',needle)
    assert.equal(result?.target.start,json.indexOf(declaration),needle)
  }
})

test('web app helpers navigate across their explicit library module', async () => {
  const app = await readFile(new URL('../../../examples/web-server/app.lobo', import.meta.url), 'utf8')
  const http = await readFile(new URL('../../../libraries/http/http.lobo', import.meta.url), 'utf8')
  const bundle = [{ name: 'app', text: app }, { name: 'http', text: http }]
  for (const [needle, target] of [['text_response(200', 'text_response('], ['strip_prefix(path', 'strip_prefix('], ['Matched(name)', 'Matched(Text)']]) {
    const result = navigation.definitionInBundle(bundle, 'app', app.indexOf(needle))
    assert.equal(result?.source, 'http', needle)
    assert.equal(result?.target.start, http.indexOf(target), needle)
  }
})

const library = `module lib @id("lib");
record Pair @id("pair") { value: I64 }
variant Answer @id("answer") { Ok(Pair), Empty }
function wrap @id("wrap")(n: I64) -> Answer { Answer::Ok(Pair { value: n }) }`
const app = `module app @id("app");
import make @id("wrap");
import Reply @id("answer");
import Item @id("pair");
function main @id("main")(n: I64) -> I64 {
  let explicit: Item = Item { value: n };
  match (make(explicit.value)) { Ok(p) { p.value }, Empty { 0 } }
}`
const sources = [{ name: 'app', text: app }, { name: 'lib', text: library }]
function target(text, needle, name = 'app', occurrence = 'lastIndexOf') {
  return navigation.definitionInBundle(sources, name, text[occurrence](needle))
}

test('imported generic results retain caller nominal identity for field navigation', () => {
  const lib = 'module lib @id("lib"); function identity<T> @id("id")(x: T) -> T { x }'
  const app = 'module app @id("app"); import echo @id("id"); record Item @id("item") { value: I64 } function f @id("f")(x: Item) -> I64 { echo::<Item>(x).value }'
  const bundle = [{name:'app',text:app},{name:'lib',text:lib}]
  const call = navigation.definitionInBundle(bundle,'app',app.lastIndexOf('echo::'))
  assert.equal(call?.source,'lib')
  assert.equal(call?.target.start,lib.indexOf('identity<'))
  const field = navigation.definitionInBundle(bundle,'app',app.lastIndexOf('value'))
  assert.equal(field?.source,'app')
  assert.equal(field?.target.start,app.indexOf('value:'))
})

test('bundle navigation follows selected aliases and nominal fields/cases to their owning source', () => {
  let result = target(app, 'make(')
  assert.equal(result.source, 'lib')
  assert.equal(result.target.start, library.indexOf('wrap @'))
  result = target(app, 'Item =')
  assert.equal(result.source, 'lib')
  assert.equal(result.target.start, library.indexOf('Pair @'))
  for (const needle of ['value: n', 'value))', 'value },']) {
    result = target(app, needle)
    assert.equal(result.source, 'lib', needle)
    assert.equal(result.target.start, library.indexOf('value:'))
  }
  result = target(app, 'Ok(p)')
  assert.equal(result.source, 'lib')
  assert.equal(result.target.start, library.indexOf('Ok(Pair)'))
  result = target(app, 'make @')
  assert.equal(result.source, 'lib')
  assert.equal(result.target.start, library.indexOf('wrap @'))
})

test('inferred hidden nominal payloads keep identity despite same-spelling types in other modules', () => {
  const root = `module app @id("app"); import make @id("wrap");
record Pair @id("other.pair") { value: Bool }
function main @id("main")() -> I64 { match (make(5)) { Ok(p) { p.value }, Empty { 0 } } }`
  const bundle = [{ name: 'app', text: root }, { name: 'lib', text: library }]
  const result = navigation.definitionInBundle(bundle, 'app', root.lastIndexOf('value'))
  assert.equal(result.source, 'lib')
  assert.equal(result.target.start, library.indexOf('value:'))
  assert.equal(navigation.definitionInBundle([...bundle].reverse(), 'app', root.lastIndexOf('value')).source, 'lib')
})

test('mutual function imports resolve without initialization or source ordering', () => {
  const left = 'module a @id("a"); import next @id("b.f"); function f @id("a.f")(n: I64) -> I64 { next(n) }'
  const right = 'module b @id("b"); import next @id("a.f"); function f @id("b.f")(n: I64) -> I64 { next(n) }'
  const result = navigation.definitionInBundle([{ name: 'a', text: left }, { name: 'b', text: right }], 'a', left.lastIndexOf('next('))
  assert.equal(result.source, 'b')
  assert.equal(result.target.start, right.indexOf('f @'))
})

test('missing/ambiguous closures and aggregate budgets fail closed', () => {
  for (const bundle of [
    sources.slice(0, 1),
    [sources[0], sources[0]],
    [...sources, { name: 'duplicate', text: library }],
    [sources[0], { name: 'lib', text: library.replace('value: I64', 'value: Missing') }],
    [sources[0], { name: 'lib', text: library.replace('value: n', 'value: (') }],
    [sources[0], { name: 'lib', text: library + ' '.repeat(1_048_576) }],
    [...sources, ...Array.from({ length: 15 }, (_, index) => ({ name: `more${index}`, text: '' }))]
  ]) assert.equal(navigation.definitionInBundle(bundle, 'app', app.lastIndexOf('make(')), undefined)
  assert.equal(navigation.definitionInBundle(sources, 'unknown', 0), undefined)
  assert.equal(navigation.definitionInBundle(sources, 'app', -1), undefined)
})

test('actual collection/tree callers navigate to one shared reverse and cursor shape', async () => {
  const bundle = await Promise.all(['collections', 'tree', 'sequences'].map(async name => ({ name,
    text: await readFile(new URL(`../../../examples/${name}.lobo`, import.meta.url), 'utf8') })))
  const shared = bundle[2].text
  for (const [name, needle] of [['collections', 'reverse(work.kept)'], ['tree', 'reverse_shared(input)']]) {
    const text = bundle.find(file => file.name === name).text
    const result = navigation.definitionInBundle(bundle, name, text.indexOf(needle))
    assert.equal(result.source, 'sequences')
    assert.equal(result.target.start, shared.indexOf('fn reverse(') + 3)
  }
  const result = navigation.definitionInBundle(bundle, 'sequences', shared.lastIndexOf('reversed'))
  assert.equal(result.source, 'sequences')
  assert.equal(result.target.start, shared.indexOf('reversed:'))
})

test('ordinary testing laws navigate imported helpers and inferred summary fields', async () => {
  const bundle = await Promise.all(['testing/checks', 'tree', 'sequences'].map(async name => ({ name,
    text: await readFile(new URL(`../../../examples/${name}.lobo`, import.meta.url), 'utf8') })))
  const text = bundle[0].text
  const result = navigation.definitionInBundle(bundle, 'testing/checks', text.indexOf('.count') + 1)
  assert.equal(result.source, 'tree')
  assert.equal(result.target.start, bundle[1].text.indexOf('count: I64'))
})

test('try navigates an imported producer payload without unused type imports', () => {
  const library='module models; record Hidden {value:I64} variant R<T> {Ok(T),Err(Text)} fn get()->R<Hidden> {Ok(Hidden {value:42})} identities {module:"models",Hidden:"hidden",R:"foreign.result",get:"get"}'
  const app='module app; use models {get}; variant Reply {Ok(I64),Err(Text)} fn main()->Reply {let value=try get(); Ok(value.value)} identities {module:"app",models.get:"get",Reply:"reply",main:"main"}'
  const sources=[{name:'app',text:app},{name:'models',text:library}]
  const call=navigation.definitionInBundle(sources,'app',app.indexOf('try get')+4)
  assert.equal(call?.source,'models')
  assert.equal(call?.target.start,library.indexOf('get()->'))
  const field=navigation.definitionInBundle(sources,'app',app.indexOf('value.value')+6)
  assert.equal(field?.source,'models')
  assert.equal(field?.target.start,library.indexOf('value:I64'))
})

test('maintained try snapshot resolves its OS producers with the explicit three-file closure', async () => {
  const snapshot=await readFile(new URL('../../../examples/native/snapshot.lobo',import.meta.url),'utf8')
  const metadata=JSON.parse(await readFile(new URL('../../../examples/native/snapshot.lobo.ids.json',import.meta.url),'utf8')).identities
  const os=await readFile(new URL('../../../libraries/native/native.lobo',import.meta.url),'utf8')
  const permissions=await readFile(new URL('../../../libraries/native/permissions.lobo',import.meta.url),'utf8')
  const sources=[{name:'snapshot',text:snapshot,identities:metadata},{name:'os',text:os},{name:'access',text:permissions}]
  for(const name of ['restrict_capability','monotonic_ms','random_bytes']) {
    const result=navigation.definitionInBundle(sources,'snapshot',snapshot.indexOf(`os.${name}`)+3)
    assert.equal(result?.source,'os',name)
    assert.equal(result?.target.start,os.indexOf(`fn ${name}`)+3,name)
  }
})

test('SQLite row helpers and their typed try payloads navigate through the maintained bundle', async () => {
  const paths=JSON.parse(await readFile(new URL('../../../scripts/sqlite-rest-sources.json',import.meta.url),'utf8'))
  const sources=await Promise.all(paths.map(async name=>({
    name,text:await readFile(new URL(`../../../${name}`,import.meta.url),'utf8'),
    identities:JSON.parse(await readFile(new URL(`../../../${name}.ids.json`,import.meta.url),'utf8').catch(()=>'{}')).identities
  })))
  const controller=sources.find(s=>s.name.endsWith('/controller.lobo'))
  const sql=sources.find(s=>s.name==='libraries/sql/sql.lobo')
  for(const name of ['expect_columns','integer_at','text_at']) {
    const result=navigation.definitionInBundle(sources,controller.name,controller.text.indexOf(`${name}(row`))
    assert.equal(result?.source,sql.name,name)
    assert.equal(result?.target.start,sql.text.indexOf(`fn ${name}(`)+3,name)
  }
  const rowUse=controller.text.indexOf('row_json(row)')
  const result=navigation.definitionInBundle(sources,controller.name,rowUse)
  assert.equal(result?.source,controller.name)
  assert.equal(result?.target.start,controller.text.indexOf('fn row_json(')+3)
})


test('SQLite app navigates to the shared context server and actionful controller', async () => {
  const paths=JSON.parse(await readFile(new URL('../../../scripts/sqlite-rest-sources.json',import.meta.url),'utf8'))
  const sources=await Promise.all(paths.map(async name=>({name,
    text:await readFile(new URL(`../../../${name}`,import.meta.url),'utf8'),
    identities:JSON.parse(await readFile(new URL(`../../../${name}.ids.json`,import.meta.url),'utf8').catch(()=>'{}')).identities
  })))
  const app=sources.find(s=>s.name.endsWith('/sqlite-rest/app.lobo'))
  const transport=sources.find(s=>s.name.endsWith('/http-server/transport.lobo'))
  const controller=sources.find(s=>s.name.endsWith('/sqlite-rest/controller.lobo'))
  for(const [offset,target,needle] of [[app.text.indexOf('serve_with(cap'),transport,'serve_with<C>'],[app.text.indexOf('db, handle')+4,controller,'handle(db:']]) {
    const result=navigation.definitionInBundle(sources,app.name,offset)
    assert.equal(result?.source,target.name)
    assert.equal(result?.target.start,target.text.indexOf(needle))
  }
})


test('pure REST app navigates its lifecycle entry and the shared accept state fields', async () => {
 const paths=['examples/native/rest/app.lobo','examples/native/rest/controller.lobo','libraries/http-server/transport.lobo','libraries/http-server/server.lobo','libraries/http/http.lobo','libraries/json/json.lobo','libraries/native/permissions.lobo']
 const sources=await Promise.all(paths.map(async name=>({name,text:await readFile(new URL(`../../../${name}`,import.meta.url),'utf8'),identities:JSON.parse(await readFile(new URL(`../../../${name}.ids.json`,import.meta.url),'utf8').catch(()=>'{}')).identities})))
 const app=sources[0],transport=sources[2]
 const entry=navigation.definitionInBundle(sources,app.name,app.text.indexOf('serve(cap'))
 assert.equal(entry?.source,transport.name)
 assert.equal(entry?.target.start,transport.text.indexOf('serve(cap:'))
 for(const [needle,shift,target] of [['next_connection(listener, state',0,'next_connection(listener:'],['connection.state)',11,'state: ServeState }']]) {
  const result=navigation.definitionInBundle(sources,transport.name,transport.text.indexOf(needle)+shift)
  assert.equal(result?.source,transport.name,needle)
  assert.equal(result?.target.start,transport.text.indexOf(target),needle)
 }
})


test('typed OS permission cases and HTTP profile navigate to their definitions',async()=>{
 const paths=['examples/native/snapshot.lobo','libraries/native/native.lobo','libraries/native/permissions.lobo']
 const sources=await Promise.all(paths.map(async name=>({name,text:await readFile(new URL(`../../../${name}`,import.meta.url),'utf8'),identities:JSON.parse(await readFile(new URL(`../../../${name}.ids.json`,import.meta.url),'utf8').catch(()=>'{}')).identities})))
 const app=sources[0],access=sources[2]
 for(const [needle,target] of [['ClockRead','ClockRead,'],['RandomRead','RandomRead,'],['allow(one','allow(permissions:'],['one(ClockRead','one(permission:']]) {
  const result=navigation.definitionInBundle(sources,app.name,app.text.indexOf(needle))
  assert.equal(result?.source,access.name,needle)
  assert.equal(result?.target.start,access.text.indexOf(target),needle)
 }
})
