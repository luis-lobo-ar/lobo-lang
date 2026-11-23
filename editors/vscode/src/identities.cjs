'use strict'

const NAME = /^[A-Za-z_][A-Za-z_0-9]*(?:\.[A-Za-z_][A-Za-z_0-9]*)?$/
const ID = /^[A-Za-z0-9][A-Za-z0-9_.:\-]{0,127}$/

function validateBindings(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value) || Object.keys(value).length > 2048 ||
      Object.entries(value).some(([key, id]) => !NAME.test(key) || typeof id !== 'string' || !ID.test(id))) {
    throw new Error('Invalid identity bindings')
  }
  return value
}

// The sidecar schema contains two objects and string values. Reject duplicate keys before
// JSON.parse can discard them, including duplicate keys spelled with JSON escapes.
function parseManifest(text) {
  if (Buffer.byteLength(text) > 1_048_576) throw new Error('Identity metadata exceeds its byte budget')
  const pattern = /[ \t\r\n]+|"(?:[^"\\\x00-\x1f]|\\(?:["\\/bfnrt]|u[0-9a-fA-F]{4}))*"|[{}:,]/gy
  const tokens = []
  while (pattern.lastIndex < text.length) {
    const match = pattern.exec(text)
    if (!match) throw new Error('Invalid identity metadata JSON')
    if (!/^\s/.test(match[0])) tokens.push(match[0])
    if (tokens.length > 8300) throw new Error('Identity metadata exceeds its token budget')
  }
  let position = 0
  const take = expected => {
    if (tokens[position++] !== expected) throw new Error('Invalid identity metadata JSON')
  }
  const string = () => {
    const token = tokens[position++]
    if (!token?.startsWith('"')) throw new Error('Identity metadata requires strings')
    return JSON.parse(token)
  }
  function object(depth) {
    if (depth > 1) throw new Error('Invalid identity metadata nesting')
    take('{')
    const result = Object.create(null)
    if (tokens[position] !== '}') {
      while (true) {
        const key = string()
        if (Object.hasOwn(result, key)) throw new Error('Duplicate identity metadata key')
        take(':')
        result[key] = tokens[position] === '{' ? object(depth + 1) : string()
        if (tokens[position] !== ',') break
        take(',')
      }
    }
    take('}')
    return result
  }
  const value = object(0)
  if (position !== tokens.length || value.format !== 'lobo-identities/0.1' ||
      Object.keys(value).length !== 2 || !Object.hasOwn(value, 'identities')) throw new Error('Unsupported identity metadata schema')
  return { ...validateBindings(value.identities) }
}

module.exports = { parseManifest, validateBindings }
