'use strict'

const fs = require('node:fs')
const path = require('node:path')
const os = require('node:os')
const crypto = require('node:crypto')
const { spawnSync } = require('node:child_process')
const ROOT = path.resolve(__dirname, '..')
const MANIFEST = path.join(__dirname, 'source-migration.json')
const BEFORE = path.join(ROOT, 'benchmarks/private-evidence/source-migration-20261114/before')
const sha = text => crypto.createHash('sha256').update(text).digest('hex')

function maintainedPath(file) {
  return typeof file === 'string' && /^(examples|libraries)\/.+\.lobo$/.test(file) &&
    !file.split('/').some(part => !part || part === '.' || part === '..') && !file.includes('\\') &&
    !file.startsWith('examples/v1/') && !['examples/mvc.lobo','examples/mvc.compact.lobo','examples/transfer.lobo'].includes(file)
}

function tokens(source) {
  const pattern = /\s+|\/\/[^\r\n]*|"(?:[^"\\\r\n]|\\[^\r\n])*"|[A-Za-z_][A-Za-z_0-9]*|[0-9]+(?:\.[0-9]+)?(?:[eE][+\-]?[0-9]+)?|::|->|=>|==|!=|<=|>=|&&|\|\||[@();,:{}.+*/%<>=!&\-]/gy
  const result = []
  while (pattern.lastIndex < source.length) {
    const start = pattern.lastIndex
    const match = pattern.exec(source)
    if (!match) throw new SyntaxError(`Unsupported token at ${start}`)
    if (/^\s|^\/\//.test(match[0])) continue
    result.push({ text: match[0], start, end: pattern.lastIndex })
  }
  return result
}

function parse(source) {
  const lexed = tokens(source)
  if (lexed[0]?.text !== 'module' || !/^[A-Za-z_]\w*$/.test(lexed[1]?.text ?? '')) throw new SyntaxError('Expected a Core module')
  const pairs = new Map()
  const stack = []
  for (const [index, token] of lexed.entries()) {
    if (['(', '{'].includes(token.text)) stack.push(index)
    if ([')', '}'].includes(token.text)) {
      const open = stack.pop()
      if (open === undefined || (lexed[open].text === '(' ? ')' : '}') !== token.text) throw new SyntaxError('Unbalanced source')
      pairs.set(open, index)
    }
  }
  if (stack.length) throw new SyntaxError('Unclosed source')
  return { lexed, pairs }
}

function migrate(source) {
  const { lexed: t, pairs } = parse(source)
  if (t[2]?.text === ';') return source
  const ids = []
  const imports = []
  const edits = []
  const words = new Set(t.map(token => token.text))
  let group = 'deps'
  for (let suffix = 2; words.has(group); suffix++) group = `deps_${suffix}`
  let moduleEnd
  let depth = 0
  for (let i = 0; i < t.length; i++) {
    const token = t[i]
    if (depth === 0 && ['module', 'function', 'record', 'variant', 'import'].includes(token.text)) {
      const name = t[i + 1]
      let annotation = i + 2
      if (t[annotation]?.text === '<') {
        while (annotation < t.length && t[annotation].text !== '>') annotation++
        annotation++
      }
      if (t[annotation]?.text !== '@' || t[annotation + 1]?.text !== 'id' || t[annotation + 2]?.text !== '(' ||
          pairs.get(annotation + 2) !== annotation + 4 || !t[annotation + 3]?.text.startsWith('"')) throw new SyntaxError('Expected stable declaration identity')
      const identity = JSON.parse(t[annotation + 3].text)
      const close = t[annotation + 4]
      if (token.text === 'import') {
        if (t[annotation + 5]?.text !== ';') throw new SyntaxError('Expected import semicolon')
        imports.push(name.text)
        ids.push([`${group}.${name.text}`, identity])
        for (let j = i; j <= annotation + 5; j++) edits.push({ start: t[j].start, end: t[j].end, text: '' })
      } else {
        ids.push([token.text === 'module' ? 'module' : name.text, identity])
        let start = t[annotation].start
        while (start > t[annotation - 1].end && /[ \t]/.test(source[start - 1])) start--
        edits.push({ start, end: close.end, text: '' })
        if (token.text === 'function') edits.push({ start: token.start, end: token.end, text: 'fn' })
        if (token.text === 'module') {
          if (t[annotation + 5]?.text !== ';') throw new SyntaxError('Expected module semicolon')
          moduleEnd = t[annotation + 5].end
        }
      }
    }
    if (token.text === '{') depth++
    if (token.text === '}') depth--
    if (token.text === 'match' && t[i+1]?.text === '(' && t[pairs.get(i+1)+1]?.text === '{') {
      const open = pairs.get(i + 1) + 1
      const end = pairs.get(open)
      let cursor = open + 1
      while (cursor < end) {
        if (!/^[A-Za-z_]\w*$/.test(t[cursor]?.text ?? '')) throw new SyntaxError('Invalid match case')
        cursor++
        if (t[cursor]?.text === '(') cursor = pairs.get(cursor) + 1
        if (t[cursor]?.text !== '{') throw new SyntaxError('Expected match arm block')
        edits.push({ start: t[cursor].start, end: t[cursor].start, text: '=> ' })
        cursor = pairs.get(cursor) + 1
        if (t[cursor]?.text === ',') cursor++
        else if (cursor !== end) throw new SyntaxError('Expected match separator')
      }
    }
  }
  if (moduleEnd === undefined) throw new SyntaxError('Missing module identity')
  if (imports.length) edits.push({ start: moduleEnd, end: moduleEnd, text: `\n\nuse ${group} { ${imports.join(', ')} };` })
  let output = source
  for (const edit of edits.sort((a, b) => b.start - a.start || b.end - a.end)) output = output.slice(0, edit.start) + edit.text + output.slice(edit.end)
  output = output.replace(/^[ \t]+$/gm, '').replace(/\n{3,}/g, '\n\n').trimEnd()
  return `${output}\n\nidentities {\n${ids.map(([key, value]) => `  ${key}: ${JSON.stringify(value)}`).join(',\n')}\n}\n`
}

