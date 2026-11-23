import assert from 'node:assert/strict'
import test from 'node:test'
import { registerDiagnostics } from '../src/diagnostics.cjs'

function fixture(trusted = true) {
  const handlers = {}
  const timers = new Map()
  const published = new Map()
  const messages = []
  const document = { uri: { toString: () => 'untitled:one' }, languageId: 'lobo', version: 1, isClosed: false, getText: () => 'source v1' }
  const listen = name => handler => { handlers[name] = handler; return { dispose() { delete handlers[name] } } }
  const workspace = {
    isTrusted: trusted, textDocuments: [document],
    getConfiguration: () => ({ get: (key, fallback) => key === 'compilerPath' ? '/trusted/lobo' : fallback }),
    getWorkspaceFolder: () => undefined,
    onDidOpenTextDocument: listen('open'), onDidChangeTextDocument: listen('change'),
    onDidCloseTextDocument: listen('close'), onDidChangeConfiguration: listen('configuration'),
    onDidGrantWorkspaceTrust: listen('trust')
  }
  const collection = {
    set: (uri, diagnostics) => published.set(uri.toString(), diagnostics),
    delete: uri => published.delete(uri.toString()), clear: () => published.clear(), dispose() { this.clear() }
  }
  const api = {
    workspace, languages: { createDiagnosticCollection: () => collection },
    window: { createOutputChannel: () => ({ appendLine: line => messages.push(line), dispose() {} }) },
    DiagnosticSeverity: { Error: 0 },
    Range: class { constructor(a, b, c, d) { this.start = { line: a, character: b }; this.end = { line: c, character: d } } },
    Diagnostic: class { constructor(range, message, severity) { Object.assign(this, { range, message, severity }) } }
  }
  const options = {
    scheduleTimer(callback) { const key = Symbol(); timers.set(key, callback); return key },
    cancelTimer(key) { timers.delete(key) }
  }
  const runTimers = () => { const callbacks = [...timers.values()]; timers.clear(); for (const callback of callbacks) callback() }
  return { api, workspace, document, handlers, published, messages, options, runTimers, timers }
}

const result = { kind: 'diagnostic', diagnostic: {
  code: 'compile.type.mismatch', message: 'types must match exactly',
  range: { start: { line: 0, character: 4 }, end: { line: 0, character: 8 } }
} }
const flush = async () => { for (let count = 0; count < 5; count++) await Promise.resolve() }

test('trusted current source is debounced and compiler ranges become editor diagnostics', async () => {
  const state = fixture()
  let checks = 0
  const registration = registerDiagnostics(state.api, { ...state.options, check: async (path, source) => {
    checks++; assert.equal(path, '/trusted/lobo'); assert.equal(source, 'source v1'); return result
  } })
  assert.equal(checks, 0)
  state.runTimers()
  await flush()
  assert.equal(checks, 1)
  const [diagnostic] = state.published.get('untitled:one')
  assert.equal(diagnostic.code, 'compile.type.mismatch')
  assert.equal(diagnostic.source, 'lobo')
  assert.deepEqual(diagnostic.range.start, { line: 0, character: 4 })
  registration.dispose()
})

test('untrusted workspaces never launch compiler checks, even after edit events', async () => {
  const state = fixture(false)
  const registration = registerDiagnostics(state.api, { ...state.options, check() { assert.fail('Untrusted compiler execution') } })
  state.handlers.change({ document: state.document })
  state.runTimers()
  await flush()
  assert.equal(state.published.size, 0)
  registration.dispose()
})

test('new unsaved versions cancel old checks and reject out-of-order results', async () => {
  const state = fixture()
  const checks = []
  const registration = registerDiagnostics(state.api, { ...state.options, check: (path, source, options) =>
    new Promise(resolve => checks.push({ resolve, signal: options.signal, source })) })
  state.runTimers()
  state.document.version++
  state.document.getText = () => 'source v2'
  state.handlers.change({ document: state.document })
  assert.ok(checks[0].signal.aborted)
  state.runTimers()
  assert.equal(checks[1].source, 'source v2')
  checks[1].resolve({ kind: 'ok' })
  await flush()
  checks[0].resolve(result)
  await flush()
  assert.equal(state.published.size, 0)
  registration.dispose()
})

