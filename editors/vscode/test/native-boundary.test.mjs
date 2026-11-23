import assert from 'node:assert/strict'
import test from 'node:test'
import {definitionAt,definitionInBundle} from '../src/navigation.cjs'
import {checkSource} from '../src/compiler.cjs'
import {resolve} from 'node:path'

const source=`module native_editor;
record Input {value:I64}
fn integer(input:Input)->I64 {Native::roundtrip_i64(input.value)}
fn inspect(listener:Native::Listener)->I64 actions {net.inspect} {
  match Native::port(listener) {Err(code)=>code,Ok(port)=>port}
}
identities {module:"native.editor",Input:"native.editor.input",integer:"native.editor.integer",inspect:"native.editor.inspect"}`

test('native arguments and opaque types preserve ordinary definition navigation',()=>{
  assert.equal(definitionAt(source,source.indexOf('input.value')+6)?.target.start,source.indexOf('value:I64'))
  assert.equal(definitionAt(source,source.indexOf('port(listener)')+5)?.target.start,source.indexOf('listener:Native'))
  assert.equal(definitionAt(source,source.lastIndexOf('=>port')+2)?.target.start,source.indexOf('Ok(port)')+3)
  assert.equal(definitionAt(source,source.indexOf('Native::port')),undefined)
  assert.equal(definitionAt(source,source.indexOf('Native::Listener')),undefined)
})

test('native operation errors use original compiler ranges',async()=>{
  const compiler=resolve(import.meta.dirname,'../../../target/debug/lobo')
  assert.equal((await checkSource(compiler,source)).kind,'ok')
  const bad=source.replace('input.value','true')
  const result=await checkSource(compiler,bad)
  assert.equal(result.kind,'diagnostic')
  assert.equal(result.diagnostic.code,'compile.type.mismatch')
  assert.equal(result.diagnostic.range.start.byte,bad.indexOf('true'))
})

test('native types also work in explicit source bundles',()=>{
  const target=definitionInBundle([{name:'app',text:source}],'app',source.indexOf('input.value')+6)
  assert.equal(target?.target.start,source.indexOf('value:I64'))
  assert.equal(target?.source,'app')
})

test('legacy Native variants keep ordinary constructor navigation',()=>{
  const old='module old @id("old"); variant Native @id("old.native") { Case(I64) } function f @id("old.f")()->Native {Native::Case(42)}'
  assert.equal(definitionAt(old,old.lastIndexOf('Case('))?.target.start,old.indexOf('Case(I64)'))
})

test('SQLite result payloads and handles retain navigation through try',async()=>{
  const text='module m; record User {id:I64} fn lookup(cap:Native::SqliteCapability,user:User)->Native::I64Result actions {db.open,db.close} {let db=try Native::sqlite_open(cap); let closed=try Native::sqlite_close(db); Ok(user.id)} identities {module:"sqlite.editor",User:"sqlite.editor.user",lookup:"sqlite.editor.lookup"}'
  assert.equal(definitionAt(text,text.indexOf('user.id')+5)?.target.start,text.indexOf('id:I64'))
  assert.equal(definitionAt(text,text.indexOf('sqlite_close(db)')+13)?.target.start,text.indexOf('db=try'))
  const compiler=resolve(import.meta.dirname,'../../../target/debug/lobo')
  assert.equal((await checkSource(compiler,text)).kind,'ok')
  const bad=text.replace('sqlite_close(db)','sqlite_close(1)')
  const result=await checkSource(compiler,bad)
  assert.equal(result.kind,'diagnostic')
  assert.equal(result.diagnostic.code,'compile.type.mismatch')
})
