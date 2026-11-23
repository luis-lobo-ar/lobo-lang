import assert from 'node:assert/strict'
import test from 'node:test'
import { definitionInBundle } from '../src/navigation.cjs'
import { project } from '../src/compact.cjs'
import { checkBundle } from '../src/compiler.cjs'
import { resolve } from 'node:path'
import { parseManifest } from '../src/identities.cjs'

const text = 'module sample; fn increment(n:I64)->I64 {n+1} fn answer()->I64 {increment(41)}'
const identities = { module: 'stable.module', increment: 'stable.increment', answer: 'stable.answer' }
test('managed snapshots keep navigation in original source and native compiler diagnostics', async () => {
  const sources = [{ name: 'main', text, identities }]
  const result = definitionInBundle(sources, 'main', text.lastIndexOf('increment'))
  assert.equal(result?.target.start, text.indexOf('increment'))
  assert.match(project(text, identities).text, /"stable.increment"/)
  const compiler = resolve(import.meta.dirname, '../../../target/debug/lobo')
  assert.deepEqual(await checkBundle(compiler, { root: 'main', sources }), { kind: 'ok' })
  const bad = text.replace('n+1', 'false')
  const checked = await checkBundle(compiler, { root: 'main', sources: [{ name: 'main', text: bad, identities }] })
  assert.equal(checked.kind, 'diagnostic')
  assert.equal(checked.diagnostic.range.start.byte, bad.indexOf('false'))
  assert.throws(() => project(text, { ...identities, stale: 'old.id' }))
  assert.equal(definitionInBundle([{ name:'main',text }], 'main', text.lastIndexOf('increment')), undefined)
})

test('identity sidecars reject non-JSON whitespace, duplicate escaped keys, unknown fields and schemas', () => {
  const valid = JSON.stringify({format:'lobo-identities/0.1',identities})
  assert.deepEqual(parseManifest(valid), identities)
  for (const bad of [valid.replace('{', '{\u00a0'), valid + '{}', valid.replace('lobo-identities/0.1', 'next'),
    valid.replace('"module":', '"module":"first","mod\\u0075le":'),
    valid.replace('"identities":', '"extra":"bad","identities":')]) {
    assert.throws(() => parseManifest(bad))
  }
})
