import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { createRequire } from 'node:module'
import test from 'node:test'
import textmate from 'vscode-textmate'
import oniguruma from 'vscode-oniguruma'

const require = createRequire(import.meta.url)
const wasm = await readFile(require.resolve('vscode-oniguruma/release/onig.wasm'))
await oniguruma.loadWASM(wasm)
const grammarPath = new URL('../syntaxes/lobo.tmLanguage.json', import.meta.url)
const registry = new textmate.Registry({
  onigLib: Promise.resolve({
    createOnigScanner: patterns => new oniguruma.OnigScanner(patterns),
    createOnigString: value => new oniguruma.OnigString(value)
  }),
  loadGrammar: async scope => scope === 'source.lobo'
    ? textmate.parseRawGrammar(await readFile(grammarPath, 'utf8'), grammarPath.pathname)
    : null
})
const grammar = await registry.loadGrammar('source.lobo')
assert.ok(grammar)

function tokenize(source) {
  let stack = textmate.INITIAL
  return source.split(/\r?\n/).map(line => {
    const result = grammar.tokenizeLine(line, stack)
    assert.equal(result.stoppedEarly, false)
    stack = result.ruleStack
    return result.tokens.map(token => ({
      text: line.slice(token.startIndex, token.endIndex), scopes: token.scopes
    }))
  })
}

function expectScope(source, text, scope) {
  const tokens = tokenize(source).flat().filter(token => token.text === text)
  assert.ok(tokens.length, `Missing token ${JSON.stringify(text)}`)
  for (const token of tokens) assert.ok(token.scopes.includes(scope), JSON.stringify(token))
}

test('compact authoring declarations, imports, JSON and match arms have scopes', () => {
  const source = '// lobo:compact/0.1\nfn render(user: User) -> Json { json { "name": user.name } }\nuse http { Response, not_found };\nidentities { module: "mvc" }\nmatch find(id) { Missing => http.not_found() }'
  expectScope(source, 'fn', 'storage.type.function.lobo')
  expectScope(source, 'render', 'entity.name.function.lobo')
  expectScope(source, 'use', 'keyword.control.import.lobo')
  expectScope(source, 'identities', 'storage.type.metadata.lobo')
  expectScope(source, 'json', 'support.function.json.lobo')
  expectScope(source, 'match', 'keyword.control.conditional.lobo')
  expectScope(source, '=>', 'keyword.operator.lobo')
  expectScope('// lobo:compact/0.1\nhttp.not_found()', 'not_found', 'entity.name.function.lobo')
  expectScope('// lobo:compact/0.1\nuse http { not_found };', 'not_found', 'entity.name.import.lobo')
})

test('modern syntax highlights without an opt-in marker', () => {
  expectScope('module app;\nfn value(n:I64)->I64 {n}', 'fn', 'storage.type.function.lobo')
  expectScope('use api { Response };', 'use', 'keyword.control.import.lobo')
  expectScope('identities { module: "app" }', 'identities', 'storage.type.metadata.lobo')
  expectScope('json { "name": value }', 'json', 'support.function.json.lobo')
  expectScope('match answer { None => 0 }', '=>', 'keyword.operator.lobo')
  expectScope('fn identity<T>(x:T)->T {x}', 'T', 'entity.name.type.lobo')
})

test('operator binding declarations highlight the keyword, symbol and named target', () => {
  const source = 'operator +(Vec2, Vec2) -> Vec2 = vector_add;'
  expectScope(source, 'operator', 'storage.type.operator.lobo')
  expectScope(source, '+', 'keyword.operator.lobo')
  expectScope(source, 'vector_add', 'entity.name.function.lobo')
  expectScope('operator +(Vec2,Vec2)->Vec2 = // exact target\n vector_add;', 'vector_add', 'entity.name.function.lobo')
})

