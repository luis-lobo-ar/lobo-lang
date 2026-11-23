import assert from 'node:assert/strict'
import { createRequire } from 'node:module'
import test from 'node:test'

const require = createRequire(import.meta.url)
const { migrate, maintainedPath } = require('./migrate-syntax.cjs')

test('migration preserves explicit IDs, imports, comments, strings and nested match blocks', () => {
  const old = `// Keep this comment.
module app @id("stable.module");
import Reply @id("remote.reply");
import make @id("remote.make");
record Box<T> @id("stable.box") { value: T }
function main @id("stable.main")() -> Reply {
  // function fake @id("not.a.declaration")
  let code = "function fake @id(\\\"quoted\\\")";
  match (make()) { Ok(item) { match (item) { Yes { make() }, No { make() } } }, Empty { make() } }
}`
  const source = migrate(old)
  assert.match(source, /module app;/)
  assert.match(source, /use deps \{ Reply, make \};/)
  assert.match(source, /record Box<T> \{/)
  assert.match(source, /fn main\(\)/)
  assert.match(source, /Ok\(item\) => \{/)
  assert.match(source, /Yes => \{/)
  assert.ok(source.includes('// function fake @id("not.a.declaration")'))
  assert.ok(source.includes('"function fake @id(\\\"quoted\\\")"'))
  for (const id of ['stable.module', 'remote.reply', 'remote.make', 'stable.box', 'stable.main']) {
    assert.equal(source.split(`"${id}"`).length - 1, 1, id)
  }
  assert.equal(migrate(source), source)
})

test('migration chooses a noncolliding group and leaves ordinary contextual names unchanged', () => {
  const old = 'module app @id("m"); import read @id("remote.read"); function deps @id("f")()->I64 actions { deps.read } { read() }'
  const source = migrate(old)
  assert.match(source, /use deps_2 \{ read \};/)
  assert.match(source, /actions \{ deps.read \}/)
  assert.match(source, /fn deps\(\)/)
})

test('migration rejects malformed or illustrative source without partially rewriting it', () => {
  for (const source of ['module finance.transfers @id("x") machine A {}',
    'module m @id("m"); function f @id("f")()->I64 { match(x) { Case 1 } }',
    'module m @id("m"); function f @id("f")()->Text { "unterminated }']) {
    assert.throws(() => migrate(source), SyntaxError)
  }
})

test('migration write scope excludes frozen fixtures and path traversal', () => {
  for (const file of ['examples/core.lobo', 'libraries/json/json.lobo']) assert.equal(maintainedPath(file), true)
  for (const file of ['examples/v1/document.lobo', 'examples/mvc.lobo', 'examples/transfer.lobo',
    'examples/../research/fixture.lobo', '/examples/core.lobo', 'benchmarks/fixtures/repair-study/tree.lobo']) {
    assert.equal(maintainedPath(file), false, file)
  }
})
