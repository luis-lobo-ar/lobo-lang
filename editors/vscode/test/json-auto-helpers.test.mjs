import assert from 'node:assert/strict'
import test from 'node:test'
import {readFileSync} from 'node:fs'
import {resolve} from 'node:path'
import {project} from '../src/compact.cjs'
import {definitionInBundle} from '../src/navigation.cjs'
import {checkBundle} from '../src/compiler.cjs'
import {execFileSync} from 'node:child_process'

const source=`module m;
use js {Json,stringify};
record User {name:Text}
fn render(user:User)->Json {json {"name":user.name}}
fn encode(user:User)->Text {js.stringify(render(user))}
identities {module:"m",js.Json:"lobo.json.value",js.stringify:"lobo.json.stringify",User:"user",render:"render",encode:"encode"}`
const read=name=>readFileSync(resolve(import.meta.dirname,'../../..',name),'utf8')
const sources=[{name:'app',text:source},{name:'json',text:read('libraries/json/json.lobo')},{name:'http',text:read('libraries/http/http.lobo')}]

test('automatic JSON helpers preserve original navigation and native diagnostics',async()=>{
  const tokens=project(source).tokens
  assert.ok(tokens.some(token=>token.text==='__lobo_json_object'&&token.generated))
  assert.ok(tokens.some(token=>token.text==='__lobo_json_field'&&token.generated))
  const field=definitionInBundle(sources,'app',source.indexOf('user.name')+5)
  assert.equal(field?.target.start,source.indexOf('name:Text'))
  assert.equal(field?.source,'app')
  const compiler=resolve(import.meta.dirname,'../../../target/debug/lobo')
  assert.equal((await checkBundle(compiler,{root:'app',sources})).kind,'ok')
  const bad=source.replace('user.name','true')
  const result=await checkBundle(compiler,{root:'app',sources:[{name:'app',text:bad},...sources.slice(1)]})
  assert.equal(result.kind,'diagnostic')
  assert.equal(result.diagnostic.range.start.byte,bad.indexOf('true'))
})

test('reserved helpers stay generated and compatibility export remains ordinary legacy source',()=>{
  const projection=project(source)
  assert.ok(!projection.text.includes('__lobo_json_'))
  const compiler=resolve(import.meta.dirname,'../../../target/debug/lobo')
  const bundle={format:'lobo-source/0.1',root:'app',sources:[{name:'app',text:projection.text},...sources.slice(1)]}
  assert.equal(JSON.parse(execFileSync(compiler,['check','--bundle-stdin'],{input:JSON.stringify(bundle)})).status,'ok')
  assert.throws(()=>project(source.replace('user.name','__lobo_json_object()')))
  const colliding=source.replace('fn render','fn lobo_export_json_object()->I64 {7} fn render').replace('render:"render"','lobo_export_json_object:"collision",render:"render"')
  const portable=project(colliding).text
  assert.ok(portable.includes('import lobo_export_json_object_0'))
  const collisionBundle={...bundle,sources:[{name:'app',text:portable},...sources.slice(1)]}
  assert.equal(JSON.parse(execFileSync(compiler,['check','--bundle-stdin'],{input:JSON.stringify(collisionBundle)})).status,'ok')
  const empty=project(source.replace('json {"name":user.name}','json {}'))
  assert.ok(!empty.tokens.some(token=>token.text==='__lobo_json_field'))
})
