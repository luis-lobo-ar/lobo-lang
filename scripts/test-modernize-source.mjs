import assert from 'node:assert/strict'
import {createRequire} from 'node:module'
import test from 'node:test'
const require=createRequire(import.meta.url)
const {candidates,maintainedPath}=require('./modernize-source.cjs')

test('constructors, specializations and callbacks have distinct safe rewrite candidates',()=>{
  const source='fn main()->Box<Text> {let x=Box::<Text> {value:"x"}; apply::<Text,Text>(&identity::<Text>,x.value)}'
  const edits=candidates(source)
  assert.ok(edits.some(c=>c.kind==='type-arguments' && c.text.includes('Box {value:')))
  assert.ok(edits.some(c=>c.kind==='callback' && c.text.includes('(identity,x.value)')))
  assert.ok(edits.some(c=>c.kind==='type-arguments' && c.text.includes('apply(&identity')))
})
test('qualified cases preserve intervening comments and never rewrite literal text',()=>{
  const source='fn f()->Reply {let text="Reply::Ok(1)"; Reply:: // keep this\n Ok(1)}'
  const edits=candidates(source).filter(c=>c.kind==='constructor-owner')
  assert.equal(edits.length,1)
  assert.ok(edits[0].text.includes('// keep this\n'))
  assert.ok(edits[0].text.includes('"Reply::Ok(1)"'))
})
test('Text concatenation retains the original association and comments',()=>{
  const source='fn f(a:Text,b:Text,c:Text)->Text {Text::concat(a, Text::concat(b,c))}'
  const edits=candidates(source).filter(c=>c.kind==='text-concat')
  assert.ok(edits.some(c=>c.text.includes('(a + ( Text::concat(b,c)))')))
})
test('scope excludes frozen comparisons, studies, historical specs and test compatibility fixtures',()=>{
  for(const p of ['examples/core.lobo','examples/mvc.compact.lobo','libraries/native/native.lobo','.agents/skills/lobo-programming/references/examples.lobo']) assert.equal(maintainedPath(p),true,p)
  for(const p of ['examples/v1/document.lobo','examples/mvc.lobo','examples/transfer.lobo','benchmarks/fixtures/repair-study/tree.lobo','spec/core-v1.0.md','tests/native_syntax.rs','examples/../research/a.lobo']) assert.equal(maintainedPath(p),false,p)
})

test('single-expression arms and redundant groups can be shortened under compiler verification',()=>{
  const source='fn f()->I64 {match(value) {Ok(n)=>{(n)},Err(e)=>{0}}}'
  const edits=candidates(source)
  assert.ok(edits.some(c=>c.kind==='arm-block' && c.text.includes('Ok(n)=>(n)')))
  assert.ok(edits.some(c=>c.kind==='group' && c.text.includes('Ok(n)=>{n}')))
})

test('layout cleanup preserves literal and comment contents',()=>{
  const {tidy}=require('./modernize-source.cjs')
  const source='fn f()->Text {\n// keep =>  ,\n  match x {\n    Ok(n) =>  \n      "raw  =>  text" ,\n    Err(e) => ""\n  }\n}\n'
  const value=tidy(source)
  assert.ok(value.includes('// keep =>  ,'))
  assert.ok(value.includes('Ok(n) => "raw  =>  text",'))
})

test('a preceding builtin result type does not hide an inferable call', () => {
  const source = 'fn text(value: Text) -> Text { identity::<Text>(value) }'
  assert.ok(candidates(source).some(c => c.text.includes('identity(value)')))
})

test('match subject grouping leaves a lexical boundary after the keyword', () => {
  assert.ok(candidates('fn f()->I64 {match(value) {Ok(n)=>n,Err(e)=>0}}')
    .some(c => c.text.includes('match value {')))
})
