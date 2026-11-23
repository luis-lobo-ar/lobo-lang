'use strict'

const STABLE_ID = /^[A-Za-z0-9][A-Za-z0-9_.:\-]{0,127}$/
const NAME = /^[A-Za-z_][A-Za-z_0-9]*$/
const { validateBindings } = require('./identities.cjs')

class ProjectionError extends SyntaxError {
  constructor(message, token = { start: 0, end: 0 }) {
    super(message)
    this.span = { start: token.start, end: token.end }
  }
}

function fail(message, token) { throw new ProjectionError(message, token) }

function lex(source) {
  if (Buffer.byteLength(source, 'utf8') > 1_048_576) fail('Compact source exceeds 1 MiB')
  const pattern = /[ \t\r\n]+|\/\/[^\r\n]*|"(?:[^"\\\r\n]|\\[^\r\n])*"|[A-Za-z_][A-Za-z_0-9]*|[0-9]+(?:\.[0-9]+)?(?:[eE][+\-]?[0-9]+)?|\.\.\.|::|->|=>|==|!=|<=|>=|&&|\|\||[@();,:{}.+*/%<>=!&\-]/gy
  const tokens = []
  while (pattern.lastIndex < source.length) {
    const start = pattern.lastIndex
    const match = pattern.exec(source)
    if (!match) fail('Unsupported or incomplete compact token', { start, end: start + 1 })
    const text = match[0]
    if (/^[ \t\r\n]|^\/\//.test(text)) continue
    if (text.startsWith('"')) {
      let value
      try { value = JSON.parse(text) } catch { fail('Invalid quoted string', { start, end: pattern.lastIndex }) }
      for (let i = 0; i < value.length; i++) {
        const unit = value.charCodeAt(i)
        if (unit >= 0xd800 && unit <= 0xdbff) {
          const next = value.charCodeAt(++i)
          if (!(next >= 0xdc00 && next <= 0xdfff)) fail('Unpaired surrogate', { start, end: pattern.lastIndex })
        } else if (unit >= 0xdc00 && unit <= 0xdfff) fail('Unpaired surrogate', { start, end: pattern.lastIndex })
      }
    }
    tokens.push({ text, start, end: pattern.lastIndex })
    if (tokens.length > 8192) fail('Compact source exceeds 8192 tokens', tokens.at(-1))
  }
  return tokens
}

function groups(tokens) {
  let position = 0
  function read(close, depth) {
    if (depth > 32) fail('Compact nesting exceeds 32 levels', tokens[position])
    const result = []
    while (position < tokens.length) {
      const token = tokens[position++]
      if (token.text === close) return { children: result, close: token }
      if ([')', '}'].includes(token.text)) fail('Unexpected closing delimiter', token)
      if (['(', '{'].includes(token.text)) {
        const group = read(token.text === '(' ? ')' : '}', depth + 1)
        result.push({ ...token, ...group })
      } else result.push(token)
    }
    if (close) fail(`Expected ${close}`, tokens.at(-1))
    return result
  }
  return read(undefined, 0)
}

function split(nodes, delimiter) {
  const result = [[]]
  let angleDepth = 0
  let previous
  for (const node of nodes) {
    if (node.text === '<' && (previous === '::' || angleDepth > 0)) angleDepth++
    if (node.text === '>' && angleDepth > 0) angleDepth--
    if (node.text === delimiter && angleDepth === 0) result.push([])
    else result.at(-1).push(node)
    previous = node.text
  }
  return result
}

function synthetic(text, anchor) { return { text, start: anchor.start, end: anchor.end, generated: true } }
function flatten(nodes) { return nodes.flatMap(node => node.children ? [node, ...flatten(node.children), node.close] : [node]) }

function metadata(nodes, supplied, end) {
  const marker = nodes.findIndex(node => node.text === 'identities')
  let external
  if (supplied !== undefined) {
    validateBindings(supplied)
    external = new Map(Object.entries(supplied).map(([key, id]) => [key, { text: JSON.stringify(id), start:end, end, generated:true }]))
  }
  if (marker < 0 && external) return { body:nodes, ids:external }
  if (marker < 0 || marker !== nodes.length - 2 || nodes.at(-1).text !== '{') fail('End compact source with one identities table', nodes.at(-1))
  const ids = new Map()
  for (const entry of split(nodes.at(-1).children, ',')) {
    if (!entry.length) fail('Empty identity entry', nodes.at(-1))
    const colon = entry.findIndex(node => node.text === ':')
    const keyTokens = entry.slice(0, colon)
    if (colon < 1 || entry.length !== colon + 2 ||
        keyTokens.some((token, i) => i % 2 === 0 ? !NAME.test(token.text) : token.text !== '.')) fail('Invalid identity entry', entry[0])
    const key = keyTokens.map(token => token.text).join('')
    const value = entry.at(-1)
    if (!value.text.startsWith('"') || !STABLE_ID.test(JSON.parse(value.text))) fail('Invalid stable identity', value)
    if (ids.has(key)) fail('Repeated identity key', entry[0])
    ids.set(key, value)
  }
  if (external && (ids.size !== external.size || [...ids].some(([key, value]) => value.text !== external.get(key)?.text && JSON.parse(value.text) !== supplied[key]))) fail('Conflicting inline and external identities')
  return { body: nodes.slice(0, marker), ids }
}

