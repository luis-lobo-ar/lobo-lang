'use strict'

const { project } = require('./compact.cjs')

const RESERVED = new Set(['module', 'function', 'let', 'if', 'else', 'requires', 'ensures',
  'I64', 'F64', 'Bool', 'List', 'Bytes', 'Text', 'Fn', 'true', 'false', 'result'])
const OPERATORS = [['||'], ['&&'], ['==', '!='], ['<', '<=', '>', '>='], ['+', '-'], ['*', '/', '%']]

function tokenize(source, identities) {
  const projection = project(source, identities)
  if (projection.tokens) return [...projection.tokens, { text: '', start: source.length, end: source.length }]
  if (source.length > 1_048_576) throw new SyntaxError('Document budget exceeded')
  const pattern = /[ \t\r\n]+|\/\/[^\r\n]*|"(?:[^"\\\r\n]|\\[^\r\n])*"|[A-Za-z_][A-Za-z_0-9]*|[0-9]+(?:\.[0-9]+)?(?:[eE][+\-]?[0-9]+)?|\.\.\.|::|->|==|!=|<=|>=|&&|\|\||[@();,:{}.+*/%<>=!&\-]/gy
  const tokens = []
  while (pattern.lastIndex < source.length) {
    const start = pattern.lastIndex
    const match = pattern.exec(source)
    if (!match) throw new SyntaxError('Unsupported or incomplete token')
    const text = match[0]
    if (/^[ \t\r\n]|^\/\//.test(text)) continue
    tokens.push({ text, start, end: pattern.lastIndex })
    if (tokens.length > 8192) throw new SyntaxError('Token budget exceeded')
  }
  tokens.push({ text: '', start: source.length, end: source.length })
  return tokens
}

function range(token) {
  return { start: token.start, end: token.end }
}

function callbackName(actions = []) { return actions.length ? `Fn[${actions.join(';')}]` : 'Fn' }
function isCallback(name) { return name === 'Fn' || name?.startsWith('Fn[') }

function application(type) {
  if (type === undefined) return { name: undefined, arguments: [] }
  const begin = type.indexOf('<')
  if (begin < 0) return { name: type, arguments: [] }
  const arguments_ = []
  let depth = 0
  let start = begin + 1
  for (let index = start; index < type.length - 1; index++) {
    if (type[index] === '<') depth++
    if (type[index] === '>') depth--
    if (type[index] === ',' && depth === 0) { arguments_.push(type.slice(start,index)); start = index + 1 }
  }
  arguments_.push(type.slice(start,-1))
  return { name: type.slice(0,begin), arguments: arguments_ }
}

function mapType(type, atom) {
  if (type === undefined) return undefined
  const parsed = application(type)
  const name = atom(parsed.name)
  if (name === undefined) return undefined
  if (!parsed.arguments.length) return name
  const args = parsed.arguments.map(value => mapType(value,atom))
  return appliedType(name,args)
}

function appliedType(name,args) {
  if (name === undefined || args.includes(undefined)) return undefined
  if (!args.length) return name
  let bytes=name.length+2+args.length-1
  let nodes=1
  for (const type of args) {
    bytes+=type.length
    nodes++
    let depth=1
    for (const scalar of type) {
      if (scalar === '<') {depth++;nodes++}
      if (scalar === ',') nodes++
      if (scalar === '>') depth--
      if (depth>8 || nodes>8192 || bytes>1_048_576) return undefined
    }
  }
  return `${name}<${args.join(',')}>`
}

function instantiate(type, shape, args) {
  return mapType(type, atom => atom.startsWith('@') ? args[shape?.typeParameters?.indexOf(atom.slice(1))] : atom)
}

function guarded(resolve) {
  let active = false
  let cached
  return () => {
    if (cached !== undefined) return cached
    if (active) return undefined
    active = true
    try { cached=resolve(); return cached } finally { active = false }
  }
}

function constrain(pattern, actual, parameters, bindings) {
  if (actual === undefined) return true
  if (parameters.has(pattern)) {
    if (bindings.has(pattern)) return bindings.get(pattern) === actual
    bindings.set(pattern, actual)
    return true
  }
  const a = application(pattern)
  const b = application(actual)
  return a.name === b.name && a.arguments.length === b.arguments.length &&
    a.arguments.every((type, index) => constrain(type, b.arguments[index], parameters, bindings))
}

