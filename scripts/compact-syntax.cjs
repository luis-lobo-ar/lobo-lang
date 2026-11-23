#!/usr/bin/env node
'use strict'

// Authoring projection only. The unchanged Rust compiler owns language semantics.
const fs = require('node:fs')
const path = require('node:path')
const { project, projectionDiagnostic } = require('../editors/vscode/src/compact.cjs')
const { parseManifest } = require('../editors/vscode/src/identities.cjs')

const LIMIT = 1_048_576
function readSource(name, budget) {
  const fd = fs.openSync(name, fs.constants.O_RDONLY | (fs.constants.O_NONBLOCK ?? 0))
  try {
    const stat = fs.fstatSync(fd)
    if (!stat.isFile() || stat.size > budget) throw new Error('Expected a regular source file within the 1 MiB bundle budget')
    const buffer = Buffer.alloc(budget + 1)
    let size = 0
    while (size < buffer.length) {
      const count = fs.readSync(fd, buffer, size, buffer.length - size, null)
      if (!count) break
      size += count
    }
    if (size > budget) throw new Error('Source bundle exceeds 1 MiB')
    return new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(buffer.subarray(0, size))
  } finally { fs.closeSync(fd) }
}

function main(args) {
  const names = []
  let directory
  for (let i = 0; i < args.length; i++) {
    const arg = args[i]
    if (arg === '--source' && args[i + 1]) names.push(args[++i])
    else if (arg === '--out-dir' && args[i + 1] && directory === undefined) directory = args[++i]
    else if (!arg.startsWith('-') && !names.length) names.push(arg)
    else throw new Error('Usage: node scripts/compact-syntax.cjs ROOT [--source FILE] [--out-dir NEW_DIRECTORY]')
  }
  if (!names.length || names.length > 16 || new Set(names).size !== names.length) throw new Error('Supply 1..16 distinct explicit sources')
  let size = 0
  const sources = names.map(name => {
    if (!name.length || Buffer.byteLength(name) > 1024 || name.includes('\0')) throw new Error('Invalid source label')
    const text = readSource(name, LIMIT - size)
    size += Buffer.byteLength(text)
    let identities
    try {
      const metadata = readSource(`${name}.ids.json`, LIMIT - size)
      size += Buffer.byteLength(metadata)
      identities = parseManifest(metadata)
    } catch (error) { if (error.code !== 'ENOENT') throw error }
    return { name, text, ...(identities === undefined ? {} : { identities }) }
  })
  let count = 0
  const projected = sources.map(file => {
    let projection
    try { projection = project(file.text, file.identities) } catch (error) {
      throw Object.assign(new Error(error.message), { diagnostic: projectionDiagnostic(error, file.text, file.name) })
    }
    // Existing source admission still runs in the Rust compiler; count projected bytes here as well.
    count += Buffer.byteLength(projection.text)
    if (count > LIMIT) throw new Error('Lowered bundle exceeds 1 MiB')
    return { name: file.name, text: projection.text }
  })
  const bundle = { format: 'lobo-source/0.1', root: names[0], sources: projected }
  if (directory === undefined) return bundle
  // Complete projection before any write. A fresh directory prevents accidental overwrites.
  fs.mkdirSync(directory)
  const outputs = projected.map((file, index) => {
    const name = index === 0 ? 'app.lobo' : `source-${index}.lobo`
    fs.writeFileSync(path.join(directory, name), `${file.text}\n`, { flag: 'wx' })
    return path.join(directory, name)
  })
  fs.writeFileSync(path.join(directory, 'bundle.json'), `${JSON.stringify(bundle, null, 2)}\n`, { flag: 'wx' })
  fs.writeFileSync(path.join(directory, 'inputs.json'), `${JSON.stringify({ format: 'lobo-compact/0.1', sources }, null, 2)}\n`, { flag: 'wx' })
  return { status: 'projected', root: outputs[0], sources: outputs.slice(1), bundle: path.join(directory, 'bundle.json') }
}

try { process.stdout.write(`${JSON.stringify(main(process.argv.slice(2)), null, 2)}\n`) }
catch (error) {
  process.stderr.write(`${JSON.stringify({ status: 'error', message: error.message, ...(error.diagnostic ? { diagnostic: error.diagnostic } : {}) })}\n`)
  process.exitCode = 1
}