function project(source, identities) {
  // The module header selects source spelling. Comments and the historical marker have no semantics.
  const lexed = lex(source)
  if (lexed[0]?.text !== 'module' || !NAME.test(lexed[1]?.text ?? '') || lexed[2]?.text !== ';') {
    if (identities !== undefined) fail('External identities require modern source')
    return { text: source }
  }
  for (const token of lexed) if (token.text.startsWith('__lobo_')) fail('Compiler-generated names are reserved',token)
  const { body, ids } = metadata(groups(lexed), identities, source.length)
  const used = new Set()
  const namespaces = new Map()
  const imported = new Map()
  const declarations = new Set()
  const identity = (key, anchor) => {
    const value = ids.get(key)
    if (!value) fail(`Missing persistent identity for ${key}`, anchor)
    used.add(key)
    return ['@', 'id', '('].map(text => synthetic(text, anchor)).concat(value, synthetic(')', anchor))
  }
  const headers = []
  const content = []
  if (body[0]?.text !== 'module' || !NAME.test(body[1]?.text ?? '') || body[2]?.text !== ';') fail('Expected module name;', body[0])
  headers.push(body[0], body[1], ...identity('module', body[1]), body[2])
  for (let i = 3; i < body.length; i++) {
    const node = body[i]
    if (node.text === 'use') {
      if (declarations.size) fail('Use groups must precede declarations', node)
      const namespace = body[++i]
      const members = body[++i]
      if (!namespace || !NAME.test(namespace.text) || members?.text !== '{' || body[++i]?.text !== ';' || namespaces.has(namespace.text)) fail('Expected unique use namespace { members };', node)
      const selected = new Set()
      for (const entry of split(members.children, ',')) {
        const member = entry[0]
        if (entry.length !== 1 || !NAME.test(member.text) || imported.has(member.text)) fail('Imports require unique selected member names', member ?? node)
        selected.add(member.text)
        imported.set(member.text, { namespace: namespace.text, id: ids.get(`${namespace.text}.${member.text}`) })
        headers.push(synthetic('import', node), member, ...identity(`${namespace.text}.${member.text}`, member), synthetic(';', member))
      }
      namespaces.set(namespace.text, selected)
    } else if (['fn', 'record', 'variant'].includes(node.text)) {
      const name = body[++i]
      if (!name || !NAME.test(name.text) || declarations.has(name.text)) fail('Expected unique declaration name', name ?? node)
      declarations.add(name.text)
      content.push(node.text === 'fn' ? { ...node, text: 'function' } : node, name)
      if (body[i + 1]?.text === '<') {
        let depth = 0
        do {
          const parameter = body[++i]
          if (!parameter) fail('Unterminated type parameters', name)
          content.push(parameter)
          if (parameter.text === '<') depth++
          if (parameter.text === '>') depth--
        } while (depth > 0)
      }
      content.push(...identity(name.text, name))
    } else if (['function', 'import', '@'].includes(node.text)) fail('Use compact declarations and the identities table', node)
    else content.push(node)
  }
  for (const key of ids.keys()) if (!used.has(key)) fail(`Unbound identity key ${key}; preserve its ID when renaming the binding`, ids.get(key))
  for (const name of namespaces.keys()) if (declarations.has(name) || imported.has(name)) fail('Namespace conflicts with a declaration or import', body[1])

  const generated = new Map()
  function dependency(name, stableId, anchor) {
    if (name === 'Json') {
      const entry=imported.get(name)
      if (!entry || JSON.parse(entry.id.text)!==stableId) fail(`json objects require explicit ${name} import with identity ${stableId}`,anchor)
      return synthetic(name,anchor)
    }
    const selected=[...imported].find(([,entry])=>JSON.parse(entry.id.text)===stableId)
    if (selected) return synthetic(selected[0],anchor)
    const alias=`__lobo_json_${name}`
    if (!generated.has(stableId)) {
      generated.set(stableId,alias)
      for (const text of ['import',alias,'@','id','(',JSON.stringify(stableId),')',';']) headers.push(synthetic(text,anchor))
    }
    return synthetic(alias,anchor)
  }
  function rewrite(nodes, depth = 0) {
    if (depth > 32) fail('Compact rewrite nesting exceeds 32 levels', nodes[0])
    const out = []
    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i]
      if (node.text === 'json' && nodes[i + 1]?.text === '{') {
        const fields = nodes[++i]
        const nominal = dependency('Json', 'lobo.json.value', node)
        const object = dependency('object', 'lobo.json.object', node)
        let value = [object, synthetic('(', node), synthetic(')', node)]
        const seen = new Set()
        for (const entry of fields.children.length ? split(fields.children, ',') : []) {
          if (!entry[0]?.text.startsWith('"') || entry[1]?.text !== ':' || entry.length < 3) fail('JSON fields require a quoted key and a Text expression', entry[0] ?? node)
          const key = JSON.parse(entry[0].text)
          if (seen.has(key)) fail('Duplicate JSON key', entry[0])
          seen.add(key)
          const field = dependency('field', 'lobo.json.field', node)
          value = [field, synthetic('(', node), ...value, synthetic(',', node), entry[0], synthetic(',', node), nominal,
            synthetic('::', node), synthetic('String', node), synthetic('(', node), ...rewrite(entry.slice(2), depth + 1),
            synthetic(')', node), synthetic(')', node)]
        }
        out.push(nominal, synthetic('::', node), synthetic('Object', node), synthetic('(', node), ...value, synthetic(')', node))
      } else if (node.text === 'match') {
        const subject = nodes[++i]
        if (!subject) fail('Missing match subject', node)
        const subjectNodes = [subject]
        if (!subject.children && NAME.test(subject.text)) {
          if (namespaces.has(subject.text) && nodes[i+1]?.text === '.') {
            if (!NAME.test(nodes[i+2]?.text ?? '')) fail('Missing selected match subject member', subject)
            subjectNodes.push(nodes[++i], nodes[++i])
          }
          if (subject.text === 'Native' && nodes[i+1]?.text === '::') {
            if (!NAME.test(nodes[i+2]?.text ?? '')) fail('Missing native match operation',subject)
            subjectNodes.push(nodes[++i],nodes[++i])
          }
          if (nodes[i+1]?.text === '::' && nodes[i+2]?.text === '<') {
            subjectNodes.push(nodes[++i])
            let angles = 0
            do {
              const part = nodes[++i]
              if (!part) fail('Unterminated match subject type arguments', subject)
              subjectNodes.push(part)
              if (part.text === '<') angles++
              if (part.text === '>') angles--
            } while (angles > 0)
          }
          if (nodes[i+1]?.text === '(') subjectNodes.push(nodes[++i])
        }
        if (subject.text !== '(' && !NAME.test(subject.text)) fail('Match subject must be a name, call or parenthesized expression', subject)
        const arms = nodes[++i]
        if (arms?.text !== '{') fail('Missing match arms', arms ?? subject)
        out.push(node, synthetic('(', subject), ...rewrite(subjectNodes, depth + 1), synthetic(')', subject), arms)
        for (const [index, arm] of split(arms.children, ',').entries()) {
          const arrow = arm.findIndex(token => token.text === '=>')
          if (arrow < 1 || arrow > 2 || !NAME.test(arm[0].text) || (arrow === 2 && arm[1].text !== '(') || arrow === arm.length - 1) fail('Expected Case(binding) => expression', arm[0] ?? arms)
          if (index) out.push(synthetic(',', arm[0]))
          const expression = arm.slice(arrow + 1)
          const body = expression.length === 1 && expression[0].text === '{' ? expression[0].children : expression
          out.push(...flatten(arm.slice(0, arrow)), synthetic('{', arm[arrow]), ...rewrite(body, depth + 1), synthetic('}', arm[arrow]))
        }
        out.push(arms.close)
      } else if (namespaces.has(node.text)) {
        const member = nodes[i + 2]
        if (nodes[i + 1]?.text !== '.' || !namespaces.get(node.text).has(member?.text)) fail('Namespace needs an explicitly selected member', node)
        out.push(member)
        i += 2
      } else if (node.text === '=>') fail('Arrow is only valid in match arms', node)
      else if (node.children) out.push(node, ...rewrite(node.children, depth + 1), node.close)
      else out.push(node)
    }
    return out
  }
  const lowered=rewrite(content)
  const tokens = headers.concat(lowered)
  if (tokens.length > 8192) fail('Lowered compact source exceeds 8192 tokens', body[0])
  const exportAliases=new Map()
  const names=new Set(lexed.map(token=>token.text))
  for (const alias of generated.values()) {
    const prefix=alias.replace('__lobo_','lobo_export_')
    let candidate=prefix
    for (let n=0;names.has(candidate);n++) candidate=`${prefix}_${n}`
    names.add(candidate)
    exportAliases.set(alias,candidate)
  }
  const text = tokens.map(token => token.generated ? exportAliases.get(token.text) ?? token.text : token.text).join(' ')
  if (Buffer.byteLength(text, 'utf8') > 1_048_576) fail('Lowered compact source exceeds 1 MiB', body[0])
  return { text, tokens: tokens.map(token => ({ ...token, end: token.generated ? token.start : token.end })) }
}

function point(source, offset) {
  const before = source.slice(0, offset)
  const lines = before.split(/\r\n|\r|\n/)
  return { byte: Buffer.byteLength(before), line: lines.length - 1, character: lines.at(-1).length }
}

function projectionDiagnostic(error, source, name) {
  const start = Math.min(error.span?.start ?? 0, source.length)
  const end = Math.min(error.span?.end ?? start, source.length)
  return { code: 'compile.compact.invalid', phase: 'project', message: error.message,
    ...(name === undefined ? {} : { source: name }), range: { start: point(source, start), end: point(source, end) } }
}

module.exports = { project, projectionDiagnostic }