function compileProgram(compiler, files, texts) {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'lobo-migration-'))
  try {
    const names = files.map((file, index) => {
      const name = path.join(temporary, `${index}.lobo`)
      fs.writeFileSync(name, texts.get(file))
      return name
    })
    const output = path.join(temporary, 'program.loboc')
    const args = ['compile', names[0], ...names.slice(1).flatMap(name => ['--source', name]), '--emit', 'checked', '-o', output]
    const result = spawnSync(compiler, args, { encoding: 'utf8', timeout: 15000, maxBuffer: 2_097_152 })
    if (result.status !== 0) throw new Error(`Compile ${files[0]}: ${result.stdout} ${result.stderr}`)
    const artifact = JSON.parse(fs.readFileSync(output, 'utf8'))
    return sha(JSON.stringify(artifact.program))
  } finally { fs.rmSync(temporary, { recursive: true }) }
}

function inventory(source) {
  const t = tokens(source)
  const declarations = []
  const imports = []
  for (let i = 0; i < t.length; i++) {
    if (!['module', 'function', 'record', 'variant', 'import'].includes(t[i].text)) continue
    let position = i + 2
    if (t[position]?.text === '<') { while (t[position]?.text !== '>' && position < t.length) position++; position++ }
    if (t[position]?.text !== '@') continue
    const id = JSON.parse(t[position + 3].text)
    if (t[i].text === 'import') imports.push(id)
    else declarations.push(id)
  }
  return { declarations, imports }
}

function capture(compiler) {
  if (fs.existsSync(MANIFEST)) throw new Error('The migration baseline already exists')
  const excluded = new Set(['examples/mvc.lobo', 'examples/mvc.compact.lobo', 'examples/transfer.lobo'])
  const files = ['examples', 'libraries'].flatMap(directory => fs.readdirSync(path.join(ROOT, directory), { recursive: true })
    .map(file => `${directory}/${file.split(path.sep).join('/')}`))
    .filter(file => file.endsWith('.lobo') && !file.startsWith('examples/v1/') && !excluded.has(file)).sort()
  const texts = new Map(files.map(file => [file, fs.readFileSync(path.join(ROOT, file), 'utf8')]))
  const metadata = new Map(files.map(file => [file, inventory(texts.get(file))]))
  const owners = new Map()
  for (const [file, entry] of metadata) for (const id of entry.declarations) {
    if (owners.has(id)) throw new Error(`Repeated declaration ${id}`)
    owners.set(id, file)
  }
  const bundles = files.map(root => {
    const closure = new Set([root])
    function visit(file) { for (const id of metadata.get(file).imports) {
      const owner = owners.get(id)
      if (!owner) throw new Error(`Missing explicit owner for ${id}`)
      if (!closure.has(owner)) { closure.add(owner); visit(owner) }
    } }
    visit(root)
    const sources = [root, ...[...closure].filter(file => file !== root).sort()]
    return { root, sources, program_sha256: compileProgram(compiler, sources, texts) }
  })
  const sessionPath = 'examples/testing/agent-session.jsonl'
  const sessionText = fs.readFileSync(path.join(ROOT, sessionPath), 'utf8')
  const session = sessionText.trimEnd().split('\n').map(line => JSON.parse(line))
  const embedded = session.flatMap((row, line) => (row.action.sources ?? []).map((source, index) => ({ line, index, program_sha256: compileProgram(compiler, [source.name], new Map([[source.name, source.text]])) })))
  const result = { format: 'lobo-source-migration/1', language: 'core/1.2', files: files.map(file => ({ path: file, before_sha256: sha(texts.get(file)), identities: metadata.get(file).declarations, import_ids: metadata.get(file).imports })), bundles, session: { path: sessionPath, before_sha256: sha(sessionText), embedded }, excluded: ['examples/v1/**', ...excluded, 'examples/agent-session.jsonl', 'benchmarks/**', 'research/**', 'spec/**', 'dist/**'] }
  for (const [file, text] of [...texts, [sessionPath, sessionText]]) {
    const target = path.join(BEFORE, file); fs.mkdirSync(path.dirname(target), { recursive: true }); fs.writeFileSync(target, text, { flag: 'wx' })
  }
  fs.writeFileSync(MANIFEST, `${JSON.stringify(result, null, 2)}\n`, { flag: 'wx' })
  return { status: 'captured', files: files.length, bundles: bundles.length, embedded: embedded.length }
}