const NATIVE_RESULTS = new Map([
  ['CapabilityResult','Native::Capability'],['I64Result','I64'],['BytesResult','Bytes'],['BoolResult','Bool'],
  ['ListenerResult','Native::Listener'],['StreamResult','Native::Stream'],['SqliteCapabilityResult','Native::SqliteCapability'],['DatabaseResult','Native::Database'],['StatementResult','Native::Statement'],['TextResult','Text'],['F64Result','F64']
].map(([name,payload]) => [`Native::${name}`, {typeParameters:[],cases:new Map([['Ok',{payload}],['Err',{payload:'I64'}]])}]))
const NATIVE_TYPES = new Set(['Native::Capability','Native::Listener','Native::Stream','Native::SqliteCapability','Native::Database','Native::Statement',...NATIVE_RESULTS.keys()])
const NATIVE_SIGNATURES = {
  sqlite_capability:[0,'Native::SqliteCapability'],
  sqlite_restrict:[2,'Native::SqliteCapabilityResult'],
  sqlite_open:[1,'Native::DatabaseResult'],
  sqlite_close:[1,'Native::BoolResult'],
  sqlite_prepare:[2,'Native::StatementResult'],
  sqlite_bind_null:[2,'Native::BoolResult'],
  sqlite_bind_i64:[3,'Native::BoolResult'],
  sqlite_bind_f64:[3,'Native::BoolResult'],
  sqlite_bind_text:[3,'Native::BoolResult'],
  sqlite_bind_bytes:[3,'Native::BoolResult'],
  sqlite_step:[1,'Native::BoolResult'],
  sqlite_column_count:[1,'Native::I64Result'],
  sqlite_column_type:[2,'Native::I64Result'],
  sqlite_column_i64:[2,'Native::I64Result'],
  sqlite_column_f64:[2,'Native::F64Result'],
  sqlite_column_text:[2,'Native::TextResult'],
  sqlite_column_bytes:[2,'Native::BytesResult'],
  sqlite_finalize:[1,'Native::BoolResult'],
  sqlite_begin:[1,'Native::BoolResult'],
  sqlite_commit:[1,'Native::BoolResult'],
  sqlite_rollback:[1,'Native::BoolResult'],
  sqlite_changes:[1,'Native::I64Result'],

  abi_version:[0,'I64'],roundtrip_i64:[1,'I64'],roundtrip_bool:[1,'Bool'],copy_bytes:[1,'Bytes'],
  capability:[0,'Native::Capability'],restrict_capability:[2,'Native::CapabilityResult'],monotonic_ms:[1,'Native::I64Result'],random_bytes:[2,'Native::BytesResult'],
  listen:[2,'Native::ListenerResult'],connect:[3,'Native::StreamResult'],accept:[2,'Native::StreamResult'],
  port:[1,'Native::I64Result'],read:[3,'Native::BytesResult'],write:[3,'Native::I64Result'],
  close_listener:[1,'Native::BoolResult'],close_stream:[1,'Native::BoolResult'],shutdown_write:[1,'Native::BoolResult']
}

// Structural parsing only: collect lexical references without evaluating or
// type-checking. All state belongs to this one document snapshot/request.
class Resolver {
  constructor(tokens, options = {}) {
    this.tokens = tokens
    this.position = 0
    this.depth = 0
    this.functions = new Map()
    this.records = new Map()
    this.variants = new Map()
    this.outputs = new Map()
    this.inputs = new Map()
    this.typeParameters = new Map()
    this.functionParameters = new Map()
    this.operators = []
    this.checks = []
    this.references = []
    this.imports = new Map()
    this.localNames = new Set()
    this.declaredNames = new Set()
    this.importIds = new Map()
    this.options = options
    this.inference = tokens.some(token => token.generated)
    this.inferenceVisits = 0
    if (options.scope) {
      for (const [key, shape] of options.records) this.records.set(key, shape)
      for (const [key, shape] of options.variants) this.variants.set(key, shape)
      for (const [name, declaration] of options.scope) {
        if (declaration.kind === 'function') {
          this.functions.set(name, declaration.token)
          this.outputs.set(name, declaration.output)
          this.inputs.set(name, declaration.inputs)
          this.functionParameters.set(name, declaration.typeParameters ?? [])
        } else this[declaration.kind === 'record' ? 'records' : 'variants'].set(name, declaration)
      }
    }
  }

  variantFor(name) { return NATIVE_RESULTS.get(name) ?? this.variants.get(name) }

  current() { return this.tokens[this.position] }

  callTypes(name, explicit, arguments_) {
    this.inferenceBudget()
    const parameters = this.functionParameters.get(name) ?? []
    if (explicit.length || !parameters.length || !this.inference) return explicit.length === parameters.length ? explicit : undefined
    const inputs = this.inputs.get(name)
    if (!inputs || inputs.length !== arguments_.length) return undefined
    const variables = new Set(parameters.map(name => `@${name}`))
    const bindings = new Map()
    if (!inputs.every((type, index) => constrain(type, arguments_[index]?.(), variables, bindings))) return undefined
    const types = parameters.map(name => bindings.get(`@${name}`))
    return types.includes(undefined) ? undefined : types
  }

  functionType(name) {
    if (!this.functions.has(name) || this.functionParameters.get(name)?.length) return undefined
    const types = [...(this.inputs.get(name) ?? []), this.outputs.get(name)]
    return appliedType(callbackName(this.functions.get(name)?.actions),types)
  }

  inferenceBudget() {
    if (++this.inferenceVisits>262144) throw new SyntaxError('Type inference budget exceeded')
  }

  constructorTypes(nominal, shape, explicit, parts, expected) {
    this.inferenceBudget()
    if (!shape) return undefined
    const parameters = shape.typeParameters ?? []
    if (explicit.length || !parameters.length || !this.inference) return explicit.length === parameters.length ? explicit : undefined
    const variables = new Set(parameters.map(name => `@${name}`))
    const bindings = new Map()
    const hint = expected?.()
    if (application(hint).name === nominal && !constrain(`${nominal}<${[...variables].join(',')}>`, hint, variables, bindings)) return undefined
    for (const [pattern, actual] of parts) {
      if (!constrain(pattern, actual?.(), variables, bindings)) return undefined
    }
    const types = parameters.map(name => bindings.get(`@${name}`))
    return appliedType(nominal,types) === undefined ? undefined : types
  }

  contextualCase(name, env, expected) {
    if (!this.inference || env.has(name) || this.functions.has(name) || this.imports.has(name) ||
        this.records.has(name) || this.variants.has(name)) return undefined
    const type = expected?.()
    const parsed = application(type)
    const shape = this.variantFor(parsed.name)
    const selected = shape?.cases.get(name)
    return selected ? { type, shape, selected, arguments:parsed.arguments } : undefined
  }

  eat(text) {
    if (this.current().text !== text) return false
    this.position++
    return true
  }

  expect(text) {
    if (!this.eat(text)) throw new SyntaxError(`Expected ${text}`)
  }