test('generic function parameters and explicit calls have type and function scopes', () => {
  expectScope('function identity<T> @id("id")(x: T) -> T { x }', 'T', 'entity.name.type.lobo')
  expectScope('identity::<List<Item>>(xs)', 'identity', 'entity.name.function.lobo')
  expectScope('identity::<List<Item>>(xs)', 'Item', 'entity.name.type.lobo')
})

test('generic nominal declarations and nested constructor arguments have type scopes', () => {
  expectScope('record Cursor<T> @id("cursor") { rest: List<T> }', 'Cursor', 'entity.name.type.lobo')
  expectScope('record Cursor<T> @id("cursor") { rest: List<T> }', 'T', 'entity.name.type.lobo')
  expectScope('variant Option<T> @id("option") { None, Some(T) }', 'Option', 'entity.name.type.lobo')
  expectScope('Option::<Cursor<Item>>::Some(x)', 'Item', 'entity.name.type.lobo')
})

test('pure function types and named references have scopes', () => {
  expectScope('let f: Fn(I64) -> I64 = &increment;', 'Fn', 'support.type.lobo')
  expectScope('let f: Fn(I64) -> I64 = &increment;', '&', 'keyword.operator.lobo')
})

test('F64 types, operations and fractional/exponent literals have scopes', () => {
  expectScope('let x: F64 = F64::from_i64(2) + 1.5e-2;', 'F64', 'support.type.lobo')
  expectScope('F64::from_i64(2)', 'from_i64', 'support.function.buffer.lobo')
  expectScope('1.5e-2', '1.5e-2', 'constant.numeric.float.lobo')
})

test('highlights module and function declarations, identity strings, and Core types', () => {
  const source = 'module sample @id("sample.module");\nfunction sum @id("sample.sum")(n: I64) -> Bool { true }'
  expectScope(source, 'module', 'storage.type.module.lobo')
  expectScope(source, 'sample', 'entity.name.namespace.lobo')
  expectScope(source, 'function', 'storage.type.function.lobo')
  expectScope(source, 'sum', 'entity.name.function.lobo')
  expectScope(source, 'id', 'entity.name.function.annotation.lobo')
  expectScope(source, 'sample.sum', 'string.quoted.double.lobo')
  expectScope(source, 'I64', 'support.type.lobo')
  expectScope(source, 'Bool', 'support.type.lobo')
})

test('highlights selected imports while preserving ordinary import function calls', () => {
  expectScope('import reverse @id("lib.reverse");','import','keyword.control.import.lobo')
  expectScope('import reverse @id("lib.reverse");','reverse','entity.name.import.lobo')
  expectScope('import(value)','import','entity.name.function.lobo')
})

test('highlights bindings, contracts, conditionals, booleans, and result', () => {
  const source = 'let value: I64 = 42; requires { true } ensures { result > 0 } if false { 0 } else { value }'
  expectScope(source, 'let', 'storage.type.lobo')
  expectScope(source, 'requires', 'keyword.control.contract.lobo')
  expectScope(source, 'ensures', 'keyword.control.contract.lobo')
  expectScope(source, 'if', 'keyword.control.conditional.lobo')
  expectScope(source, 'else', 'keyword.control.conditional.lobo')
  expectScope(source, 'true', 'constant.language.boolean.lobo')
  expectScope(source, 'false', 'constant.language.boolean.lobo')
  expectScope(source, 'result', 'variable.language.result.lobo')
  expectScope(source, '42', 'constant.numeric.integer.lobo')
})

test('highlights record declarations, constructor labels and field selectors', () => {
  expectScope('record Pair @id("pair") { left: I64 }', 'Pair', 'entity.name.type.lobo')
  expectScope('record Pair @id("pair") { left: I64 }', 'record', 'storage.type.record.lobo')
  expectScope('Pair { left: 1 }', 'left', 'variable.other.property.lobo')
  expectScope('p.left', 'left', 'variable.other.property.lobo')
  expectScope('record()', 'record', 'entity.name.function.lobo')
})

