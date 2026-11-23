import assert from 'node:assert/strict'
import test from 'node:test'
import {createRequire} from 'node:module'
const require=createRequire(import.meta.url)
const {definitionAt,definitionInBundle}=require('../src/navigation.cjs')
const source=`module m;
record Item {value:I64}
fn load(value:Item)->Item actions {demo.read} {value}
fn apply<T>(callback:Fn(T) actions {demo.read}->T,value:T)->T actions {demo.read} {callback(value)}
fn main()->I64 actions {demo.read} {let item=apply(load,Item {value:42}); item.value}
fn reference()->Fn(Item) actions {demo.read}->Item {&load}
fn pure(value:Item)->Item {value}
fn returns_pure()->Fn(Item)->Item actions {demo.read} {&pure}
identities {module:"m",Item:"item",load:"load",apply:"apply",main:"main",reference:"reference",pure:"pure",returns_pure:"returns_pure"}`
test('action-bound callback types preserve inferred fields, references and parameter navigation',()=>{
 for(const [needle,target,delta] of [['apply(load','load(value:',6],['item.value','value:I64',5],['callback(value)','callback:Fn',0],['&load','load(value:',1]]) {
  assert.equal(definitionAt(source,source.indexOf(needle)+delta)?.target.start,source.indexOf(target),needle)
 }
})
test('action-bound callbacks retain cross-module navigation and contextual generic inference',()=>{
 const lib=source.replace('module m;','module library;')
 const app=`module a; use lib {Item,load,apply}; fn main()->I64 actions {demo.read} {let item=apply(load,Item {value:42}); item.value} identities {module:"a",lib.Item:"item",lib.load:"load",lib.apply:"apply",main:"app.main"}`
 const sources=[{name:'app',text:app},{name:'lib',text:lib}]
 assert.equal(definitionInBundle(sources,'app',app.indexOf('apply(load')+6)?.source,'lib')
 assert.equal(definitionInBundle(sources,'app',app.indexOf('item.value')+5)?.target.start,lib.indexOf('value:I64'))
})
test('malformed action-bound types fail closed in local navigation',()=>{
 for(const replacement of ['actions {demo.read,demo.read}','actions {single}','actions {demo.read,}']) {
  const invalid=source.replace('actions {demo.read}->T',replacement+'->T')
  assert.equal(definitionAt(invalid,invalid.indexOf('item.value')+5),undefined)
 }
})
