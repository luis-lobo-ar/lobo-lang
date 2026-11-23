import assert from 'node:assert/strict'
import { EventEmitter } from 'node:events'
import { PassThrough } from 'node:stream'
import { resolve } from 'node:path'
import test from 'node:test'
import { setTimeout as delay } from 'node:timers/promises'
import { checkSource } from '../src/compiler.cjs'

const compiler = resolve(import.meta.dirname, '../../../target/debug/lobo')
const source = 'module m @id("m"); function f @id("f")() -> I64 { false }'

test('source checker accepts operator bindings and reports target-signature errors', async () => {
  const text = 'module m @id("m"); record V @id("v") { x:I64 } operator +(V,V)->V = add; function add @id("add")(a:V,b:V)->V {a} function f @id("f")(v:V)->V {v+v}'
  assert.equal((await checkSource(compiler,text)).kind,'ok')
  const result = await checkSource(compiler,text.replace('+(V,V)->V','+(V,V)->I64'))
  assert.equal(result.kind,'diagnostic')
  assert.equal(result.diagnostic.code,'compile.operator.signature')
})

test('source checker accepts explicit generic calls and diagnoses concrete type errors', async () => {
  const text = 'module m @id("m"); function id<T> @id("id")(x: T) -> T { x } function f @id("f")() -> I64 { id::<I64>(7) }'
  assert.equal((await checkSource(compiler,text)).kind,'ok')
  const result = await checkSource(compiler,text.replace('id::<I64>(7)','id::<Bool>(7)'))
  assert.equal(result.kind,'diagnostic')
  assert.equal(result.diagnostic.code,'compile.type.mismatch')
})

test('source checker accepts list-guarded recursive data but diagnoses inline recursion', async () => {
  const recursive = 'module m @id("m"); variant Node @id("node") { Num(I64), Sum(List<Node>) } function f @id("f")() -> Node { Node::Sum(List::empty<Node>()) }'
  assert.equal((await checkSource(compiler, recursive)).kind, 'ok')
  const result = await checkSource(compiler, recursive.replace('Sum(List<Node>)', 'Sum(Node)'))
  assert.equal(result.kind, 'diagnostic')
  assert.equal(result.diagnostic.code, 'compile.record.cycle')
})

function fakeProcess(stdout, code = 0) {
  const child = new EventEmitter()
  child.stdin = new PassThrough()
  child.stdout = new PassThrough()
  child.stderr = new PassThrough()
  child.kill = () => { queueMicrotask(() => child.emit('close', null, 'SIGTERM')); return true }
  child.stdin.on('finish', () => queueMicrotask(() => {
    if (stdout === null) return
    child.stdout.write(stdout)
    child.emit('close', code)
  }))
  return child
}

test('source checker consumes real compiler JSON for unsaved source and success', async () => {
  const failure = await checkSource(compiler, source)
  assert.equal(failure.kind, 'diagnostic')
  assert.equal(failure.diagnostic.code, 'compile.type.mismatch')
  assert.equal(failure.diagnostic.range.start.byte, source.indexOf('false'))
  assert.equal((await checkSource(compiler, source.replace('false', '1'))).kind, 'ok')
})

test('source checker reports action-bound failures from the real compiler', async () => {
  const text = 'module m @id("m"); function main @id("main")() -> I64 { leaf() } function leaf @id("leaf")() -> I64 actions { demo.observe } { 42 }'
  const result = await checkSource(compiler, text)
  assert.equal(result.kind, 'diagnostic')
  assert.equal(result.diagnostic.code, 'compile.action.undeclared')
  assert.equal(text.slice(result.diagnostic.range.start.byte, result.diagnostic.range.end.byte), 'leaf()')
  assert.equal((await checkSource(compiler, text.replace('-> I64 { leaf()', '-> I64 actions { demo.observe } { leaf()'))).kind, 'ok')
})

