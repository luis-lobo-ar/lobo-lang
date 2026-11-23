import assert from 'node:assert/strict'
import test from 'node:test'
import {definitionAt,definitionInBundle} from '../src/navigation.cjs'
import {checkBundle} from '../src/compiler.cjs'
import {resolve} from 'node:path'
import {readFileSync} from 'node:fs'

const source=`module m;
record Item {value:I64}
fn identity<T>(x:T)->T {x}
fn apply<T,U>(f:Fn(T)->U,x:T)->U {f(x)}
fn extract(item:Item)->I64 {item.value}
fn main()->I64 {let item=identity(Item {value:42}); apply(extract,item)}
identities {module:"m",Item:"item",identity:"identity",apply:"apply",extract:"extract",main:"main"}`

test('inferred call results preserve field and contextual callback navigation',async()=>{
  for(const [use,target] of [['identity(Item','identity<T>'],['apply(extract','apply<T,U>'],['extract,item','extract(item:']]) {
    assert.equal(definitionAt(source,source.lastIndexOf(use))?.target.start,source.indexOf(target),use)
  }
  const withField=source.replace('apply(extract,item)','identity(item).value')
  assert.equal(definitionAt(withField,withField.lastIndexOf('.value')+1)?.target.start,withField.indexOf('value:I64'))
  const compiler=resolve(import.meta.dirname,'../../../target/debug/lobo')
  assert.equal((await checkBundle(compiler,{root:'main',sources:[{name:'main',text:source}]})).kind,'ok')
})

test('inferred imported generics and callback aliases navigate by identity',()=>{
  const app='module app; use shared {identity,extract,Item}; fn main()->I64 {shared.identity(Item {value:42}).value} identities {module:"app",main:"main",shared.identity:"identity",shared.extract:"extract",shared.Item:"item"}'
  const lib=source.replace('module m;','module lib;').replace('module:"m"','module:"lib"').replace('main:"main"','main:"lib.main"')
  const sources=[{name:'app',text:app},{name:'lib',text:lib}]
  const result=definitionInBundle(sources,'app',app.indexOf('.value}')+1)
  assert.equal(result?.source,'lib')
  assert.equal(result?.target.start,lib.indexOf('value:I64'))
})

test('maintained generic MVC navigates its inferred adapter, callback and view fields',()=>{
  const names=['examples/mvc.generic.lobo','libraries/http/http.lobo','libraries/json/json.lobo']
  const sources=names.map(name=>({name,text:readFileSync(resolve(import.meta.dirname,'../../..',name),'utf8')}))
  sources[0].identities=JSON.parse(readFileSync(resolve(import.meta.dirname,'../../../examples/mvc.generic.lobo.ids.json'),'utf8')).identities
  const source=sources[0].text
  for(const [offset,target] of [[source.lastIndexOf('respond_json('),source.indexOf('respond_json<T>')],[source.lastIndexOf('render_user)'),source.indexOf('render_user(user:')],[source.indexOf('user.name')+5,source.indexOf('name: Text')]]) {
    const result=definitionInBundle(sources,names[0],offset)
    assert.equal(result?.source,names[0])
    assert.equal(result?.target.start,target)
  }
})
