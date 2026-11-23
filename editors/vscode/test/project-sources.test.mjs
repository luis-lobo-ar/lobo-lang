import assert from 'node:assert/strict'
import test from 'node:test'
import { createRequire } from 'node:module'
import { mkdtemp, writeFile, mkdir, symlink, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { pathToFileURL } from 'node:url'
import { execFileSync } from 'node:child_process'
const require = createRequire(import.meta.url)
const projects = require('../src/project.cjs')

function uri(file) { return { scheme: 'file', fsPath: file, toString: () => pathToFileURL(file).href } }
async function fixture(t) {
  const directory = await mkdtemp(path.join(tmpdir(), 'lobo-editor-project-'))
  t.after(() => rm(directory, { recursive: true, force: true }))
  await writeFile(path.join(directory, 'root.lobo'), 'root on disk')
  await writeFile(path.join(directory, 'lib.lobo'), 'dependency on disk')
  let files = ['root.lobo', 'lib.lobo']
  const document = { uri: uri(path.join(directory, 'root.lobo')), languageId: 'lobo', version: 1,
    isClosed: false, getText: () => 'unsaved root' }
  const handlers = {}
  const watch = []
  const listen = key => handler => { handlers[key] = handler; return { dispose() { delete handlers[key] } } }
  const workspace = { isTrusted: true, textDocuments: [document],
    getWorkspaceFolder: () => ({ uri: uri(directory) }),
    getConfiguration: () => ({ get: (key, fallback) => key === 'sourceFiles' ? files : fallback }),
    onDidOpenTextDocument: listen('open'), onDidChangeTextDocument: listen('change'),
    onDidCloseTextDocument: listen('close'), onDidChangeConfiguration: listen('configuration'),
    onDidGrantWorkspaceTrust: listen('trust'),
    createFileSystemWatcher(pattern) {
      const callbacks = {}
      const watcher = { pattern, callbacks, disposed: false, dispose() { this.disposed = true } }
      for (const event of ['Change', 'Create', 'Delete']) watcher[`onDid${event}`] = fn => { callbacks[event] = fn; return { dispose() {} } }
      watch.push(watcher)
      return watcher
    }
  }
  const api = { workspace, Uri: { joinPath: (base, ...parts) => uri(path.join(base.fsPath, ...parts)) },
    RelativePattern: class { constructor(base, pattern) { this.base = base; this.pattern = pattern } } }
  return { api, workspace, document, directory, handlers, watch, setFiles(value) { files = value } }
}

test('explicit source snapshots overlay unsaved text and invalidate on dependency versions and file events', async t => {
  const state = await fixture(t)
  const project = projects.createSourceProjects(state.api)
  t.after(() => project.dispose())
  const dependency = { ...state.document, uri: uri(path.join(state.directory, 'lib.lobo')), getText: () => 'unsaved dependency' }
  state.workspace.textDocuments.push(dependency)
  const snapshot = await project.snapshot(state.document)
  assert.deepEqual(snapshot.sources, [{ name: 'root.lobo', text: 'unsaved root' }, { name: 'lib.lobo', text: 'unsaved dependency' }])
  assert.equal(snapshot.root, 'root.lobo')
  assert.equal(snapshot.isCurrent(), true)
  dependency.version++
  assert.equal(snapshot.isCurrent(), false)
  const next = await project.snapshot(state.document)
  let invalidations = 0
  const listener = project.onDidChange(() => invalidations++)
  assert.deepEqual(state.watch.map(w => w.pattern.pattern).sort(), ['lib.lobo', 'lib.lobo.ids.json', 'root.lobo', 'root.lobo.ids.json'])
  state.watch[0].callbacks.Change()
  assert.equal(next.isCurrent(), false)
  assert.equal(invalidations, 1)
  listener.dispose()
})

test('trust, explicit membership, path containment and strict bounded UTF-8 reads are enforced', async t => {
  const state = await fixture(t)
  const project = projects.createSourceProjects(state.api)
  t.after(() => project.dispose())
  state.workspace.isTrusted = false
  assert.equal(await project.snapshot(state.document), undefined)
  state.workspace.isTrusted = true
  state.setFiles(['lib.lobo'])
  assert.equal(await project.snapshot(state.document), undefined)
  for (const invalid of [['../escape.lobo'], ['/root.lobo'], ['*.lobo'], ['root.lobo', 'root.lobo'], ['a\\b.lobo'], Array(17).fill('root.lobo')]) {
    state.setFiles(invalid)
    await assert.rejects(() => project.snapshot(state.document), /sourceFiles/)
  }
  state.setFiles(['root.lobo', 'lib.lobo'])
  await writeFile(path.join(state.directory, 'lib.lobo'), Buffer.alloc(1_048_577, 0xff))
  await assert.rejects(() => project.snapshot(state.document), /budget/)
  await writeFile(path.join(state.directory, 'lib.lobo'), Buffer.from([0xff]))
  await assert.rejects(() => project.snapshot(state.document), /UTF-8/)
  await mkdir(path.join(state.directory, 'inside'))
  await symlink(tmpdir(), path.join(state.directory, 'outside'))
  state.setFiles(['root.lobo', 'outside/escape.lobo'])
  await assert.rejects(() => project.snapshot(state.document), /workspace|ENOENT/)
  state.setFiles(['root.lobo', 'missing.lobo'])
  await assert.rejects(() => project.snapshot(state.document), /ENOENT/)
})

test('special-file sources fail without waiting for a FIFO writer', { skip: process.platform === 'win32' }, async t => {
  const state = await fixture(t)
  execFileSync('mkfifo', [path.join(state.directory, 'pipe.lobo')])
  state.setFiles(['root.lobo', 'pipe.lobo'])
  const project = projects.createSourceProjects(state.api)
  t.after(() => project.dispose())
  await assert.rejects(() => Promise.race([project.snapshot(state.document),
    new Promise((resolve, reject) => { const timer = setTimeout(() => reject(new Error('FIFO read blocked')), 1000); timer.unref() })]), /regular file/)
})

test('Unicode validation does not require newer String methods in older supported extension hosts', async t => {
  const state = await fixture(t)
  const project = projects.createSourceProjects(state.api)
  t.after(() => project.dispose())
  const descriptor = Object.getOwnPropertyDescriptor(String.prototype, 'isWellFormed')
  Object.defineProperty(String.prototype, 'isWellFormed', { value: undefined, configurable: true })
  try {
    assert.ok(await project.snapshot(state.document))
    state.document.getText = () => 'unpaired \ud800'
    await assert.rejects(() => project.snapshot(state.document), /Unicode/)
  } finally {
    if (descriptor) Object.defineProperty(String.prototype, 'isWellFormed', descriptor)
    else delete String.prototype.isWellFormed
  }
})

test('unrelated output/plaintext edits cannot trigger a diagnostic refresh loop, but listed plaintext dependencies invalidate', async t => {
  const state = await fixture(t)
  const project = projects.createSourceProjects(state.api)
  t.after(() => project.dispose())
  const snapshot = await project.snapshot(state.document)
  let invalidations = 0
  project.onDidChange(() => invalidations++)
  state.handlers.change({ document: { languageId: 'plaintext', uri: uri(path.join(state.directory, 'output.log')) } })
  assert.equal(invalidations, 0)
  assert.equal(snapshot.isCurrent(), true)
  state.handlers.change({ document: { languageId: 'plaintext', uri: uri(path.join(state.directory, 'lib.lobo')) } })
  assert.equal(invalidations, 1)
  assert.equal(snapshot.isCurrent(), false)
})

test('managed sidecars overlay unsaved metadata, invalidate snapshots and reject duplicate or invalid bindings', async t => {
  const state = await fixture(t)
  const metadata = path.join(state.directory, 'root.lobo.ids.json')
  const manifest = { format: 'lobo-identities/0.1', identities: { module: 'stable.module' } }
  await writeFile(metadata, JSON.stringify(manifest))
  const project = projects.createSourceProjects(state.api)
  t.after(() => project.dispose())
  const first = await project.snapshot(state.document)
  assert.deepEqual(first.sources[0].identities, manifest.identities)
  assert.ok(state.watch.some(w => w.pattern.pattern === 'root.lobo.ids.json'))
  const open = { uri: uri(metadata), languageId:'json', version:1, isClosed:false,
    getText: () => JSON.stringify({ ...manifest, identities:{module:'unsaved.module'} }) }
  state.workspace.textDocuments.push(open)
  state.handlers.open(open)
  assert.equal(first.isCurrent(), false)
  const second = await project.snapshot(state.document)
  assert.equal(second.sources[0].identities.module, 'unsaved.module')
  open.version++
  assert.equal(second.isCurrent(), false)
  open.getText = () => '{"format":"lobo-identities/0.1","identities":{"module":"one","module":"two"}}'
  await assert.rejects(() => project.snapshot(state.document), /duplicate/i)
  open.getText = () => JSON.stringify({ ...manifest, identities:{module:'bad id'} })
  await assert.rejects(() => project.snapshot(state.document), /identity/i)
  state.workspace.isTrusted = false
  assert.equal(await project.snapshot(state.document), undefined)
})