  name() {
    const token = this.current()
    if (!/^[A-Za-z_][A-Za-z_0-9]*$/.test(token.text) || RESERVED.has(token.text) || (token.text.startsWith('__lobo_') && !token.generated)) {
      throw new SyntaxError('Expected identifier')
    }
    this.position++
    return token
  }

  type(depth = 0) {
    if (depth > 8) throw new SyntaxError('Type depth budget exceeded')
    if (this.eat('Fn')) {
      this.expect('(')
      const parameters = []
      if (!this.eat(')')) {
        do { if (parameters.length >= 16) throw new SyntaxError('Function signature arity exceeded'); parameters.push(this.type(depth+1)) } while (this.eat(','))
        this.expect(')')
      }
      const actions = this.actions()
      this.expect('->')
      return `${callbackName(actions)}<${[...parameters,this.type(depth+1)].join(',')}>`
    }
    if (this.eat('List')) {
      this.expect('<')
      const element = this.type(depth + 1)
      this.typeClose()
      return `List<${element}>`
    }
    if (this.inference && this.current().text === 'Native' && this.tokens[this.position+1]?.text === '::') {
      this.position+=2
      const type=`Native::${this.name().text}`
      if (!NATIVE_TYPES.has(type)) throw new SyntaxError('Unknown native type')
      return type
    }
    if (this.eat('I64')) return 'I64'
    if (this.eat('F64')) return 'F64'
    if (this.eat('Bool')) return 'Bool'
    if (this.eat('Bytes')) return 'Bytes'
    if (this.eat('Text')) return 'Text'
    const token = this.name()
    const parameter = this.typeParameters.get(token.text)
    if (parameter) {
      this.references.push({ token, target: parameter })
      return `@${token.text}`
    }
    this.references.push({ token, resolve: () => (this.records.get(token.text) ?? this.variantFor(token.text))?.token ?? this.imports.get(token.text) })
    this.checks.push(() => this.records.has(token.text) || this.variants.has(token.text) || this.imports.has(token.text))
    if (this.eat('<')) {
      const args = [this.type(depth+1)]
      while (this.eat(',')) { if (args.length >= 8) throw new SyntaxError('Type argument budget exceeded'); args.push(this.type(depth+1)) }
      this.typeClose()
      return `${this.nominal(token.text)}<${args.join(',')}>`
    }
    return this.nominal(token.text)
  }

  nominal(name) {
    if (!this.options.scope) return name
    const declaration = this.options.scope.get(name)
    if (!declaration || declaration.kind === 'function') throw new SyntaxError('Unknown nominal type')
    return `$${declaration.id}`
  }
  typeClose() {
    if (this.current().text === '>=') this.tokens[this.position] = { ...this.current(), text: '=', start: this.current().start + 1 }
    else this.expect('>')
  }

  identity() {
    this.expect('@')
    this.expect('id')
    this.expect('(')
    if (!/^"[A-Za-z0-9][A-Za-z0-9_.:\-]{0,127}"$/.test(this.current().text)) {
      throw new SyntaxError('Expected stable identity')
    }
    const id = this.current().text.slice(1, -1)
    this.position++
    this.expect(')')
    return id
  }

  declare(token) {
    this.references.push({ token, target: token })
  }

  claim(token) {
    if (this.declaredNames.has(token.text)) throw new SyntaxError('Duplicate declaration or import')
    this.declaredNames.add(token.text)
  }

  skipBlock() {
    this.expect('{')
    let depth = 1
    while (depth) {
      const text = this.current().text
      if (!text) throw new SyntaxError('Incomplete body')
      if (text === '{' && ++depth > 32) throw new SyntaxError('Nesting budget exceeded')
      if (text === '}') depth--
      this.position++
    }
  }

  bind(env, token) {
    if (env.has(token.text) || this.typeParameters.has(token.text)) throw new SyntaxError('Duplicate visible binding')
    env.set(token.text, token)
    this.localNames.add(token.text)
    this.declare(token)
  }

  program() {
    this.expect('module')
    this.name()
    this.moduleId = this.identity()
    this.expect(';')
    do {
      if (this.eat('import')) {
        const alias = this.name()
        this.claim(alias)
        this.importIds.set(alias.text, this.identity())
        this.expect(';')
        if (this.imports.has(alias.text)) throw new SyntaxError('Duplicate import alias')
        this.imports.set(alias.text, alias)
        this.references.push({ token: alias, target: this.options.scope?.get(alias.text)?.token ?? alias })
      } else if (this.eat('record')) this.record()
      else if (this.eat('variant')) this.variant()
      else if (this.eat('operator')) this.operator()
      else this.function()
    } while (this.current().text)
    if (!this.options.scope && this.records.size + this.variants.size > 64) throw new SyntaxError('Compound budget exceeded')
    for (const name of this.functions.keys()) {
      if (this.records.has(name) || this.variants.has(name)) throw new SyntaxError('Duplicate declaration')
    }
    for (const name of this.records.keys()) if (this.variants.has(name)) throw new SyntaxError('Duplicate declaration')
    for (const name of this.imports.keys()) {
      if (!this.options.scope && (this.functions.has(name) || this.records.has(name) || this.variants.has(name))) throw new SyntaxError('Import alias conflicts with declaration')
    }
    // Core also forbids binding names that collide with any module function.
    for (const name of this.localNames) {
      if (this.declaredNames.has(name)) throw new SyntaxError('Binding shadows declaration')
    }
    if (!this.options.headersOnly && this.checks.some(check => !check())) throw new SyntaxError('Unresolved record shape')
    if (!this.options.headersOnly) {
      const keys = new Set()
      for (const binding of this.operators) {
        const key = JSON.stringify([binding.symbol,binding.left,binding.right])
        if (keys.has(key) || !this.validOperator(binding)) throw new SyntaxError('Invalid operator binding')
        keys.add(key)
      }
    }
  }

