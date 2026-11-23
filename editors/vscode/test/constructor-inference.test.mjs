import assert from 'node:assert/strict'
import test from 'node:test'
import {definitionAt,definitionInBundle} from '../src/navigation.cjs'

const source=`module m;
record Box<T> {value:T}
variant Maybe<T> {None,Some(T)}
variant Other<T> {None,Some(T)}
fn create(flag:Bool)->Maybe<Box<I64>> {if flag {Some(Box {value:42})} else {None}}
fn main()->I64 {match create(true) {None=>0,Some(box)=>box.value}}
identities {module:"m",Box:"box",Maybe:"maybe",Other:"other",create:"create",main:"main"}`

test('contextual cases resolve to their expected variant and inferred records retain fields',()=>{
  for(const [use,target] of [['Some(Box','Some(T)'],['else {None','None,Some(T)'],['box.value','value:T']]) {
    const delta=use==='else {None'?6:use==='box.value'?4:0
    assert.equal(definitionAt(source,source.indexOf(use)+delta)?.target.start,source.indexOf(target),use)
  }
})

test('qualified payload inference supplies a later field or match type without context',()=>{
  const text=source.replace('match create(true)','let chosen=Maybe::Some(Box {value:42}); match chosen')
  assert.equal(definitionAt(text,text.indexOf('box.value')+4)?.target.start,text.indexOf('value:T'))
})

test('imported expected variant identities select cases with the same spelling',()=>{
  const lib=source.replace('module m;','module lib;').replace('module:"m"','module:"lib"')
  const app='module app; use models {Maybe,Other,Box}; fn create()->Maybe<Box<I64>> {Some(Box {value:42})} identities {module:"app",models.Maybe:"maybe",models.Other:"other",models.Box:"box",create:"app.create"}'
  const sources=[{name:'app',text:app},{name:'lib',text:lib}]
  const result=definitionInBundle(sources,'app',app.indexOf('Some(Box'))
  assert.equal(result?.source,'lib')
  assert.equal(result?.target.start,lib.indexOf('Some(T)'))
  const unknown=app.replace('->Maybe<Box<I64>> {Some(Box {value:42})}','->I64 {let value=Some(Box {value:42}); 0}')
  assert.equal(definitionInBundle([{name:'app',text:unknown},sources[1]],'app',unknown.indexOf('Some(Box')),undefined)
})

test('annotations, static arguments and loop state supply constructor navigation context',()=>{
  const text=source.replace('fn main()->I64 {match create(true) {None=>0,Some(box)=>box.value}}',
    'fn accept(value:Maybe<Box<I64>>)->I64 {match value {None=>0,Some(box)=>box.value}} fn main()->I64 {let box:Box<Maybe<I64>> = Box {value:None}; let state=fold(i in 1,state:Maybe<I64> = None) {Some(i)}; accept(Some(Box {value:42}))}')
    .replace('main:"main"','main:"main",accept:"accept"')
  for(const offset of [text.indexOf('value:None')+6,text.indexOf('= None)')+2]) {
    assert.equal(definitionAt(text,offset)?.target.start,text.indexOf('None,Some(T)'))
  }
  assert.equal(definitionAt(text,text.lastIndexOf('Some(Box'))?.target.start,text.indexOf('Some(T)'))
})

test('inferred constructor type growth fails closed at the compiler nesting bound',()=>{
  let body='let v0=Box {value:1};'
  for(let n=1;n<=10;n++) body+=`let v${n}=Box {value:v${n-1}};`
  const text=`module m; record Box<T> {value:T} fn main()->I64 {${body}0} identities {module:"m",Box:"box",main:"main"}`
  assert.equal(definitionAt(text,text.lastIndexOf('value:')),undefined)
})
