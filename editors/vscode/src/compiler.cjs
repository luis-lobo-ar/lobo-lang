'use strict'

const { spawn } = require('node:child_process')

function unavailable(message) { return { kind: 'unavailable', message } }

function validRange(range, source) {
  if (!range?.start || !range.end) return false
  const bytes = Buffer.from(source, 'utf8')
  for (const point of [range.start, range.end]) {
    if (![point.byte, point.line, point.character].every(value => Number.isSafeInteger(value) && value >= 0)) return false
    if (point.byte > bytes.length || (point.byte < bytes.length && (bytes[point.byte] & 0xc0) === 0x80)) return false
    let line = 0
    let character = 0
    let cr = false
    for (const scalar of bytes.subarray(0, point.byte).toString('utf8')) {
      if (scalar === '\r' || (scalar === '\n' && !cr)) { line++; character = 0 }
      else if (scalar !== '\n') character += scalar.length
      cr = scalar === '\r'
    }
    if (line !== point.line || character !== point.character) return false
  }
  return range.start.byte <= range.end.byte
}

function decodeResult(stdout, stderr, code, source) {
  let result
  try { result = JSON.parse(stdout) } catch {
    return unavailable(`Compiler did not return source-check JSON${stderr ? `: ${stderr.slice(0, 1024)}` : ''}`)
  }
  if (!result || typeof result !== 'object' || Array.isArray(result)) return unavailable('Invalid compiler response envelope')
  if (code === 0 && result.status === 'ok' &&
      (typeof source === 'string' || ['core/0.9-draft', 'core/0.10-draft', 'core/0.11-draft', 'core/0.12-draft', 'core/1.0', 'core/1.1', 'core/1.2', 'core/1.3', 'core/1.4', 'core/1.5', 'core/1.6', 'core/1.7', 'core/1.8', 'core/1.9', 'core/1.10', 'core/1.11'].includes(result.language)) &&
      ['core/0.2-draft', 'core/0.3-draft', 'core/0.4-draft', 'core/0.5-draft', 'core/0.6-draft', 'core/0.7-draft', 'core/0.8-draft', 'core/0.9-draft', 'core/0.10-draft', 'core/0.11-draft', 'core/0.12-draft', 'core/1.0', 'core/1.1', 'core/1.2', 'core/1.3', 'core/1.4', 'core/1.5', 'core/1.6', 'core/1.7', 'core/1.8', 'core/1.9', 'core/1.10', 'core/1.11'].includes(result.language) && Array.isArray(result.entries)) {
    return { kind: 'ok' }
  }
  const diagnostic = result.diagnostic
  const diagnosticSource = typeof source === 'string'
    ? (diagnostic?.source === undefined ? source : undefined)
    : source.sources.find(file => file.name === diagnostic?.source)?.text
  if (code === 1 && result.status === 'error' && result.kind === 'resource_exhausted' &&
      typeof diagnostic?.code === 'string' && diagnostic.code.startsWith('resource.') && typeof diagnostic.message === 'string') {
    return unavailable(`Compiler resource limit: ${diagnostic.code}: ${diagnostic.message}`)
  }
  if (code === 1 && result.status === 'error' && result.kind === 'compile_error' &&
      typeof diagnostic?.code === 'string' && diagnostic.code.startsWith('compile.') &&
      typeof diagnostic.phase === 'string' && typeof diagnostic.message === 'string' &&
      typeof diagnosticSource === 'string' && validRange(diagnostic.range, diagnosticSource)) {
    return { kind: 'diagnostic', diagnostic }
  }
  return unavailable(`Unsupported/unlocated compiler outcome; use Lobo ${typeof source === 'string' ? '0.2.1+' : '0.9+'}: ${diagnostic?.code ?? 'invalid response'}`)
}

async function checkSource(executable, source, options = {}) {
  if (options.signal?.aborted) return { kind: 'cancelled' }
  if (Buffer.byteLength(source, 'utf8') > 1_048_576) return unavailable('Source exceeds the compiler 1 MiB budget')
  return checkInput(executable, source, source, '--stdin', options)
}

async function checkBundle(executable, bundle, options = {}) {
  if (options.signal?.aborted) return { kind: 'cancelled' }
  if (!Array.isArray(bundle?.sources) || !bundle.sources.length || bundle.sources.length > 16 ||
      bundle.sources.some(file => typeof file?.name !== 'string' || typeof file?.text !== 'string') ||
      new Set(bundle.sources.map(file => file.name)).size !== bundle.sources.length ||
      !bundle.sources.some(file => file.name === bundle.root)) return unavailable('Invalid explicit source bundle')
  if (bundle.sources.reduce((sum, file) => sum + Buffer.byteLength(file.text, 'utf8') +
      (file.identities === undefined ? 0 : Buffer.byteLength(JSON.stringify(file.identities))), 0) > 1_048_576) {
    return unavailable('Source bundle exceeds the compiler 1 MiB budget')
  }
  const input = JSON.stringify({ format: bundle.sources.some(file => file.identities !== undefined) ? 'lobo-source/0.2' : 'lobo-source/0.1', root: bundle.root, sources: bundle.sources })
  if (Buffer.byteLength(input, 'utf8') > 4_194_304) return unavailable('Source bundle envelope exceeds its 4 MiB budget')
  return checkInput(executable, input, bundle, '--bundle-stdin', options)
}

function checkInput(executable, input, source, mode, options) {
  return new Promise(resolve => {
    let settled = false
    let child
    let timer
    let killTimer
    let closed = false
    const finish = result => {
      if (settled) return
      settled = true
      clearTimeout(timer)
      options.signal?.removeEventListener('abort', cancel)
      resolve(result)
    }
    const stop = result => {
      if (settled) return
      finish(result)
      if (!child || closed) return
      killTimer = setTimeout(() => { if (!closed) child.kill('SIGKILL') }, options.killGraceMs ?? 250)
      killTimer.unref()
      child.kill('SIGTERM')
    }
    const cancel = () => stop({ kind: 'cancelled' })
    try {
      child = (options.spawnProcess ?? spawn)(executable, ['check', mode], {
        shell: false, windowsHide: true, stdio: ['pipe', 'pipe', 'pipe']
      })
    } catch (error) { finish(unavailable(`Cannot start compiler: ${error.message}`)); return }
    const stdout = []
    const stderr = []
    let outputBytes = 0
    for (const [stream, output] of [[child.stdout, stdout], [child.stderr, stderr]]) {
      stream.on('data', chunk => {
        if (settled) return
        const bytes = Buffer.from(chunk)
        outputBytes += bytes.length
        if (outputBytes > (options.maxOutputBytes ?? 2_097_152)) {
          stop(unavailable('Compiler output exceeded its 2 MiB budget'))
          return
        }
        output.push(bytes)
      })
    }
    child.on('error', error => finish(unavailable(`Cannot start compiler: ${error.message}`)))
    child.stdin.on('error', error => {
      if (error.code !== 'EPIPE') stop(unavailable(`Cannot send source to compiler: ${error.message}`))
    })
    child.on('close', code => {
      closed = true
      clearTimeout(killTimer)
      if (settled) return
      finish(decodeResult(Buffer.concat(stdout).toString('utf8'), Buffer.concat(stderr).toString('utf8'), code, source))
    })
    timer = setTimeout(() => stop(unavailable('Compiler check timed out after its time budget')), options.timeoutMs ?? 5000)
    options.signal?.addEventListener('abort', cancel, { once: true })
    if (options.signal?.aborted) { cancel(); return }
    child.stdin.end(input, 'utf8')
  })
}

module.exports = { checkSource, checkBundle }