  operator() {
    if (this.operators.length >= 64) throw new SyntaxError('Operator binding budget exceeded')
    const symbol = this.current().text
    if (!['+','-','*','/','%'].includes(symbol)) throw new SyntaxError('Expected arithmetic symbol')
    this.position++
    this.expect('(')
    const left = this.type()
    this.expect(',')
    const right = this.type()
    this.expect(')')
    this.expect('->')
    const output = this.type()
    this.expect('=')
    const target = this.name()
    this.expect(';')
    this.references.push({ token: target, resolve: () => this.functions.get(target.text) })
    this.operators.push({ symbol,left,right,output,target })
  }

  validOperator(binding) {
    const owns = type => {
      const name = application(type).name
      const shape = this.records.get(name) ?? this.variantFor(name)
      if (!shape || shape.typeParameters?.length) return false
      return this.options.scope ? shape.owner === binding.target.source : this.declaredNames.has(name)
    }
    const target = this.functions.get(binding.target.text)
    if (!target || !target.pure || target.typeParameters?.length) return false
    if (this.options.scope && this.options.scope.get(binding.target.text)?.owner !== binding.target.source) return false
    return (owns(binding.left) || owns(binding.right)) &&
      this.inputs.get(binding.target.text)?.join('|') === [binding.left,binding.right].join('|') &&
      this.outputs.get(binding.target.text) === binding.output
  }

  operatorBinding(symbol, left, right) {
    const bindings = this.options.operatorBindings ?? this.operators.map(binding => ({
      ...binding, token: this.functions.get(binding.target.text)
    }))
    const matches = bindings.filter(binding => binding.symbol === symbol && binding.left === left && binding.right === right)
    return matches.length === 1 ? matches[0] : undefined
  }

  variant() {
    const token = this.name()
    this.claim(token)
    const cases = new Map()
    const typeParameters = this.parseTypeParameters()
    this.variants.set(token.text, { token, cases, typeParameters })
    this.declare(token)
    token.id = this.identity()
    this.expect('{')
    do {
      const name = this.name()
      if (cases.has(name.text) || cases.size >= 16) throw new SyntaxError('Invalid variant cases')
      const payload = this.eat('(') ? this.type() : undefined
      if (payload !== undefined) this.expect(')')
      cases.set(name.text, { token: name, payload })
      this.declare(name)
    } while (this.eat(','))
    this.expect('}')
    this.typeParameters = new Map()
  }

  isMatch() {
    if (this.current().text !== 'match' || this.tokens[this.position + 1]?.text !== '(') return false
    let depth = 0
    for (let index = this.position + 1; index < this.tokens.length; index++) {
      if (this.tokens[index].text === '(') depth++
      if (this.tokens[index].text === ')' && --depth === 0) return this.tokens[index + 1]?.text === '{'
    }
    return false
  }

  match(env, expected) {
    this.expect('match')
    this.expect('(')
    const subject = this.expression(env)
    this.expect(')')
    this.expect('{')
    const shape = () => this.variantFor(application(subject?.()).name)
    const cases = () => shape()?.cases
    const arms = new Map()
    if (!this.eat('}')) {
      do {
        const name = this.name()
        if (arms.has(name.text)) throw new SyntaxError('Duplicate match case')
        const target = () => cases()?.get(name.text)
        this.references.push({ token: name, resolve: () => target()?.token })
        const binding = this.eat('(') ? this.name() : undefined
        if (binding) this.expect(')')
        const inner = new Map(env)
        if (binding) {
          binding.type = () => instantiate(target()?.payload,shape(),application(subject?.()).arguments)
          this.bind(inner, binding)
        }
        const type = this.block(inner, expected)
        arms.set(name.text, { binding, type })
      } while (this.eat(','))
      this.expect('}')
    }
    this.checks.push(() => {
      const targets = cases()
      return targets && targets.size === arms.size && [...arms].every(([name, arm]) =>
        targets.has(name) && (targets.get(name).payload !== undefined) === Boolean(arm.binding))
    })
    return () => {
      const types = [...arms.values()].map(arm => arm.type?.())
      return types.every(type => type === types[0]) ? types[0] : undefined
    }
  }

  record() {
    const token = this.name()
    this.claim(token)
    const fields = new Map()
    const typeParameters = this.parseTypeParameters()
    this.records.set(token.text, { token, fields, typeParameters })
    this.declare(token)
    token.id = this.identity()
    this.expect('{')
    do {
      const field = this.name()
      if (fields.has(field.text) || fields.size >= 16) throw new SyntaxError('Invalid record fields')
      this.expect(':')
      fields.set(field.text, { token: field, type: this.type() })
      this.declare(field)
    } while (this.eat(','))
    this.expect('}')
    this.typeParameters = new Map()
  }

  parseTypeParameters() {
    this.typeParameters = new Map()
    if (this.eat('<')) {
      do {
        const parameter = this.name()
        if (this.typeParameters.has(parameter.text) || this.typeParameters.size >= 8) throw new SyntaxError('Invalid type parameters')
        this.typeParameters.set(parameter.text,parameter)
        this.declare(parameter)
      } while (this.eat(','))
      this.expect('>')
    }
    const parameters = this.typeParameters
    this.checks.push(() => [...parameters.keys()].every(key => !this.declaredNames.has(key)))
    return [...parameters.keys()]
  }

