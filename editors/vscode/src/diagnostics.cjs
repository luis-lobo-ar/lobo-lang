'use strict'

const { checkSource, checkBundle } = require('./compiler.cjs')

function registerDiagnostics(vscode, options = {}) {
  const collection = vscode.languages.createDiagnosticCollection('lobo')
  const output = vscode.window.createOutputChannel('Lobo Compiler')
  const pending = new Map()
  const contributions = new Map()
  const queue = []
  const check = options.check ?? checkSource
  const bundleCheck = options.checkBundle ?? checkBundle
  const projects = options.projects
  const scheduleTimer = options.scheduleTimer ?? setTimeout
  const cancelTimer = options.cancelTimer ?? clearTimeout
  let active = 0
  let disposed = false

  function current(job) {
    return !disposed && !job.document.isClosed && vscode.workspace.isTrusted &&
      job.document.version === job.version && pending.get(job.key) === job &&
      (!job.snapshot || job.snapshot.isCurrent())
  }

  function render() {
    collection.clear()
    const targets = new Map()
    for (const { uri, diagnostic } of contributions.values()) {
      const key = uri.toString()
      if (!targets.has(key)) targets.set(key, { uri, diagnostics: new Map() })
      targets.get(key).diagnostics.set(JSON.stringify([diagnostic.code, diagnostic.range, diagnostic.message]), diagnostic)
    }
    for (const { uri, diagnostics } of targets.values()) collection.set(uri, [...diagnostics.values()])
  }

  function remove(document) {
    const key = document.uri.toString()
    const old = pending.get(key)
    if (old) { cancelTimer(old.timer); old.controller.abort() }
    pending.delete(key)
    contributions.delete(key)
    render()
  }

  function publish(job, result) {
    if (!current(job)) return
    if (result.kind === 'diagnostic') {
      const detail = result.diagnostic
      const target = job.snapshot ? job.snapshot.documents.get(detail.source)?.uri : job.document.uri
      if (!target) { output.appendLine('Compiler diagnostic has no matching source snapshot'); return }
      const { start, end } = detail.range
      const diagnostic = new vscode.Diagnostic(new vscode.Range(start.line, start.character, end.line, end.character),
        detail.message, vscode.DiagnosticSeverity.Error)
      diagnostic.source = 'lobo'
      diagnostic.code = detail.code
      contributions.set(job.key, { uri: target, diagnostic })
      render()
    } else {
      contributions.delete(job.key)
      render()
      if (result.kind === 'unavailable') output.appendLine(result.message)
    }
  }

  async function run(job) {
    active++
    try {
      if (projects) job.snapshot = await projects.snapshot(job.document, job.controller.signal)
      if (!current(job)) return
      const result = job.snapshot
        ? await bundleCheck(job.executable, job.snapshot, { signal: job.controller.signal })
        : await check(job.executable, job.document.getText(), { signal: job.controller.signal })
      publish(job, result)
    } catch (error) {
      if (current(job)) output.appendLine(`Compiler integration failure: ${error.message}`)
    } finally {
      active--
      if (pending.get(job.key) === job) pending.delete(job.key)
      drain()
    }
  }

  function drain() {
    while (!disposed && active < 2 && queue.length) {
      const job = queue.shift()
      if (current(job)) void run(job)
    }
  }

  function schedule(document) {
    remove(document)
    if (disposed || document.languageId !== 'lobo' || document.isClosed || !vscode.workspace.isTrusted) return
    const config = vscode.workspace.getConfiguration('lobo', document.uri)
    if (!config.get('diagnostics.enabled', true)) return
    const configured = config.get('compilerPath', 'lobo')
    if (typeof configured !== 'string' || !configured.trim()) { output.appendLine('Set lobo.compilerPath to the compiler executable'); return }
    const folder = vscode.workspace.getWorkspaceFolder(document.uri)
    if (configured.includes('${workspaceFolder}') && !folder) { output.appendLine('Compiler path requires an open workspace folder'); return }
    const executable = configured.replaceAll('${workspaceFolder}', folder?.uri.fsPath ?? '')
    const job = { key: document.uri.toString(), document, version: document.version,
      executable, controller: new AbortController() }
    pending.set(job.key, job)
    job.timer = scheduleTimer(() => { if (current(job)) { queue.push(job); drain() } }, 250)
  }

  const refresh = () => { for (const document of vscode.workspace.textDocuments) schedule(document) }
  const registrations = [
    vscode.workspace.onDidCloseTextDocument(remove),
    ...(projects ? [projects.onDidChange(refresh)] : [
      vscode.workspace.onDidOpenTextDocument(schedule),
      vscode.workspace.onDidChangeTextDocument(event => schedule(event.document)),
      vscode.workspace.onDidGrantWorkspaceTrust(refresh),
      vscode.workspace.onDidChangeConfiguration(event => { if (event.affectsConfiguration('lobo')) refresh() })
    ])
  ]
  refresh()
  return {
    dispose() {
      disposed = true
      for (const job of [...pending.values()]) remove(job.document)
      queue.length = 0
      for (const registration of registrations) registration.dispose()
      collection.dispose()
      output.dispose()
    }
  }
}

module.exports = { registerDiagnostics }