test('highlights variant declarations, payload types, qualified cases and contextual match', () => {
  const source = 'variant Answer @id("answer") { Ok(Pair), Empty }'
  expectScope(source, 'variant', 'storage.type.variant.lobo')
  expectScope(source, 'Answer', 'entity.name.type.lobo')
  expectScope(source, 'Ok', 'entity.name.constant.lobo')
  expectScope(source, 'Pair', 'entity.name.type.lobo')
  expectScope(source, 'Empty', 'entity.name.constant.lobo')
  expectScope('Answer::Ok(42)', 'Answer', 'entity.name.type.lobo')
  expectScope('Answer::Ok(42)', 'Ok', 'entity.name.constant.lobo')
  expectScope('Answer::Empty', '::', 'punctuation.accessor.lobo')
  expectScope('match (x) { Ok(n) { n }, Empty { 0 } }', 'match', 'keyword.control.conditional.lobo')
  expectScope('match(42)', 'match', 'entity.name.function.lobo')
  expectScope('variant()', 'variant', 'entity.name.function.lobo')
})

test('highlights nested list element types and intrinsic operations', () => {
  const source = 'record Row @id("row") { values: List<List<Pair>> }'
  expectScope(source,'List','support.type.lobo')
  expectScope(source,'Pair','entity.name.type.lobo')
  expectScope('List::empty<List<Pair>>()','Pair','entity.name.type.lobo')
  for (const op of ['empty','prepend','head','tail','length']) {
    expectScope(`List::${op}${op === 'empty' ? '<I64>()' : '(xs)'}`,op,'support.function.list.lobo')
  }
})

test('recursive variant payloads use existing nominal and list scopes', () => {
  const source = 'variant Node @id("node") { Num(I64), Branch(List<Node>) }'
  expectScope(source, 'Node', 'entity.name.type.lobo')
  expectScope(source, 'List', 'support.type.lobo')
  expectScope(source, 'Branch', 'entity.name.constant.lobo')
})

test('text and byte operations and JSON escapes retain their proper scopes', () => {
  for (const type of ['Text','Bytes']) expectScope(`x: ${type}`,type,'support.type.lobo')
  expectScope('Text::from_bytes(xs)','from_bytes','support.function.buffer.lobo')
  expectScope('Bytes::slice(xs,0,1)','slice','support.function.buffer.lobo')
  expectScope(String.raw`"line\n quote\" end"`,'\\n','constant.character.escape.lobo')
  const tokens=tokenize(String.raw`let t = "a\" // not comment"; t`).flat()
  assert.ok(tokens.some(token=>token.text.includes('// not comment')&&token.scopes.includes('string.quoted.double.lobo')))
  assert.ok(tokens.at(-1).scopes.includes('variable.other.lobo'))
})

test('highlights action clauses without reserving ordinary actions names', () => {
  expectScope('function f @id("f")() -> I64 actions { demo.observe } { 42 }', 'actions', 'keyword.control.action.lobo')
  expectScope('actions { demo.observe }', 'demo.observe', 'support.constant.action.lobo')
  expectScope('if actions { 1 } else { 0 }', 'actions', 'variable.other.lobo')
  expectScope('actions(42)', 'actions', 'entity.name.function.lobo')
  expectScope('function f @id("f")() -> Pair actions { demo.observe } { p }', 'actions', 'keyword.control.action.lobo')
})

test('recognizes fold/in only in the unambiguous counted-fold prefix', () => {
  const source = 'fold (i in n, a: I64 = 0) { fold(j in 2, b: I64 = a) { b + i + j } }'
  expectScope(source, 'fold', 'keyword.control.loop.lobo')
  expectScope(source, 'in', 'keyword.control.loop.lobo')
  expectScope(source, 'i', 'variable.other.lobo')
  const ordinary = 'function fold @id("f")(in: I64) -> I64 { let id = in; fold(in) + id }'
  const tokens = tokenize(ordinary).flat()
  for (const token of tokens.filter(token => ['fold', 'in', 'id'].includes(token.text))) {
    assert.ok(!token.scopes.some(scope => scope.startsWith('keyword.')), JSON.stringify(token))
  }
  expectScope('fold(in)', 'fold', 'entity.name.function.lobo')
  expectScope('fold(in)', 'in', 'variable.other.lobo')
})