  function() {
    this.expect('function')
    const name = this.name()
    this.claim(name)
    this.functions.set(name.text, name)
    this.declare(name)
    this.typeParameters = new Map()
    if (this.eat('<')) {
      do {
        const parameter = this.name()
        if (this.typeParameters.has(parameter.text) || this.typeParameters.size >= 8) throw new SyntaxError('Invalid type parameters')
        this.typeParameters.set(parameter.text,parameter)
        this.declare(parameter)
      } while (this.eat(','))
      this.expect('>')
    }
    name.typeParameters = [...this.typeParameters.keys()]
    this.functionParameters.set(name.text,name.typeParameters)
    const typeParameters = this.typeParameters
    this.checks.push(() => [...typeParameters.keys()].every(key => !this.declaredNames.has(key)))
    name.id = this.identity()
    this.expect('(')
    const parameters = new Map()
    if (!this.eat(')')) {
      do {
        const parameter = this.name()
        this.bind(parameters, parameter)
        this.expect(':')
        const type = this.type()
        parameter.type = () => type
      } while (this.eat(','))
      this.expect(')')
    }
    this.expect('->')
    const output = this.type()
    this.inputs.set(name.text,[...parameters.values()].map(parameter => parameter.type()))
    this.outputs.set(name.text, output)
    name.type = () => output
    name.actions = this.actions()
    name.pure = name.actions.length === 0
    if (this.options.headersOnly) {
      while (this.eat('requires')) this.skipBlock()
      while (this.eat('ensures')) this.skipBlock()
      this.skipBlock()
      this.typeParameters = new Map()
      return
    }
    while (this.eat('requires')) this.contract(parameters)
    const post = new Map(parameters)
    post.set('result', name)
    while (this.eat('ensures')) this.contract(post)
    this.block(parameters, () => output)
    this.typeParameters = new Map()
  }

  actions() {
    if (!this.eat('actions')) return []
    this.expect('{')
    const seen = new Set()
    if (!this.eat('}')) {
      do {
        const parts = [this.name().text]
        this.expect('.')
        do { parts.push(this.name().text) } while (this.eat('.'))
        const id = parts.join('.')
        if (id.length > 128 || seen.has(id) || seen.size >= 64) throw new SyntaxError('Invalid action set')
        seen.add(id)
      } while (this.eat(','))
      this.expect('}')
    }
    return [...seen].sort()
  }

  contract(env) {
    this.expect('{')
    this.expression(env)
    this.expect('}')
  }

  block(outer, expected) {
    this.expect('{')
    const env = new Map(outer)
    while (true) {
      if (this.eat('let')) {
        const name = this.name()
        const annotation = this.eat(':') ? this.type() : undefined
        this.expect('=')
        const inferred = this.expression(env, 0, annotation ? () => annotation : undefined)
        name.type = annotation ? () => annotation : inferred
        this.expect(';')
        this.bind(env, name)
        continue
      }
      const type = this.expression(env, 0, expected)
      if (this.eat(';')) {
        if (!this.inference || !type?.propagation) throw new SyntaxError('Only standalone try may discard a value')
        this.checks.push(() => type() !== undefined)
        continue
      }
      this.expect('}')
      return type
    }
  }

  expression(env, level = 0, expected) {
    if (level === OPERATORS.length) return this.primary(env, expected)
    let type = this.expression(env, level + 1, expected)
    let count = 0
    while (OPERATORS[level].includes(this.current().text)) {
      if ((level === 2 || level === 3) && count > 0) break
      if (++count > 32) throw new SyntaxError('Expression budget exceeded')
      const token = this.tokens[this.position++]
      const left = type
      const right = this.expression(env, level + 1, expected)
      if (level >= 4) {
        const binding = () => this.operatorBinding(token.text, left?.(), right?.())
        this.references.push({ token, resolve: () => binding()?.token })
        type = () => binding()?.output ?? (left?.() === right?.() ? left?.() : undefined)
      } else type = () => 'Bool'
    }
    return type
  }

  primary(env, expected) {
    if (++this.depth > 32) throw new SyntaxError('Nesting budget exceeded')
    let type = this.primaryInner(env, expected)
    let fields = 0
    while (this.current().text === '.' || this.current().text === '(') {
      if (++fields > 32) throw new SyntaxError('Field depth budget exceeded')
      if (this.eat('(')) {
        if (!this.eat(')')) { do { this.expression(env) } while (this.eat(',')); this.expect(')') }
        const callee = type
        type = () => { const parsed = application(callee?.()); return isCallback(parsed.name) ? parsed.arguments.at(-1) : undefined }
        continue
      }
      this.expect('.')
      const field = this.name()
      const base = type
      const shape = () => this.records.get(application(base?.()).name)
      const definition = () => shape()?.fields.get(field.text)
      this.references.push({ token: field, resolve: () => definition()?.token })
      this.checks.push(() => Boolean(definition()))
      type = () => instantiate(definition()?.type,shape(),application(base?.()).arguments)
    }
    this.depth--
    return type
  }

