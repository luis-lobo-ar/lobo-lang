import assert from 'node:assert/strict'
import { readFileSync, mkdtempSync, existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { spawn, spawnSync } from 'node:child_process'
import { createRequire } from 'node:module'
import test from 'node:test'

const require = createRequire(import.meta.url)
const { project } = require('../src/compact.cjs')
const { definitionAt, definitionInBundle } = require('../src/navigation.cjs')
const { checkSource, checkBundle } = require('../src/compiler.cjs')
const compiler = new URL('../../../target/debug/lobo', import.meta.url).pathname
const sample = `// lobo:compact/0.1
module sample;
record Item { name: Text }
variant Lookup { Missing, Found(Item) }
fn find(id: I64) -> Lookup { Lookup::Found(Item { name: "🐺" }) }
fn show(id: I64) -> Text {
  match find(id) {
    Missing => "none",
    Found(item) => item.name
  }
}
identities {
  module: "compact.sample",
  Item: "compact.item",
  Lookup: "compact.lookup",
  find: "compact.find",
  show: "compact.show"
}
`

test('compact source checks with persistent identities and original-source navigation', async () => {
  const lowered = project(sample)
  assert.match(lowered.text, /function find @ id \( "compact.find" \)/)
  assert.equal((await checkSource(compiler, sample)).kind, 'ok')
  for (const [use, target] of [['find(id)', 'find(id:'], ['item.name', 'item) =>'], ['name\n', 'name: Text']]) {
    const offset = sample.lastIndexOf(use)
    const actual = definitionAt(sample, offset)
    assert.ok(actual, use)
    assert.equal(actual.target.start, sample.indexOf(target))
  }
  // Rename only the source binding and metadata key, retaining the external entry ID.
  const stable = sample.replace(/\bfind(?=\(|:)/g, 'lookup')
  assert.match(project(stable).text, /function lookup @ id \( "compact.find" \)/)
  assert.equal((await checkSource(compiler, stable)).kind, 'ok')
})

test('modern source needs no marker and the diagnostic adapter sends original text to the compiler', async () => {
  const source = sample
  let sent
  const result = await checkSource(compiler, source, { spawnProcess(executable, args, options) {
    const child = spawn(executable, args, options)
    const original = child.stdin.end.bind(child.stdin)
    child.stdin.end = (input, ...rest) => { sent = input; return original(input, ...rest) }
    return child
  } })
  assert.equal(sent, source)
  assert.equal(result.kind, 'ok')
  const plain = source.replace('// lobo:compact/0.1\n', '')
  assert.equal((await checkSource(compiler, plain)).kind, 'ok')
  assert.equal(definitionAt(plain, plain.lastIndexOf('find(id)')).target.start, plain.indexOf('find(id:'))
})

test('generic modern declarations and block arms retain structural navigation', async () => {
  const source = `module generic;
record Box<T> { value: T }
variant Option<T> { None, Some(T) }
fn identity<T>(value:T)->T {value}
fn apply<T,U>(f:Fn(T)->U,value:T)->U {f(value)}
fn main()->I64 {
  let item = Option::<Box<I64>>::Some(Box::<I64> {value:41});
  match item { None => 0, Some(box) => { let callback = &identity::<I64>; apply::<I64,I64>(callback,box.value) + 1 } }
}
identities {module:"nav.generic", Box:"nav.box", Option:"nav.option",identity:"nav.identity",apply:"nav.apply",main:"nav.main"}`
  assert.equal((await checkSource(compiler, source)).kind, 'ok')
  for (const [use, target] of [['box.value','value: T'], ['identity::<I64>', 'identity<T>'], ['apply::<I64,I64>', 'apply<T,U>']]) {
    const offset = source.indexOf(use) + (use === 'box.value' ? 4 : 0)
    assert.equal(definitionAt(source, offset)?.target.start, source.indexOf(target), use)
  }
})

test('compact diagnostics retain exact Unicode/CRLF source ranges', async () => {
  const bad = sample.replace('item.name\n', 'false\n').replaceAll('\n', '\r\n')
  const result = await checkSource(compiler, bad)
  assert.equal(result.kind, 'diagnostic')
  assert.equal(result.diagnostic.code, 'compile.type.mismatch')
  const range = result.diagnostic.range
  assert.equal(Buffer.from(bad).subarray(range.start.byte, range.end.byte).toString(), 'false')
  assert.equal(range.start.line, 8)
})

test('compact projection fails closed for missing, stale and duplicate identity metadata', async () => {
  for (const bad of [
    sample.replace('  find: "compact.find",\n', ''),
    sample.replace('  find: "compact.find",', '  stale: "compact.find",'),
    sample.replace('  find: "compact.find",', '  find: "compact.find", find: "another",'),
    sample.replace('"compact.find"', '"bad id"'),
    sample.replace('fn find', 'function find'),
    sample.replace('match find(id)', 'match Item { name: "x" }'),
    sample.replace('Missing => "none"', 'Missing =>'),
  ]) {
    assert.throws(() => project(bad), SyntaxError)
    assert.equal((await checkSource(compiler, bad)).kind, 'diagnostic')
    assert.equal(definitionAt(bad, bad.lastIndexOf('find(id)')), undefined)
  }
  assert.equal(project('module m @id("m");').text, 'module m @id("m");')
})

test('actual compact MVC checks and qualified calls navigate to supplied definitions', async () => {
  const sources = ['examples/mvc.compact.lobo', 'libraries/http/http.lobo', 'libraries/json/json.lobo']
    .map(name => ({ name, text: readFileSync(new URL(`../../../${name}`, import.meta.url), 'utf8') }))
  sources[0].identities = JSON.parse(readFileSync(new URL('../../../examples/mvc.compact.lobo.ids.json', import.meta.url), 'utf8')).identities
  const bundle = { root: sources[0].name, sources }
  assert.equal((await checkBundle(compiler, bundle)).kind, 'ok')
  const source = sources[0].text
  const offset = source.indexOf('json.json_response') + 5
  const target = definitionInBundle(sources, bundle.root, offset)
  assert.ok(target)
  assert.equal(target.source, 'libraries/json/json.lobo')
  assert.equal(target.target.start, sources[2].text.indexOf('fn json_response(') + 3)
  const fieldUse = source.indexOf('user.name') + 5
  assert.equal(definitionInBundle(sources, bundle.root, fieldUse).target.start, source.indexOf('name: Text'))
  const bad = { ...bundle, sources: [{ ...sources[0], text: source.replace('user.name', 'true') }, ...sources.slice(1)] }
  const result = await checkBundle(compiler, bad)
  assert.equal(result.kind, 'diagnostic')
  assert.equal(result.diagnostic.source, bundle.root)
  assert.equal(Buffer.from(bad.sources[0].text).subarray(result.diagnostic.range.start.byte, result.diagnostic.range.end.byte).toString(), 'true')
  for (const text of [source.replace('"name": user.name', '"name": user.name, "na\\u006de": user.name'),
    source.replace('http.not_found()', 'http.unselected()'),
    source.replace('object, field,', 'field,'),
    source.replace('"lobo.json.field"', '"wrong.field"'),
    source.replace('fn show_user', 'fn http'),
  ]) assert.throws(() => project(text), SyntaxError)
})

test('comments and quoted code remain data, with bounded projection and no stale IDs', async () => {
  const quoted = sample.replace('"🐺"', '"fn fake() { match x { Fake => json {} } } // 🐺"')
  assert.equal((await checkSource(compiler, quoted)).kind, 'ok')
  const commented = sample.replace('fn show', '// fn pretend() -> Text { "ignored" }\nfn show')
  assert.equal((await checkSource(compiler, commented)).kind, 'ok')
  assert.throws(() => project(sample.replace('"🐺"', '"\\ud800"')), SyntaxError)
  assert.throws(() => project(`${sample}${' '.repeat(1_048_576)}`), /1 MiB/)
  assert.throws(() => project(sample.replace('name: Text', `${'a '.repeat(8193)}name: Text`)), /8192 tokens/)
  assert.throws(() => project(sample.replace('item.name', `${'('.repeat(34)}item.name${')'.repeat(34)}`)), /32 levels/)
  assert.equal(definitionAt(sample, sample.indexOf('"compact.find"') + 2), undefined)
})

test('projection CLI uses explicit sources and refuses to overwrite generated output', () => {
  const scratch = mkdtempSync(join(tmpdir(), 'lobo-compact-cli-'))
  const directory = join(scratch, 'projection')
  const root = new URL('../../../', import.meta.url).pathname
  const invoke = args => spawnSync(process.execPath, ['scripts/compact-syntax.cjs', ...args], { cwd: root, encoding: 'utf8' })
  const args = ['examples/mvc.compact.lobo', '--source', 'libraries/http/http.lobo', '--source', 'libraries/json/json.lobo']
  const result = invoke([...args, '--out-dir', directory])
  assert.equal(result.status, 0, result.stderr)
  const receipt = JSON.parse(result.stdout)
  assert.match(readFileSync(receipt.root, 'utf8'), /function show_user/)
  assert.equal(JSON.parse(readFileSync(join(directory, 'inputs.json'))).sources[0].text, readFileSync(join(root, args[0]), 'utf8'))
  const before = readFileSync(receipt.root, 'utf8')
  assert.equal(invoke([...args, '--out-dir', directory]).status, 1)
  assert.equal(readFileSync(receipt.root, 'utf8'), before)
  const missing = join(scratch, 'missing')
  assert.equal(invoke(['no-such-compact-source.lobo', '--out-dir', missing]).status, 1)
  assert.equal(existsSync(missing), false)
  assert.equal(invoke([...args, '--source', args[0]]).status, 1)
  assert.equal(invoke([...args, '--guess-dependencies']).status, 1)
})
