'use strict'

const fs = require('node:fs/promises')
const { constants } = require('node:fs')
const path = require('node:path')
const LIMIT = 1_048_576
const { parseManifest } = require('./identities.cjs')

function validUnicode(text) {
  for (let index = 0; index < text.length; index++) {
    const unit = text.charCodeAt(index)
    if (unit >= 0xd800 && unit <= 0xdbff) {
      const next = text.charCodeAt(++index)
      if (!(next >= 0xdc00 && next <= 0xdfff)) return false
    } else if (unit >= 0xdc00 && unit <= 0xdfff) return false
  }
  return true
}

function inside(root, target) {
  const relative = path.relative(root, target)
  return relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative)
}

async function readSource(file, folder, budget, signal) {
  if (signal?.aborted) throw new Error('Source snapshot cancelled')
  const [base, canonical] = await Promise.all([fs.realpath(folder), fs.realpath(file)])
  if (!inside(base, canonical)) throw new Error('Source target is outside the workspace')
  const handle = await fs.open(file, constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0) | (constants.O_NONBLOCK ?? 0))
  try {
    const stat = await handle.stat()
    if (!stat.isFile()) throw new Error('Source must be a regular file')
    if (stat.size > budget) throw new Error('Source bundle exceeds its 1 MiB budget')
    const buffer = Buffer.alloc(budget + 1)
    let size = 0
    while (size < buffer.length) {
      if (signal?.aborted) throw new Error('Source snapshot cancelled')
      const { bytesRead } = await handle.read(buffer, size, buffer.length - size, null)
      if (!bytesRead) break
      size += bytesRead
    }
    if (size > budget) throw new Error('Source bundle exceeds its 1 MiB budget')
    try { return new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(buffer.subarray(0, size)) }
    catch { throw new Error('Source file is not valid UTF-8') }
  } finally { await handle.close() }
}

