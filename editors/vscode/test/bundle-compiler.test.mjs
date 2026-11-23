import assert from 'node:assert/strict'
import test from 'node:test'
import { createRequire } from 'node:module'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { EventEmitter } from 'node:events'
import { PassThrough } from 'node:stream'
const require = createRequire(import.meta.url)
const compiler = require('../src/compiler.cjs')
const executable = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../target/debug/lobo')
const root = 'module a @id("a"); import plus @id("b.f"); function main @id("main")() -> I64 { plus(41) }'
const library = 'module b @id("b"); function f @id("b.f")(n: I64) -> I64 { n + 1 }'
const bundle = { root: 'a', sources: [{ name: 'a', text: root }, { name: 'b', text: library }] }

test('bundle checker preserves defining-source diagnostics for generic specialization errors', async () => {
  const app = 'module a @id("a"); import add @id("add"); function f @id("f")() -> Bool { add::<Bool>(true) }'
  const lib = 'module b @id("b"); function add<T> @id("add")(x: T) -> T { x + x }'
  const result = await compiler.checkBundle(executable,{root:'a',sources:[{name:'a',text:app},{name:'b',text:lib}]})
  assert.equal(result.kind,'diagnostic')
  assert.equal(result.diagnostic.source,'b')
  assert.equal(result.diagnostic.code,'compile.type.mismatch')
  assert.equal(result.diagnostic.range.start.byte,lib.indexOf('x + x'))
})

test('operator lowering keeps generic local-binding errors attached to their source', async () => {
  const app = 'module a @id("a"); import bad @id("bad"); function f @id("f")()->I64 { bad::<I64>(1) }'
  const lib = 'module b @id("b"); function bad<T> @id("bad")(x:T)->T { let y = x+x; y }'
  const result = await compiler.checkBundle(executable,{root:'a',sources:[{name:'a',text:app},{name:'b',text:lib}]})
  assert.equal(result.kind,'diagnostic')
  assert.equal(result.diagnostic.code,'compile.type.mismatch')
  assert.equal(result.diagnostic.source,'b')
  assert.equal(result.diagnostic.range.start.byte,lib.indexOf('x+x'))
})

test('bundle checker uses caller-supplied snapshots and validates diagnostic ranges against the named dependency', async () => {
  assert.deepEqual(await compiler.checkBundle(executable, bundle), { kind: 'ok' })
  const broken = { ...bundle, sources: [bundle.sources[0], { name: 'b', text: library.replace('n + 1', 'missing') }] }
  const result = await compiler.checkBundle(executable, broken)
  assert.equal(result.kind, 'diagnostic')
  assert.equal(result.diagnostic.source, 'b')
  assert.equal(result.diagnostic.range.start.character, broken.sources[1].text.indexOf('missing'))
  const cancelled = await compiler.checkBundle(executable, bundle, { signal: AbortSignal.abort() })
  assert.equal(cancelled.kind, 'cancelled')
})

function reply(output, code, inspect = () => {}) {
  return (executable, args, options) => {
    assert.deepEqual(args, ['check', '--bundle-stdin'])
    assert.equal(options.shell, false)
    const child = new EventEmitter()
    Object.assign(child, { stdin: new PassThrough(), stdout: new PassThrough(), stderr: new PassThrough() })
    let input = ''
    child.stdin.on('data', chunk => { input += chunk })
    child.stdin.on('finish', () => queueMicrotask(() => {
      inspect(JSON.parse(input))
      child.stdout.write(JSON.stringify(output))
      child.emit('close', code)
    }))
    return child
  }
}

test('bundle adapter rejects unknown source labels, mismatched ranges, old success versions and oversized input before spawn', async () => {
  const diagnostic = { code: 'compile.syntax', phase: 'parse', message: 'bad input', source: 'missing',
    range: { start: { byte: 0, line: 0, character: 0 }, end: { byte: 1, line: 0, character: 1 } } }
  for (const detail of [diagnostic, { ...diagnostic, source: 'b', range: { start: { byte: 0, line: 9, character: 0 }, end: { byte: 1, line: 9, character: 1 } } }]) {
    const result = await compiler.checkBundle('fake', bundle, { spawnProcess: reply({ status: 'error', kind: 'compile_error', diagnostic: detail }, 1) })
    assert.equal(result.kind, 'unavailable')
  }
  const old = await compiler.checkBundle('fake', bundle, { spawnProcess: reply({ status: 'ok', language: 'core/0.8-draft', entries: [] }, 0) })
  assert.equal(old.kind, 'unavailable')
  const success = await compiler.checkBundle('fake', bundle, { spawnProcess: reply({ status: 'ok', language: 'core/0.9-draft', entries: ['main'] }, 0,
    input => assert.deepEqual(input, { format: 'lobo-source/0.1', ...bundle })) })
  assert.equal(success.kind, 'ok')
  for (const text of ['x'.repeat(1_048_577), '\0'.repeat(900_000)]) {
    assert.equal((await compiler.checkBundle('fake', { root: 'a', sources: [{ name: 'a', text }] }, {
      spawnProcess() { assert.fail('Over-budget source must not start a compiler') }
    })).kind, 'unavailable')
  }
})
