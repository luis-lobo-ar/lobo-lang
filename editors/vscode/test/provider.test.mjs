import assert from 'node:assert/strict'
import test from 'node:test'
import { createDefinitionProvider } from '../src/extension.cjs'

class Range {
  constructor(start, end) { this.start = start; this.end = end }
}

const source = 'module m @id("m"); function f @id("f")(n: I64) -> I64 { n }'
const document = {
  uri: 'untitled:test', getText: () => source,
  offsetAt: position => position, positionAt: offset => offset
}

test('provider returns exact origin/target ranges in the current document', () => {
  const provider = createDefinitionProvider({ Range })
  const result = provider.provideDefinition(document, source.lastIndexOf('n'), {})
  assert.equal(result.length, 1)
  assert.equal(result[0].targetUri, document.uri)
  assert.equal(result[0].targetSelectionRange.start, source.indexOf('n:'))
  assert.equal(result[0].targetSelectionRange.end, source.indexOf('n:') + 1)
  assert.equal(result[0].originSelectionRange.start, source.lastIndexOf('n'))
})

test('provider honors cancellation before reading the document', () => {
  const provider = createDefinitionProvider({ Range })
  const unread = { getText() { assert.fail('Canceled document must not be read') } }
  assert.equal(provider.provideDefinition(unread, 0, { isCancellationRequested: true }), undefined)
})

test('provider returns no result on punctuation or canceled completion', () => {
  const provider = createDefinitionProvider({ Range })
  assert.equal(provider.provideDefinition(document, source.lastIndexOf('}'), {}), undefined)
  let checks = 0
  const token = { get isCancellationRequested() { return ++checks > 1 } }
  assert.equal(provider.provideDefinition(document, source.lastIndexOf('n'), token), undefined)
})

test('provider uses bundle target URI/ranges and rejects stale dependency snapshots', async () => {
  const dependency = 'module b @id("b"); function f @id("b.f")(n: I64) -> I64 { n }'
  const root = 'module a @id("a"); import call @id("b.f"); function main @id("main")() -> I64 { call(42) }'
  const doc = { ...document, getText: () => root }
  let current = true
  const projects = { describe: () => true, snapshot: async () => ({ root: 'a',
    sources: [{ name: 'a', text: root }, { name: 'b', text: dependency }],
    documents: new Map([['b', { uri: 'file:dependency', text: dependency }]]), isCurrent: () => current }) }
  const provider = createDefinitionProvider({ Range }, projects)
  const result = await provider.provideDefinition(doc, root.lastIndexOf('call('), {})
  assert.equal(result[0].targetUri, 'file:dependency')
  assert.deepEqual(result[0].targetSelectionRange.start, { line: 0, character: dependency.indexOf('f @') })
  current = false
  assert.equal(await provider.provideDefinition(doc, root.lastIndexOf('call('), {}), undefined)
})