test('highlights typed-state while prefixes without reserving ordinary calls', () => {
  expectScope('while (state: I64 = 0; state < 3) { state + 1 }', 'while', 'keyword.control.loop.lobo')
  expectScope('while(value)', 'while', 'entity.name.function.lobo')
})

test('comments consume the rest of a line but do not leak across CRLF', () => {
  const lines = tokenize('// function fake I64 42 "id"\r\nlet n = 7; // fold (i in n)')
  for (const token of lines[0]) assert.ok(token.scopes.includes('comment.line.double-slash.lobo'))
  assert.ok(!lines[1][0].scopes.some(scope => scope.startsWith('comment.')))
  assert.ok(lines[1].at(-1).scopes.includes('comment.line.double-slash.lobo'))
})

test('quoted identity contents do not become comments, keywords, or integers', () => {
  const tokens = tokenize('"module//fold42"').flat()
  assert.ok(tokens.some(token => token.text === 'module//fold42'))
  for (const token of tokens) {
    assert.ok(token.scopes.includes('string.quoted.double.lobo'))
    assert.ok(!token.scopes.some(scope => /^(comment|keyword|constant\.numeric)\./.test(scope)))
  }
  const lines = tokenize('"unfinished\nlet n = 1;')
  assert.ok(!lines[1][0].scopes.includes('string.quoted.double.lobo'))
})

test('highlights every Core operator without splitting multi-character operators', () => {
  for (const operator of ['+', '-', '*', '/', '%', '==', '!=', '<', '<=', '>', '>=', '!', '&&', '||', '=', '->']) {
    expectScope(`a ${operator} b`, operator, 'keyword.operator.lobo')
  }
  expectScope('-9223372036854775808', '9223372036854775808', 'constant.numeric.integer.lobo')
  for (const identifier of ['I64_count', 'falsehood', 'requires_more', 'folded', 'inner', 'x42']) {
    expectScope(identifier, identifier, 'variable.other.lobo')
  }
})

test('does not advertise unsupported block comments or illustrative keywords as Core', () => {
  expectScope('component record enum return while', 'component', 'variable.other.lobo')
  expectScope('actions', 'actions', 'variable.other.lobo')
  expectScope('actions(42)', 'actions', 'entity.name.function.lobo')
  const tokens = tokenize('/* if true */').flat()
  assert.ok(tokens.every(token => !token.scopes.some(scope => scope.startsWith('comment.'))))
})

test('registers .lobo highlighting, local navigation, and supported editor behavior', async () => {
  const manifest = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'))
  assert.equal(manifest.main, './src/extension.cjs')
  await readFile(new URL(`../${manifest.main}`, import.meta.url))
  assert.equal(manifest.browser, undefined)
  assert.equal(manifest.activationEvents, undefined)
  assert.equal(manifest.dependencies, undefined)
  assert.equal(manifest.capabilities.untrustedWorkspaces.supported, 'limited')
  assert.ok(manifest.capabilities.untrustedWorkspaces.restrictedConfigurations.includes('lobo.compilerPath'))
  assert.ok(manifest.capabilities.untrustedWorkspaces.restrictedConfigurations.includes('lobo.sourceFiles'))
  assert.deepEqual(manifest.contributes.configuration.properties['lobo.sourceFiles'].default, [])
  assert.equal(manifest.contributes.configuration.properties['lobo.diagnostics.enabled'].default, true)
  const language = manifest.contributes.languages.find(language => language.id === 'lobo')
  assert.deepEqual(language.extensions, ['.lobo'])
  const contribution = manifest.contributes.grammars.find(grammar => grammar.language === 'lobo')
  assert.equal(contribution.scopeName, 'source.lobo')
  const contributed = JSON.parse(await readFile(new URL(`../${contribution.path}`, import.meta.url), 'utf8'))
  assert.equal(contributed.scopeName, contribution.scopeName)
  const config = JSON.parse(await readFile(new URL(`../${language.configuration}`, import.meta.url), 'utf8'))
  assert.deepEqual(config.comments, { lineComment: '//' })
  assert.deepEqual(config.brackets, [['{', '}'], ['(', ')']])
  assert.ok(config.autoClosingPairs.some(pair => pair.open === '"' && pair.close === '"'))
  assert.equal(config.comments.blockComment, undefined)
})