function check(compiler, apply) {
  const manifest = JSON.parse(fs.readFileSync(MANIFEST, 'utf8'))
  if (manifest.files.some(file => !maintainedPath(file.path)) || manifest.session.path !== 'examples/testing/agent-session.jsonl') throw new Error('Migration manifest exceeds its maintained-source scope')
  for (const file of manifest.frozen_study_sources ?? []) {
    if (sha(fs.readFileSync(path.join(ROOT, file.path))) !== file.sha256) throw new Error(`Frozen research fixture changed: ${file.path}`)
  }
  const originals = new Map(manifest.files.map(file => [file.path, fs.readFileSync(path.join(ROOT, file.path), 'utf8')]))
  const texts = new Map([...originals].map(([file, source]) => [file, apply ? migrate(source) : source]))
  for (const [file, text] of texts) if (tokens(text)[2]?.text !== ';') throw new Error(`Expected modern syntax: ${file}`)
  for (const bundle of manifest.bundles) if (compileProgram(compiler, bundle.sources, texts) !== bundle.program_sha256) throw new Error(`Checked program changed: ${bundle.root}`)
  const sessionPath = path.join(ROOT, manifest.session.path)
  const originalSession = fs.readFileSync(sessionPath, 'utf8')
  const rows = originalSession.trimEnd().split('\n').map(line => JSON.parse(line))
  for (const entry of manifest.session.embedded) {
    const source = rows[entry.line].action.sources[entry.index]
    if (apply) source.text = migrate(source.text)
    if (tokens(source.text)[2]?.text !== ';' || compileProgram(compiler, [source.name], new Map([[source.name, source.text]])) !== entry.program_sha256) throw new Error('Embedded session source changed')
  }
  if (apply) {
    // All semantic gates pass before the first maintained source is replaced.
    for (const [file, before] of originals) if (fs.readFileSync(path.join(ROOT, file), 'utf8') !== before) throw new Error(`Source changed during migration: ${file}`)
    if (fs.readFileSync(sessionPath, 'utf8') !== originalSession) throw new Error('Session changed during migration')
    for (const [file, text] of texts) fs.writeFileSync(path.join(ROOT, file), text)
    fs.writeFileSync(sessionPath, `${rows.map(row => JSON.stringify(row)).join('\n')}\n`)
  }
  return { status: 'passed', files: texts.size, checked_bundles: manifest.bundles.length, embedded_sources: manifest.session.embedded.length }
}

module.exports = { migrate, maintainedPath }

if (require.main === module) {
  try {
    const args = process.argv.slice(2)
    const mode = args.shift()
    let compiler = path.join(ROOT, 'target/debug/lobo')
    if (args[0] === '--compiler' && args[1] && args.length === 2) compiler = path.resolve(args[1])
    else if (args.length) throw new Error('Unexpected migration arguments')
    const result = mode === '--capture' ? capture(compiler) : ['--verify', '--apply'].includes(mode) ? check(compiler, mode === '--apply') : (() => { throw new Error('Use --capture, --verify or --apply [--compiler PATH]') })()
    process.stdout.write(`${JSON.stringify(result)}\n`)
  } catch (error) { process.stderr.write(`${error.stack}\n`); process.exitCode = 1 }
}