function createSourceProjects(vscode) {
  const workspace = vscode.workspace
  const listeners = new Set()
  const watchers = new Map()
  let epoch = 0
  let disposed = false

  function describe(document) {
    if (!workspace.isTrusted || document.uri.scheme !== 'file') return undefined
    const folder = workspace.getWorkspaceFolder(document.uri)
    if (!folder || folder.uri.scheme !== 'file') return undefined
    let files = workspace.getConfiguration('lobo', document.uri).get('sourceFiles', [])
    if (Array.isArray(files) && !files.length) {
      const relative = path.relative(folder.uri.fsPath, document.uri.fsPath).split(path.sep).join('/')
      files = [relative]
    }
    if (!Array.isArray(files) || files.length > 16 || files.some(file => typeof file !== 'string' ||
        Buffer.byteLength(file) > 1024 || !file.endsWith('.lobo') || /[\\\0*?\[\]{}:]/.test(file) ||
        file.split('/').length > 32 || file.split('/').some(part => !part || part === '.' || part === '..'))) {
      throw new Error('lobo.sourceFiles must list 1..16 explicit workspace-relative .lobo paths without traversal or wildcards')
    }
    const entries = files.map(name => ({ name, uri: vscode.Uri.joinPath(folder.uri, ...name.split('/')) }))
    const keys = entries.map(entry => entry.uri.toString())
    if (new Set(keys).size !== keys.length) throw new Error('lobo.sourceFiles contains duplicate paths')
    const root = entries.find(entry => entry.uri.toString() === document.uri.toString())
    return root ? { folder, entries, root: root.name } : undefined
  }

  function changed() {
    epoch++
    for (const listener of listeners) listener()
  }

  function watch() {
    const desired = new Map()
    for (const document of workspace.textDocuments) {
      if (document.languageId !== 'lobo' || document.isClosed) continue
      try {
        const project = describe(document)
        for (const entry of project?.entries ?? []) {
          desired.set(entry.uri.toString(), { ...entry, folder: project.folder })
          const name = `${entry.name}.ids.json`
          const uri = vscode.Uri.joinPath(project.folder.uri, ...name.split('/'))
          desired.set(uri.toString(), { name, uri, folder:project.folder })
        }
      } catch { /* Snapshot reports configuration errors; no invalid path is watched. */ }
    }
    for (const [key, watcher] of watchers) {
      if (!desired.has(key)) { watcher.dispose(); watchers.delete(key) }
    }
    if (!workspace.createFileSystemWatcher || !vscode.RelativePattern) return
    for (const [key, entry] of desired) {
      if (watchers.has(key)) continue
      const watcher = workspace.createFileSystemWatcher(new vscode.RelativePattern(entry.folder, entry.name))
      watcher.onDidChange(changed)
      watcher.onDidCreate(changed)
      watcher.onDidDelete(changed)
      watchers.set(key, watcher)
    }
  }

  const refresh = () => { watch(); changed() }
  const relevant = document => document.languageId === 'lobo' || watchers.has(document.uri.toString())
  const registrations = [
    workspace.onDidOpenTextDocument(document => { if (relevant(document)) refresh() }),
    workspace.onDidCloseTextDocument(document => { if (relevant(document)) refresh() }),
    workspace.onDidChangeTextDocument(event => { if (relevant(event.document)) changed() }),
    workspace.onDidGrantWorkspaceTrust(refresh),
    workspace.onDidChangeConfiguration(event => { if (event.affectsConfiguration('lobo')) refresh() })
  ]
  watch()

  return {
    describe,
    onDidChange(listener) { listeners.add(listener); return { dispose: () => listeners.delete(listener) } },
    async snapshot(document, signal) {
      const project = describe(document)
      if (!project || disposed) return undefined
      const generation = epoch
      const sources = []
      const documents = new Map()
      const versions = new Map()
      let bytes = 0
      for (const entry of project.entries) {
        if (signal?.aborted || disposed) throw new Error('Source snapshot cancelled')
        const key = entry.uri.toString()
        const open = workspace.textDocuments.find(doc => !doc.isClosed && doc.uri.toString() === key)
        const text = open ? open.getText() : await readSource(entry.uri.fsPath, project.folder.uri.fsPath, LIMIT - bytes, signal)
        if (open) versions.set(key, { document: open, version: open.version })
        bytes += Buffer.byteLength(text, 'utf8')
        if (bytes > LIMIT) throw new Error('Source bundle exceeds its 1 MiB budget')
        if (!validUnicode(text)) throw new Error('Source snapshot contains invalid Unicode')
        const metadataUri = vscode.Uri.joinPath(project.folder.uri, ...`${entry.name}.ids.json`.split('/'))
        const metadataKey = metadataUri.toString()
        const openMetadata = workspace.textDocuments.find(doc => !doc.isClosed && doc.uri.toString() === metadataKey)
        let metadata
        if (openMetadata) {
          metadata = openMetadata.getText()
          versions.set(metadataKey, { document:openMetadata, version:openMetadata.version })
        } else {
          try { metadata = await readSource(metadataUri.fsPath, project.folder.uri.fsPath, LIMIT - bytes, signal) }
          catch (error) { if (error.code !== 'ENOENT') throw error }
        }
        if (metadata !== undefined) {
          bytes += Buffer.byteLength(metadata)
          if (bytes > LIMIT || !validUnicode(metadata)) throw new Error('Invalid identity metadata byte budget or Unicode')
        }
        sources.push({ name: entry.name, text, ...(metadata === undefined ? {} : { identities:parseManifest(metadata) }) })
        documents.set(entry.name, { uri: entry.uri, text, document: open })
      }
      const isCurrent = () => !disposed && !signal?.aborted && workspace.isTrusted && generation === epoch &&
        [...versions.values()].every(({ document: doc, version }) => !doc.isClosed && doc.version === version)
      return { root: project.root, sources, documents, isCurrent }
    },
    dispose() {
      disposed = true
      epoch++
      listeners.clear()
      for (const registration of registrations) registration.dispose()
      for (const watcher of watchers.values()) watcher.dispose()
      watchers.clear()
    }
  }
}

module.exports = { createSourceProjects }