test('closed documents and disposed controllers cannot publish late results', async () => {
  const state = fixture()
  let complete
  const registration = registerDiagnostics(state.api, { ...state.options, check: () => new Promise(resolve => { complete = resolve }) })
  state.runTimers()
  state.document.isClosed = true
  state.handlers.close(state.document)
  complete(result)
  await flush()
  assert.equal(state.published.size, 0)
  registration.dispose()
  assert.equal(state.timers.size, 0)
  assert.equal(Object.keys(state.handlers).length, 0)
})

test('compiler infrastructure failures clear stale squiggles and are reported as infrastructure', async () => {
  const state = fixture()
  const registration = registerDiagnostics(state.api, { ...state.options, check: async () => ({ kind: 'unavailable', message: 'Compiler missing' }) })
  state.runTimers()
  await flush()
  assert.equal(state.published.size, 0)
  assert.ok(state.messages.some(message => message.includes('Compiler missing')))
  registration.dispose()
})

test('trust grants and setting changes reschedule checks without changing documents', async () => {
  const state = fixture(false)
  let checks = 0
  const registration = registerDiagnostics(state.api, { ...state.options, check: async () => { checks++; return { kind: 'ok' } } })
  state.workspace.isTrusted = true
  state.handlers.trust()
  state.runTimers()
  await flush()
  assert.equal(checks, 1)
  state.handlers.configuration({ affectsConfiguration: key => key === 'lobo' })
  state.runTimers()
  await flush()
  assert.equal(checks, 2)
  registration.dispose()
})

test('at most two compiler checks run concurrently; queued documents wait for a slot', async () => {
  const state = fixture()
  state.workspace.textDocuments = [1, 2, 3].map(id => ({ ...state.document, uri: { toString: () => `untitled:${id}` } }))
  const checks = []
  const registration = registerDiagnostics(state.api, { ...state.options,
    check: () => new Promise(resolve => checks.push(resolve)) })
  state.runTimers()
  assert.equal(checks.length, 2)
  checks[0]({ kind: 'ok' })
  await flush()
  assert.equal(checks.length, 3)
  checks[1]({ kind: 'ok' })
  checks[2]({ kind: 'ok' })
  await flush()
  registration.dispose()
})

test('dependency diagnostics use their source URI and dependency changes cancel unchanged-root checks', async () => {
  const state = fixture()
  let changed
  let revision = 1
  const dependencyUri = { toString: () => 'file:dependency' }
  const projects = { onDidChange(listener) { changed = listener; return { dispose() {} } },
    async snapshot() {
      const captured = revision
      return { root: 'root', sources: [{ name: 'root', text: 'root' }, { name: 'dep', text: 'dep' }],
        documents: new Map([['dep', { uri: dependencyUri }]]), isCurrent: () => captured === revision }
    } }
  const checks = []
  const registration = registerDiagnostics(state.api, { ...state.options, projects,
    check() { assert.fail('Bundle project must not be checked as a standalone root') },
    checkBundle: (path, bundle, options) => new Promise(resolve => checks.push({ resolve, signal: options.signal, bundle })) })
  state.runTimers()
  await flush()
  assert.equal(checks.length, 1)
  checks[0].resolve({ ...result, diagnostic: { ...result.diagnostic, source: 'dep' } })
  await flush()
  assert.equal(state.published.has('untitled:one'), false)
  assert.equal(state.published.get('file:dependency')[0].code, 'compile.type.mismatch')
  changed()
  state.runTimers()
  await flush()
  revision++
  changed()
  assert.equal(checks[1].signal.aborted, true)
  state.runTimers()
  await flush()
  checks[2].resolve({ kind: 'ok' })
  await flush()
  checks[1].resolve({ ...result, diagnostic: { ...result.diagnostic, source: 'dep' } })
  await flush()
  assert.equal(state.published.size, 0)
  registration.dispose()
})
