'use strict'

const { definitionAt, definitionInBundle } = require('./navigation.cjs')
const { registerDiagnostics } = require('./diagnostics.cjs')
const { createSourceProjects } = require('./project.cjs')

function positionAt(text, offset) {
  const lines = text.slice(0, offset).split(/\r\n|\r|\n/)
  return { line: lines.length - 1, character: lines.at(-1).length }
}

function createDefinitionProvider(vscode, projects) {
  async function fromProject(document, offset, token) {
    try {
      const controller = new AbortController()
      const cancellation = token?.onCancellationRequested?.(() => controller.abort())
      try {
        const snapshot = await projects.snapshot(document, controller.signal)
        if (!snapshot?.isCurrent() || token?.isCancellationRequested) return undefined
        const definition = definitionInBundle(snapshot.sources, snapshot.root, offset)
        if (!definition || !snapshot.isCurrent() || token?.isCancellationRequested) return undefined
        const target = snapshot.documents.get(definition.source)
        if (!target) return undefined
        const toRange = (text, span) => new vscode.Range(positionAt(text, span.start), positionAt(text, span.end))
        const range = toRange(target.text, definition.target)
        return [{ originSelectionRange: toRange(document.getText(), definition.origin), targetUri: target.uri,
          targetRange: range, targetSelectionRange: range }]
      } finally { cancellation?.dispose() }
    } catch { return undefined }
  }
  return {
    provideDefinition(document, position, token) {
      if (token?.isCancellationRequested) return undefined
      try {
        if (projects?.describe(document)) return fromProject(document, document.offsetAt(position), token)
      } catch { return undefined }
      const definition = definitionAt(document.getText(), document.offsetAt(position))
      if (!definition || token?.isCancellationRequested) return undefined
      const toRange = span => new vscode.Range(document.positionAt(span.start), document.positionAt(span.end))
      const target = toRange(definition.target)
      return [{
        originSelectionRange: toRange(definition.origin),
        targetUri: document.uri,
        targetRange: target,
        targetSelectionRange: target
      }]
    }
  }
}

function activate(context) {
  const vscode = require('vscode')
  const projects = createSourceProjects(vscode)
  context.subscriptions.push(projects)
  context.subscriptions.push(vscode.languages.registerDefinitionProvider(
    { language: 'lobo' }, createDefinitionProvider(vscode, projects)
  ))
  context.subscriptions.push(registerDiagnostics(vscode, { projects }))
}

module.exports = { activate, createDefinitionProvider }
