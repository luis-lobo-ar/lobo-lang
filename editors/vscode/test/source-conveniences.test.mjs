import assert from 'node:assert/strict'
import test from 'node:test'
import {createRequire} from 'node:module'
const require=createRequire(import.meta.url)
const {definitionAt}=require('../src/navigation.cjs')
const source=`module m;
record Box<T> {value:T,flag:Bool}
variant R {Ok(I64),Err(Text)}
fn pass()->R {Ok(1)}
fn updated<T>(original:Box<T>)->Box<T> {Box { ...original,flag:true }}
fn main()->R {try pass(); let original=Box {value:42,flag:false}; let changed=Box { ...original,flag:true }; try pass(); Ok(changed.value)}
identities {module:"m",Box:"box",R:"r",pass:"pass",updated:"updated",main:"main"}`
test('record update bases, generic fields and standalone try statements retain local navigation',()=>{
 for(const [needle,target] of [['...original,flag:true }}','original:Box<T>'],['...original,flag:true };','original=Box'],['changed.value','changed=Box'],['.value)','value:T'],['flag:true };','flag:Bool'],['try pass();','pass()->']]) {
   const offset=source.indexOf(needle)+(needle.startsWith('...')?3:needle.startsWith('.')?1:needle.startsWith('try ')?4:0)
   const result=definitionAt(source,offset)
   assert.equal(result?.target.start,source.indexOf(target),needle)
 }
})
test('unknown or repeated update fields and misplaced spreads do not produce navigation',()=>{
 for(const invalid of ['Box { ...original,unknown:true }','Box { ...original,flag:true,flag:false }','Box {flag:true,...original}']) {
   const text=source.replace('Box { ...original,flag:true }}',invalid+'}')
   assert.equal(definitionAt(text,text.lastIndexOf('changed.value')),undefined,invalid)
 }
})

test('parenthesized standalone try keeps navigation and compound discarded expressions are rejected',()=>{
  const valid=source.replace('try pass();','(try pass());')
  assert.equal(definitionAt(valid,valid.indexOf('try pass')+4)?.target.start,valid.indexOf('pass()->'))
  const invalid=source.replace('try pass();','try pass()+1;')
  assert.equal(definitionAt(invalid,invalid.lastIndexOf('changed.value')),undefined)
})