test('compiler process has fixed arguments, piped stdin, and no shell', async () => {
  const child = fakeProcess('{"status":"ok","language":"core/0.2-draft","entries":["f"]}')
  let actual = ''
  child.stdin.on('data', chunk => { actual += chunk })
  const result = await checkSource('/trusted/compiler with spaces', source, {
    spawnProcess(executable, args, options) {
      assert.equal(executable, '/trusted/compiler with spaces')
      assert.deepEqual(args, ['check', '--stdin'])
      assert.equal(options.shell, false)
      return child
    }
  })
  assert.equal(actual, source)
  assert.equal(result.kind, 'ok')
})

test('missing compiler, malformed output, and legacy CLI errors are infrastructure, not squiggles', async () => {
  assert.equal((await checkSource('/nonexistent-lobo-test-compiler', source)).kind, 'unavailable')
  for (const output of ['broken JSON', '{"status":"ok"}', 'null', '[]',
    '{"status":"error","kind":"invocation_error","diagnostic":{"code":"invocation.file.read","message":"not found"}}']) {
    const result = await checkSource('test', source, { spawnProcess: () => fakeProcess(output, 1) })
    assert.equal(result.kind, 'unavailable')
  }
})

test('a timed-out compiler that ignores graceful termination is force-killed', async () => {
  const child = fakeProcess(null)
  const signals = []
  child.kill = signal => {
    signals.push(signal)
    if (signal === 'SIGKILL') queueMicrotask(() => child.emit('close', null, signal))
    return true
  }
  const result = await checkSource('test', source, {
    timeoutMs: 5, killGraceMs: 10, spawnProcess: () => child
  })
  assert.equal(result.kind, 'unavailable')
  await delay(25)
  assert.deepEqual(signals, ['SIGTERM', 'SIGKILL'])
})

test('cancellation, timeout, and output budgets terminate checks without false diagnostics', async () => {
  const abort = new AbortController()
  abort.abort()
  const cancelled = await checkSource('test', source, {
    signal: abort.signal, spawnProcess() { assert.fail('Already canceled') }
  })
  assert.equal(cancelled.kind, 'cancelled')
  const timed = await checkSource('test', source, { timeoutMs: 5, spawnProcess: () => fakeProcess(null) })
  assert.equal(timed.kind, 'unavailable')
  assert.match(timed.message, /time/i)
  const excessive = await checkSource('test', source, { maxOutputBytes: 8, spawnProcess: () => fakeProcess('x'.repeat(16)) })
  assert.equal(excessive.kind, 'unavailable')
})

test('malformed/missing ranges are not converted into guessed error locations', async () => {
  const diagnostic = {
    code: 'compile.type.mismatch', phase: 'check', message: 'types must match',
    range: { start: { byte: 0, line: 9000, character: 0 }, end: { byte: 1, line: 9000, character: 1 } }
  }
  for (const range of [diagnostic.range, undefined]) {
    const output = JSON.stringify({ status: 'error', kind: 'compile_error', diagnostic: { ...diagnostic, range } })
    assert.equal((await checkSource('test', source, { spawnProcess: () => fakeProcess(output, 1) })).kind, 'unavailable')
  }
})

test('frontend budget failures are reported as limits, not an instruction to upgrade a working compiler', async () => {
  const output = JSON.stringify({ status: 'error', kind: 'resource_exhausted', diagnostic: {
    code: 'resource.frontend.limit', phase: 'parse', message: 'source size limit exceeded'
  } })
  const result = await checkSource('test', source, { spawnProcess: () => fakeProcess(output, 1) })
  assert.equal(result.kind, 'unavailable')
  assert.match(result.message, /resource limit/i)
  assert.ok(!result.message.includes('0.2.1+'))
})

test('source checker accepts core 1.8 try and locates error payload mismatches', async () => {
  const source='module m; variant R {Ok(I64),Err(Text)} fn get()->R {Ok(42)} fn main()->R {let n=try get(); Ok(n)} identities {module:"m",R:"r",get:"get",main:"main"}'
  assert.equal((await checkSource(compiler,source)).kind,'ok')
  const failure=await checkSource(compiler,source.replace('try get()','try 42'))
  assert.equal(failure.kind,'diagnostic')
  assert.equal(failure.diagnostic.code,'compile.try.operand')
  assert.equal(failure.diagnostic.range.start.byte,source.indexOf('try get()'))
})