  primaryInner(env, expected) {
    if (this.current().text === 'try' && /^[A-Za-z_0-9"]/.test(this.tokens[this.position+1]?.text ?? '')) {
      this.position++
      const value = this.primary(env)
      const type = () => {
        const parsed = application(value?.())
        const shape = this.variantFor(parsed.name)
        if (shape?.cases.size !== 2 || shape.cases.get('Ok')?.payload === undefined || shape.cases.get('Err')?.payload === undefined) return undefined
        return instantiate(shape.cases.get('Ok').payload,shape,parsed.arguments)
      }
      type.propagation = true
      return type
    }
    if (this.eat('&')) {
      const token = this.name()
      const args = []
      if (this.eat('::')) {
        this.expect('<')
        do { if (args.length >= 8) throw new SyntaxError('Type argument budget exceeded'); args.push(this.type()) } while (this.eat(','))
        this.expect('>')
      }
      this.references.push({ token, resolve: () => this.functions.get(token.text) ?? this.imports.get(token.text) })
      return () => {
        const atom = name => name.startsWith('@') ? args[this.functionParameters.get(token.text)?.indexOf(name.slice(1))] : name
        const types = [...(this.inputs.get(token.text) ?? []),this.outputs.get(token.text)].map(type => mapType(type,atom))
        return appliedType(callbackName((this.functions.get(token.text) ?? this.imports.get(token.text))?.actions),types)
      }
    }
    if (this.current().text.startsWith('"')) {
      const value = JSON.parse(this.current().text)
      for (let index = 0; index < value.length; index++) {
        const unit = value.charCodeAt(index)
        if (unit >= 0xd800 && unit <= 0xdbff) {
          const next = value.charCodeAt(++index)
          if (!(next >= 0xdc00 && next <= 0xdfff)) throw new SyntaxError('Unpaired surrogate')
        } else if (unit >= 0xdc00 && unit <= 0xdfff) throw new SyntaxError('Unpaired surrogate')
      }
      this.position++
      return () => 'Text'
    }
    if (['Bytes','Text','F64'].includes(this.current().text) || (this.inference && this.current().text === 'Native' && this.tokens[this.position+1]?.text === '::')) {
      const family = this.current().text
      this.position++; this.expect('::')
      const op = this.name().text
      const signatures = family === 'Native' ? NATIVE_SIGNATURES : family === 'F64'
        ? { from_i64:[1,'F64'],to_i64:[1,'I64'],from_bits:[1,'F64'],to_bits:[1,'I64'],is_finite:[1,'Bool'],is_nan:[1,'Bool'] }
        : family === 'Bytes'
        ? { empty: [0,'Bytes'], length: [1,'I64'], at: [2,'I64'], slice: [3,'Bytes'], push: [2,'Bytes'], concat: [2,'Bytes'], from_list: [1,'Bytes'] }
        : { bytes: [1,'Bytes'], valid: [1,'Bool'], from_bytes: [1,'Text'], length: [1,'I64'], at: [2,'I64'], slice: [3,'Text'], concat: [2,'Text'] }
      if (!Object.hasOwn(signatures,op)) throw new SyntaxError('Unknown buffer operation')
      const [arity,type] = signatures[op]
      this.expect('(')
      let count = 0
      if (!this.eat(')')) {
        do { this.expression(env); count++ } while (this.eat(','))
        this.expect(')')
      }
      if (count !== arity) throw new SyntaxError('Buffer operation arity')
      return () => type
    }
    if (this.eat('List')) {
      this.expect('::')
      const op = this.name().text
      if (op === 'empty') {
        this.expect('<')
        const element = this.type()
        this.expect('>')
        this.expect('('); this.expect(')')
        return () => `List<${element}>`
      }
      const arity = op === 'prepend' ? 2 : ['length', 'head', 'tail'].includes(op) ? 1 : 0
      if (!arity) throw new SyntaxError('Unknown list operation')
      this.expect('(')
      const args = []
      if (!this.eat(')')) {
        do { args.push(this.expression(env)) } while (this.eat(','))
        this.expect(')')
      }
      if (args.length !== arity) throw new SyntaxError('List operation arity')
      const list = () => args.at(-1)?.()
      this.checks.push(() => list()?.startsWith('List<') && list().endsWith('>'))
      return () => op === 'length' ? 'I64' : op === 'head' ? list()?.slice(5, -1) : list()
    }
    if (this.isMatch()) return this.match(env, expected)
    if (this.eat('!')) { this.primary(env); return () => 'Bool' }
    if (this.eat('-')) { this.primary(env); return () => 'I64' }
    if (this.eat('(')) {
      const type = this.expression(env, 0, expected)
      this.expect(')')
      return type
    }
    if (this.eat('if')) {
      this.expression(env)
      const thenType = this.block(env, expected)
      this.expect('else')
      const elseType = this.block(env, expected)
      return () => thenType?.() === elseType?.() ? thenType?.() : undefined
    }
    const prefix = this.tokens.slice(this.position, this.position + 4).map(token => token.text)
    if (prefix[0] === 'while' && prefix[1] === '(' &&
        /^[A-Za-z_][A-Za-z_0-9]*$/.test(prefix[2]) && prefix[3] === ':') {
      return this.while(env)
    }
    if (prefix[0] === 'fold' && prefix[1] === '(' &&
        /^[A-Za-z_][A-Za-z_0-9]*$/.test(prefix[2]) && prefix[3] === 'in') {
      return this.fold(env)
    }
    if (/^[0-9]+(?:\.[0-9]+)?(?:[eE][+\-]?[0-9]+)?$/.test(this.current().text) || ['true', 'false'].includes(this.current().text)) {
      const type = ['true', 'false'].includes(this.current().text) ? 'Bool' : /[.eE]/.test(this.current().text) ? 'F64' : 'I64'
      this.position++
      return () => type
    }
    const token = this.current().text === 'result' ? this.tokens[this.position++] : this.name()
    const typeArguments = []
    if (this.current().text === '::' && this.tokens[this.position + 1]?.text === '<') {
      this.position += 2
      do {
        if (typeArguments.length >= 8) throw new SyntaxError('Type argument budget exceeded')
        typeArguments.push(this.type())
      } while (this.eat(','))
      this.expect('>')
      if (!['(', '{', '::'].includes(this.current().text)) throw new SyntaxError('Expected generic application')
    }
    if (this.eat('::')) {
      const name = this.name()
      const target = () => this.variantFor(token.text)?.cases.get(name.text)
      this.references.push({ token, resolve: () => this.variantFor(token.text)?.token })
      this.references.push({ token: name, resolve: () => target()?.token })
      const nominal = this.nominal(token.text)
      const shape = () => this.variantFor(token.text)
      let value
      const types = guarded(() => this.constructorTypes(nominal, shape(), typeArguments, value ? [[target()?.payload,value]] : [], expected))
      const payload = this.eat('(')
      if (payload) { value = this.expression(env, 0, () => instantiate(target()?.payload,shape(),types() ?? [])); this.expect(')') }
      this.checks.push(() => target() && (target().payload !== undefined) === payload && types() !== undefined)
      return () => { const args=types(); return args === undefined ? undefined : appliedType(nominal,args) }
    }
    if (this.current().text === '{' && (this.tokens[this.position+1]?.text === '...' ||
        (/^[A-Za-z_][A-Za-z_0-9]*$/.test(this.tokens[this.position + 1]?.text) && this.tokens[this.position + 2]?.text === ':'))) {
      this.expect('{')
      this.references.push({ token, resolve: () => this.records.get(token.text)?.token })
      const names = new Set()
      const values = new Map()
      const shape = () => this.records.get(token.text)
      const nominal = this.nominal(token.text)
      let base
      const types = guarded(() => {
        const parts=[...values].map(([name,value]) => [shape()?.fields.get(name)?.type,value])
        if (base) parts.unshift([appliedType(nominal,(shape()?.typeParameters ?? []).map(p=>'@'+p)),base])
        return this.constructorTypes(nominal,shape(),typeArguments,parts,expected)
      })
      if (this.eat('...')) {
        if (!this.inference) throw new SyntaxError('Record updates require modern source')
        base=this.expression(env,0,() => {const args=types();return args===undefined?undefined:appliedType(nominal,args)})
      }
      const fieldsPresent=base ? this.eat(',') : true
      if (fieldsPresent) {
        do {
          const field = this.name()
          if (names.has(field.text)) throw new SyntaxError('Duplicate constructor field')
          names.add(field.text)
          this.references.push({ token: field, resolve: () => this.records.get(token.text)?.fields.get(field.text)?.token })
          this.expect(':')
          values.set(field.text,this.expression(env,0,() => instantiate(shape()?.fields.get(field.text)?.type,shape(),types() ?? [])))
        } while (this.eat(','))
      }
      this.expect('}')
      this.checks.push(() => {
        const fields=shape()?.fields,args=types()
        return fields && (base || fields.size===names.size) && [...names].every(name=>fields.has(name)) && args!==undefined &&
          (!base || base()===appliedType(nominal,args))
      })
      return () => { const args=types(); return args === undefined ? undefined : appliedType(nominal,args) }
    }
    const target = env.get(token.text)
    const contextual = () => this.contextualCase(token.text,env,expected)
    this.references.push({ token, resolve:() => target ?? this.functions.get(token.text) ?? this.imports.get(token.text) ?? contextual()?.selected.token })
    if (!this.eat('(')) return () => target?.type?.() ?? this.functionType(token.text) ?? contextual()?.type
    const arguments_ = []
    const types = guarded(() => this.callTypes(token.text, typeArguments, arguments_))
    const argumentHint = index => () => {
      const selected = contextual()
      if (selected) return instantiate(selected.selected.payload,selected.shape,selected.arguments)
      if (target && !this.functions.has(token.text)) return application(target.type?.()).arguments[index]
      return instantiate(this.inputs.get(token.text)?.[index], { typeParameters:this.functionParameters.get(token.text) }, types() ?? [])
    }
    if (!this.eat(')')) {
      do { arguments_.push(this.expression(env,0,argumentHint(arguments_.length))) } while (this.eat(','))
      this.expect(')')
    }
    this.checks.push(() => !this.functionParameters.has(token.text) || types() !== undefined)
    if (target && !this.functions.has(token.text)) return () => {
      const parsed = application(target.type?.())
      return isCallback(parsed.name) ? parsed.arguments.at(-1) : undefined
    }
    const substitute = type => mapType(type, atom => {
      if (atom.startsWith('@')) {
        const index = this.functionParameters.get(token.text)?.indexOf(atom.slice(1))
        return index === undefined || index < 0 ? undefined : types()?.[index]
      }
      return atom
    })
    return () => contextual()?.type ?? substitute(this.outputs.get(token.text))
  }

  while(outer) {
    this.expect('while')
    this.expect('(')
    const state = this.name()
    this.expect(':')
    const type = this.type()
    this.expect('=')
    this.expression(outer,0,() => type)
    this.expect(';')
    const inner = new Map(outer)
    this.bind(inner, state)
    state.type = () => type
    this.expression(inner)
    this.expect(')')
    this.block(inner,() => type)
    return () => type
  }

  fold(outer) {
    this.expect('fold')
    this.expect('(')
    const index = this.name()
    this.expect('in')
    this.expression(outer)
    this.expect(',')
    const accumulator = this.name()
    this.expect(':')
    const type = this.type()
    this.expect('=')
    this.expression(outer,0,() => type)
    this.expect(')')
    const inner = new Map(outer)
    this.bind(inner, index)
    index.type = () => 'I64'
    this.bind(inner, accumulator)
    accumulator.type = () => type
    this.block(inner,() => type)
    return () => type
  }

  definition(offset) {
    const reference = this.references.find(({ token }) => token.start <= offset && offset < token.end)
    if (!reference) return undefined
    const target = reference.resolve ? reference.resolve() : reference.target ?? this.functions.get(reference.token.text) ?? this.imports.get(reference.token.text)
    if (!target) return undefined
    return { origin: range(reference.token), target: range(target), ...(target.source ? { source: target.source } : {}) }
  }
}

function definitionAt(source, offset) {
  if (!Number.isInteger(offset) || offset < 0 || offset >= source.length) return undefined
  try {
    const resolver = new Resolver(tokenize(source))
    resolver.program()
    return resolver.definition(offset)
  } catch (error) {
    if (error instanceof SyntaxError) return undefined
    throw error
  }
}

function definitionInBundle(sources, root, offset) {
  try {
    if (!Array.isArray(sources) || !sources.length || sources.length > 16) return undefined
    const parsed = new Map()
    const ids = new Map()
    let bytes = 0
    let tokens = 0
    let slots = 0
    let compounds = 0
    for (const source of sources) {
      if (typeof source?.name !== 'string' || !source.name || Buffer.byteLength(source.name) > 1024 ||
          source.name.includes('\0') || parsed.has(source.name) || typeof source.text !== 'string') return undefined
      bytes += Buffer.byteLength(source.text, 'utf8') +
        (source.identities === undefined ? 0 : Buffer.byteLength(JSON.stringify(source.identities)))
      if (bytes > 1_048_576) return undefined
      const lexed = tokenize(source.text, source.identities).map(token => ({ ...token, source: source.name }))
      tokens += lexed.length - 1
      if (tokens > 16_384) return undefined
      const resolver = new Resolver(lexed, { headersOnly: true })
      resolver.program()
      if (ids.has(resolver.moduleId)) return undefined
      ids.set(resolver.moduleId, { kind: 'module' })
      const own = new Map()
      for (const [kind, entries] of [['function', resolver.functions], ['record', resolver.records], ['variant', resolver.variants]]) {
        for (const [name, value] of entries) {
          const token = kind === 'function' ? value : value.token
          if (ids.has(token.id)) return undefined
          const declaration = { kind, id: token.id, token, owner: source.name, ...kind === 'function'
            ? { output: resolver.outputs.get(name), inputs: resolver.inputs.get(name), typeParameters: token.typeParameters } : value }
          own.set(name, declaration)
          ids.set(token.id, declaration)
          slots++
          if (kind !== 'function') compounds++
        }
      }
      slots += resolver.importIds.size
      if (slots > 1024 || compounds > 64) return undefined
      parsed.set(source.name, { source, resolver, own })
    }
    if (!parsed.has(root) || !Number.isInteger(offset) || offset < 0 || offset >= parsed.get(root).source.text.length) return undefined
    for (const [label, entry] of parsed) {
      entry.scope = new Map(entry.own)
      for (const [alias, id] of entry.resolver.importIds) {
        const target = ids.get(id)
        if (!target || target.kind === 'module' || target.owner === label || entry.scope.has(alias)) return undefined
        entry.scope.set(alias, target)
      }
    }
    const canonicalType = (type, scope) => mapType(type, atom => {
      if (NATIVE_TYPES.has(atom) || isCallback(atom) || atom.startsWith('@') || ['Fn', 'List', 'I64', 'F64', 'Bool', 'Bytes', 'Text'].includes(atom)) return atom
      const target = scope.get(atom)
      if (!target || target.kind === 'function') throw new SyntaxError('Unresolved nominal type')
      return `$${target.id}`
    })
    const records = new Map()
    const variants = new Map()
    for (const entry of parsed.values()) {
      for (const declaration of entry.own.values()) {
        if (declaration.kind === 'function') {
          declaration.output = canonicalType(declaration.output, entry.scope)
          declaration.inputs = declaration.inputs.map(type => canonicalType(type,entry.scope))
        }
        if (declaration.kind === 'record') {
          declaration.fields = new Map([...declaration.fields].map(([name, field]) =>
            [name, { ...field, type: canonicalType(field.type, entry.scope) }]))
          records.set(`$${declaration.id}`, declaration)
        }
        if (declaration.kind === 'variant') {
          declaration.cases = new Map([...declaration.cases].map(([name, arm]) =>
            [name, { ...arm, payload: arm.payload === undefined ? undefined : canonicalType(arm.payload, entry.scope) }]))
          variants.set(`$${declaration.id}`, declaration)
        }
      }
    }
    const operatorBindings = []
    const operatorKeys = new Set()
    for (const [label, entry] of parsed) {
      for (const binding of entry.resolver.operators) {
        const left = canonicalType(binding.left,entry.scope)
        const right = canonicalType(binding.right,entry.scope)
        const output = canonicalType(binding.output,entry.scope)
        const target = entry.own.get(binding.target.text)
        const owns = type => [...entry.own.values()].some(d => d.kind !== 'function' &&
          type === `$${d.id}` && !d.typeParameters?.length)
        const key = JSON.stringify([binding.symbol,left,right])
        if (!target || target.kind !== 'function' || target.owner !== label || !target.token.pure ||
            target.typeParameters?.length || target.inputs.join('|') !== [left,right].join('|') ||
            target.output !== output || !(owns(left) || owns(right)) || operatorKeys.has(key) ||
            operatorBindings.length >= 64) return undefined
        operatorKeys.add(key)
        operatorBindings.push({ symbol:binding.symbol,left,right,output,token:target.token,id:target.id })
      }
    }
    let result
    for (const [label, entry] of parsed) {
      const resolver = new Resolver(tokenize(entry.source.text, entry.source.identities).map(token => ({ ...token, source: label })),
        { scope: entry.scope, records, variants,
          operatorBindings: operatorBindings.filter(binding => [...entry.scope.values()].some(d => d.id === binding.id)) })
      resolver.program()
      if (label === root) result = resolver.definition(offset)
    }
    return result
  } catch (error) {
    if (error instanceof SyntaxError || error instanceof RangeError) return undefined
    throw error
  }
}

module.exports = { definitionAt, definitionInBundle }
