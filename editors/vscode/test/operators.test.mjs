import assert from 'node:assert/strict'
import test from 'node:test'
import { definitionAt, definitionInBundle } from '../src/navigation.cjs'

const source = `module m @id("m");
record V @id("v") { x:I64 }
operator +(V,V)->V = add;
operator *(V,I64)->V = scale;
function add @id("add")(a:V,b:V)->V { V { x:a.x+b.x } }
function scale @id("scale")(a:V,n:I64)->V { V { x:a.x*n } }
function f @id("f")(v:V)->I64 { (v + v * 2).x }
function greeting @id("greeting")(s:Text)->Text { "Hi " + s }
`

test('operator symbols and binding targets navigate to exact named functions', () => {
  for (const [offset, target, length] of [
    [source.indexOf('= add;') + 2, source.indexOf('add @'), 3],
    [source.indexOf('v + v') + 2, source.indexOf('add @'), 1],
    [source.indexOf('v * 2') + 2, source.indexOf('scale @'), 1],
    [source.indexOf(').x }') + 2, source.indexOf('x:I64'), 1]
  ]) {
    assert.deepEqual(definitionAt(source, offset), {
      origin: { start: offset, end: offset + length },
      target: { start: target, end: target + (target === source.indexOf('scale @') ? 5 : target === source.indexOf('x:I64') ? 1 : 3) }
    })
  }
  assert.equal(definitionAt(source, source.indexOf('a.x+b.x') + 3), undefined)
  assert.equal(definitionAt(source, source.indexOf('"Hi " +') + 6), undefined)
})

test('importing a binding target enables cross-file symbol navigation with type aliases', () => {
  const app = `module app @id("app"); import Point @id("v"); import sum @id("add");
function f @id("app.f")(v:Point)->I64 { (v + v).x }`
  const offset = app.indexOf('v + v') + 2
  const target = source.indexOf('add @')
  const bundle = [{ name:'app', text:app }, { name:'lib', text:source }]
  assert.deepEqual(definitionInBundle(bundle, 'app', offset), {
    origin: { start: offset, end: offset + 1 }, target: { start: target, end: target + 3 }, source:'lib'
  })
  bundle[0].text = app.replace('import sum @id("add");','')
  assert.equal(definitionInBundle(bundle, 'app', bundle[0].text.indexOf('v + v') + 2), undefined)
})

test('invalid binding declarations do not produce a guessed definition', () => {
  for (const changed of [
    source.replace('operator +(V,V)->V = add;', 'operator +(V,V)->I64 = add;'),
    source.replace('operator +(V,V)->V = add;', 'operator +(V,V)->V = add; operator +(V,V)->V = add;'),
    source.replace('operator +(V,V)', 'operator &&(V,V)'),
    source.replace('= add;', '= absent;'),
    source.replace('record V @id("v")', 'record V<T> @id("v")'),
    source.replace('function add @id("add")(a:V,b:V)->V', 'function add @id("add")(a:V,b:V)->V actions { demo.read }')
  ]) assert.equal(definitionAt(changed, changed.indexOf('v + v') + 2), undefined)
})