for (const example of ['core', 'algorithms', 'iteration', 'actions', 'records', 'variants', 'collections', 'text', 'parser', 'tree', 'state_iteration', 'sequences', 'modules/sequences', 'ast', 'generics', 'generic_data', 'operators']) {
  test(`tokenizes executable ${example}.lobo with declaration and contract scopes`, async () => {
    const source = await readFile(new URL(`../../../examples/${example}.lobo`, import.meta.url), 'utf8')
    const tokens = tokenize(source).flat()
    assert.ok(tokens.some(token => token.scopes.includes('entity.name.namespace.lobo')))
    assert.ok(tokens.some(token => token.scopes.includes('entity.name.function.lobo')))
    // The collection contract moved with its helper; the shared algorithm/wrapper have no integer literal.
    if (example !== 'collections') assert.ok(tokens.some(token => token.scopes.includes('keyword.control.contract.lobo')))
    if (!['sequences', 'modules/sequences'].includes(example)) assert.ok(tokens.some(token => token.scopes.includes('constant.numeric.integer.lobo')))
    assert.ok(tokens.every(token => token.scopes[0] === 'source.lobo'))
    if (example === 'iteration') {
      assert.equal(tokens.filter(token => token.text === 'fold' &&
        token.scopes.includes('keyword.control.loop.lobo')).length, 2)
    }
    if (example === 'actions') {
      assert.equal(tokens.filter(token => token.text === 'actions' &&
        token.scopes.includes('keyword.control.action.lobo')).length, 2)
    }
  })
}

test('conservatively highlights multiline contextual prefixes without reserving fold/in', () => {
  const tokens = tokenize('fold (\n i in 3, a: I64 = 0\n) { a + i }').flat()
  const contextual = tokens.filter(token => ['fold', 'in'].includes(token.text))
  assert.equal(contextual.length, 2)
  assert.ok(contextual.every(token => !token.scopes.some(scope => scope.startsWith('keyword.'))))
  expectScope('fold (in in 2, a: I64 = 0) { a }', 'fold', 'keyword.control.loop.lobo')
})

test('native ABI operations and opaque result types have distinct scopes',()=>{
  expectScope('Native::listen(cap,0)','listen','support.function.native.lobo')
  expectScope('fn inspect(h:Native::Listener)->Native::I64Result { Native::port(h) }','Listener','support.type.lobo')
  expectScope('fn inspect(h:Native::Listener)->Native::I64Result { Native::port(h) }','I64Result','support.type.lobo')
  expectScope('match Native::port(h) { Ok(port)=>port, Err(code)=>code }','match','keyword.control.conditional.lobo')
})

test('native resources retain type scopes inside collections',()=>{
  expectScope('fn count(streams:List<Native::Stream>)->I64 {List::length(streams)}','Stream','support.type.lobo')
})

test('highlights try propagation while retaining ordinary try calls and identifiers', () => {
  expectScope('let cap = try Native::restrict_capability(root,3);','try','keyword.control.propagation.lobo')
  expectScope('try(value)','try','entity.name.function.lobo')
})

test('SQLite native resources and operations use native highlighting',()=>{
  expectScope('let db = try Native::sqlite_open(cap);','sqlite_open','support.function.native.lobo')
  expectScope('fn f(s:Native::Statement)->Native::TextResult {Native::sqlite_column_text(s,0)}','Statement','support.type.lobo')
  expectScope('fn f(s:Native::Statement)->Native::TextResult {Native::sqlite_column_text(s,0)}','TextResult','support.type.lobo')
})
