import assert from 'node:assert/strict'
import test from 'node:test'
import { readFile } from 'node:fs/promises'
import { definitionAt } from '../src/navigation.cjs'

function fixture(marked) {
  const spans = new Map()
  let removed = 0
  const source = marked.replace(/«(\w+):([A-Za-z_][A-Za-z0-9_]*)»/g, (match, label, name, offset) => {
    const start = offset - removed
    removed += match.length - name.length
    spans.set(label, { start, end: start + name.length })
    return name
  })
  return { source, spans }
}

function expectTarget(marked, use, declaration) {
  const { source, spans } = fixture(marked)
  const actual = definitionAt(source, spans.get(use).start)
  if (!declaration) return assert.equal(actual, undefined)
  assert.deepEqual(actual, { origin: spans.get(use), target: spans.get(declaration) })
}

test('generic parameters have function scope and explicit calls preserve concrete field navigation', () => {
  const source = `module m @id("m");
    record «record:Pair» @id("pair") { «field:value»: I64 }
    function «fn:identity»<«param:T»> @id("identity")(x: «annotation:T») -> T { x }
    function main @id("main")(p: Pair) -> I64 { «call:identity»::<«arg:Pair»>(p).«use:value» }`
  for (const [use, declaration] of [['annotation','param'],['call','fn'],['arg','record'],['use','field']]) {
    expectTarget(source,use,declaration)
  }
  expectTarget(`module m @id("m"); function f<«param:T»> @id("f")(x: T) -> T { x }
    function g @id("g")(x: «outside:T») -> I64 { 0 }`, 'outside',null)
})

test('generic records and variants substitute nested field and match payload types', () => {
  const source = `module m @id("m");
    record «item:Item» @id("item") { «key:key»: I64 }
    record «box:Box»<«formal:T»> @id("box") { «value:value»: «annotation:T» }
    variant «option:Option»<U> @id("option") { None, «some:Some»(U) }
    function f @id("f")(x: Item) -> I64 {
      let b: Box<Item> = «ctor:Box»::<Item> { value: x };
      let o = «variant:Option»::<Box<Item>>::«case:Some»(b);
      match(o) { None { 0 }, Some(v) { v.«field:value».«use:key» } }
    }`
  for (const [use, declaration] of [['annotation','formal'],['ctor','box'],['variant','option'],['case','some'],['field','value'],['use','key']]) {
    expectTarget(source,use,declaration)
  }
})

test('function references and calls through typed values preserve definition navigation', () => {
  const source = `module m @id("m");
    record Item @id("item") { «field:key»: I64 }
    record Step<T> @id("step") { apply: Fn(T) -> T }
    function «id:identity»<T> @id("identity")(x: T) -> T { x }
    function f @id("f")(item: Item) -> I64 {
      let «binding:callback»: Fn(Item) -> Item = &«ref:identity»::<Item>;
      let s = Step::<Item> { apply: callback };
      «call:callback»(s.apply(item)).«use:key»
    }`
  for (const [use, declaration] of [['ref','id'],['call','binding'],['use','field']]) expectTarget(source,use,declaration)
})

test('F64 literals and operations preserve navigation through generic values and callback types', () => {
  const source = `module m @id("m");
    record «record:Box»<T> @id("box") { «field:value»: T }
    function «fn:negative» @id("negative")(x: F64) -> F64 { -x }
    function f @id("f")() -> F64 {
      let callback: Fn(F64) -> F64 = &«ref:negative»;
      let b = «ctor:Box»::<F64> { value: callback(1.5e-2 + F64::from_i64(2)) };
      b.«use:value»
    }`
  for (const [use,declaration] of [['ref','fn'],['ctor','record'],['use','field']]) expectTarget(source,use,declaration)
})

test('generic nominal annotations split adjacent closing angle and assignment', () => {
  expectTarget('module m @id("m"); record Box<T> @id("box") { «field:value»: T } function f @id("f")() -> I64 { let b:Box<I64>=Box::<I64> { value:7 }; b.«use:value» }','use','field')
})

test('recursive list payloads navigate to their nominal type and nested cases', () => {
  const source = `module m @id("m");
    variant «decl:Node» @id("node") { «num:Num»(I64), Branch(List<«recursive:Node»>) }
    function f @id("f")(x: Node) -> I64 {
      match (x) { Num(n) { n }, Branch(xs) {
        match (List::head(xs)) { «nested:Num»(n) { n }, Branch(ys) { List::length(ys) } }
      } }
    }`
  expectTarget(source, 'recursive', 'decl')
  expectTarget(source, 'nested', 'num')
})

test('explicit import aliases have local definition targets without guessing dependency members', () => {
  expectTarget('module m @id("m"); import «alias:reverse» @id("lib.reverse"); function f @id("f")(xs: List<I64>) -> List<I64> { «use:reverse»(xs) }','use','alias')
  expectTarget('module m @id("m"); import «alias:Pair» @id("lib.pair"); function f @id("f")(p: «use:Pair») -> I64 { 0 }','use','alias')
})

test('while state scope includes condition and body but excludes initializer and following source', () => {
  const marked = `module m @id("m"); function main @id("f")() -> I64 {
    let done = while («decl:state»: I64 = «self:state»; «condition:state» < 3) { «body:state» + 1 };
    «outside:state»
  }`
  expectTarget(marked, 'self', null)
  expectTarget(marked, 'condition', 'decl')
  expectTarget(marked, 'body', 'decl')
  expectTarget(marked, 'outside', null)
})

test('forward and recursive function calls navigate to the declaration', () => {
  const source = `module sample @id("m");
    function main @id("main")() -> I64 { «forward:later»(2) }
    function «declaration:later» @id("later")(n: I64) -> I64 {
      if n == 0 { 0 } else { «recursive:later»(n - 1) }
    }`
  expectTarget(source, 'forward', 'declaration')
  expectTarget(source, 'recursive', 'declaration')
})

test('parameters resolve in contracts and the body, but body locals stay out of contracts', () => {
  const source = `module m @id("m"); function «fn:main» @id("f")(«param:n»: I64) -> I64
    requires { «pre:n» > 0 }
    ensures { «post:n» == «output:result» && «hidden:body» == 1 }
    { let «local:body» = «initial:n»; «final:body» }`
  for (const use of ['pre', 'post', 'initial']) expectTarget(source, use, 'param')
  expectTarget(source, 'output', 'fn')
  expectTarget(source, 'final', 'local')
  expectTarget(source, 'hidden', null)
})

test('a local is visible after its initializer, not within it or before it', () => {
  const source = `module m @id("m"); function main @id("f")() -> I64 {
    let early = «before:value»;
    let «decl:value» = «self:value» + 1;
    «later:value»
  }`
  expectTarget(source, 'before', null)
  expectTarget(source, 'self', null)
  expectTarget(source, 'later', 'decl')
})

test('sibling branches resolve their own same-name binding and do not leak', () => {
  const source = `module m @id("m"); function main @id("f")(flag: Bool) -> I64 {
    let answer = if flag { let «left:x» = 1; «left_use:x» } else { let «right:x» = 2; «right_use:x» };
    answer + «outside:x»
  }`
  expectTarget(source, 'left_use', 'left')
  expectTarget(source, 'right_use', 'right')
  expectTarget(source, 'outside', null)
})

test('fold names are body-only and nested folds can read enclosing bindings', () => {
  const source = `module m @id("m"); function main @id("f")(«param:n»: I64) -> I64 {
    let answer = fold («index:i» in «count:n» + «index_header:i», «acc:total»: I64 = «acc_header:total») {
      fold (j in 2, inner: I64 = «init_use:total») { inner + «outer_use:i» + j }
    };
    answer + «escaped:total»
  }`
  expectTarget(source, 'count', 'param')
  expectTarget(source, 'init_use', 'acc')
  expectTarget(source, 'outer_use', 'index')
  for (const use of ['index_header', 'acc_header', 'escaped']) expectTarget(source, use, null)
})

test('fold, in, and id remain ordinary callable/local names outside contextual syntax', () => {
  const source = `module m @id("m"); function «fn:fold» @id("f")(«param:in»: I64) -> I64 {
    let «local:id» = «param_use:in»;
    «call:fold»(«arg:in») + «local_use:id»
  }`
  expectTarget(source, 'call', 'fn')
  expectTarget(source, 'param_use', 'param')
  expectTarget(source, 'arg', 'param')
  expectTarget(source, 'local_use', 'local')
})

test('proposed actions terminology does not reserve a Core callable name', () => {
  const source = `module m @id("m");
    function «fn:actions» @id("actions")(n: I64) -> I64 { n }
    function main @id("main")() -> I64 { «call:actions»(42) }`
  expectTarget(source, 'call', 'fn')
})

test('record types, constructors, labels and selections navigate without name guessing', () => {
  const source = `module m @id("m");
    record «record:Pair» @id("pair") { «field:left»: I64, right: I64 }
    function make @id("make")() -> «type:Pair» { «constructor:Pair» { «label:left»: 1, right: 2 } }
    function main @id("main")(«param:p»: Pair) -> I64 {
      let q = «use:p»; q.«select:left» + make().«returned:left»
    }`
  for (const use of ['type', 'constructor']) expectTarget(source, use, 'record')
  for (const use of ['label', 'select', 'returned']) expectTarget(source, use, 'field')
  expectTarget(source, 'use', 'param')
})

test('forward nested record types and record fold state preserve definition targets', () => {
  const source = `module m @id("m");
    record Outer @id("outer") { inner: «type:Pair» }
    function main @id("main")(p: Outer) -> I64 {
      let q = fold(i in 1, state: Outer = p) { state }; q.inner.«selected:left»
    }
    record «pair:Pair» @id("pair") { «field:left»: I64 }`
  expectTarget(source, 'type', 'pair')
  expectTarget(source, 'selected', 'field')
})

test('record remains an ordinary callable name outside a declaration', () => {
  expectTarget('module m @id("m"); function «fn:record» @id("r")() -> I64 { 1 } function main @id("f")() -> I64 { «call:record»() }', 'call', 'fn')
})

test('variant types, constructors, cases, payload fields and arm bindings navigate', () => {
  const source = `module m @id("m");
    function main @id("main")(a: «type:Answer») -> I64 {
      let x = «ctor_type:Answer»::«ctor_case:Ok»(Pair { value: 42 });
      match (x) { «empty_case:Empty» { 0 }, «arm_case:Ok»(«binding:p») { «bound_use:p».«field_use:value» } }
    }
    variant «variant:Answer» @id("answer") { «case:Ok»(Pair), «empty:Empty» }
    record Pair @id("pair") { «field:value»: I64 }`
  for (const use of ['type', 'ctor_type']) expectTarget(source, use, 'variant')
  for (const use of ['ctor_case', 'arm_case']) expectTarget(source, use, 'case')
  expectTarget(source, 'empty_case', 'empty')
  expectTarget(source, 'bound_use', 'binding')
  expectTarget(source, 'field_use', 'field')
})

test('nested matches and sibling payload bindings keep distinct scopes', () => {
  const source = `module m @id("m");
    variant Inner @id("inner") { A(I64), B(I64) }
    variant Outer @id("outer") { Item(Inner), Empty }
    function main @id("f")(o: Outer) -> I64 {
      let x = match (o) { Item(«outer:v») { match («outer_use:v») { A(«a:n») { «a_use:n» }, B(«b:n») { «b_use:n» } } }, Empty { 0 } };
      x + «escaped:n»
    }`
  expectTarget(source, 'outer_use', 'outer')
  expectTarget(source, 'a_use', 'a')
  expectTarget(source, 'b_use', 'b')
  expectTarget(source, 'escaped', null)
})

test('ordinary match and variant names remain callable; incomplete matches fail closed', () => {
  expectTarget('module m @id("m"); function «fn:match» @id("f")(n: I64) -> I64 { n } function main @id("main")() -> I64 { «call:match»(1) }', 'call', 'fn')
  expectTarget('module m @id("m"); function «fn:variant» @id("f")() -> I64 { 1 } function main @id("main")() -> I64 { «call:variant»() }', 'call', 'fn')
  const source = 'module m @id("m"); variant V @id("v") { A(I64), B } function f @id("f")(v: V) -> I64 { match (v) { A(n) { n } } }'
  assert.equal(definitionAt(source, source.lastIndexOf('n')), undefined)
})

test('nested List types and head/tail preserve nominal field navigation without intrinsic guesses', () => {
  const source = `module m @id("m");
    record «record:Row» @id("row") { «field:value»: I64 }
    function make @id("make")(x: List<«type:Row»>) -> List<Row> { x }
    function main @id("main")(«xs:xs»: List<List<Row>>) -> I64 {
      let tail: List<List<Row>>=List::tail(«tail_use:xs»);
      let row = List::head(make(List::head(xs)));
      row.«selected:value» + List::«intrinsic:length»(tail)
    }
    function «user:length» @id("length")() -> I64 { 42 }`
  expectTarget(source, 'type', 'record')
  expectTarget(source, 'tail_use', 'xs')
  expectTarget(source, 'selected', 'field')
  expectTarget(source, 'intrinsic', null)
})

test('empty list type arguments navigate and unsupported list operations fail closed', () => {
  const source = 'module m @id("m"); record «row:Row» @id("row") { value: I64 } function f @id("f")() -> List<Row> { List::empty<«type:Row»>() }'
  expectTarget(source,'type','row')
  const bad = 'module m @id("m"); function f @id("f")(xs: List<I64>) -> I64 { List::unknown(xs) }'
  assert.equal(definitionAt(bad,bad.lastIndexOf('xs')),undefined)
})

test('text escapes and buffer intrinsics preserve scopes without string-content references', () => {
  const source = String.raw`module m @id("m");
    function «fn:frame» @id("frame")(«param:name»: Text) -> Text { Text::concat("\"«inside:frame» //", «use:name») }
    function length @id("length")() -> I64 { 1 }
    function main @id("main")() -> I64 { Text::«builtin:length»(«call:frame»("🐺")) }`
  expectTarget(source,'use','param')
  expectTarget(source,'call','fn')
  expectTarget(source,'inside',null)
  expectTarget(source,'builtin',null)
})

test('malformed JSON escapes and unpaired surrogates fail closed', () => {
  for (const text of [String.raw`"\q"`,String.raw`"\uD800"`,String.raw`"\uDC00"`]) {
    const source=`module m @id("m"); function f @id("f")(x: I64) -> I64 { let t = ${text}; x }`
    assert.equal(definitionAt(source,source.lastIndexOf('x')),undefined)
  }
})

test('action metadata is not a reference and does not break body navigation', () => {
  const source = `module m @id("m");
    function «fn:leaf» @id("leaf")(«param:n»: I64) -> I64
      actions // comment
      { demo . «metadata:leaf», demo.extra }
      requires { «pre:n» >= 0 }
      { «body:n» }
    function main @id("main")() -> I64 actions { demo.leaf } { «call:leaf»(42) }`
  expectTarget(source, 'pre', 'param')
  expectTarget(source, 'body', 'param')
  expectTarget(source, 'call', 'fn')
  expectTarget(source, 'metadata', null)
  const broken = fixture(source.replace('demo.extra', 'demo.')).source
  assert.equal(definitionAt(broken, broken.lastIndexOf('leaf(')), undefined)
})

test('multiline/comment-separated fold prefixes navigate independently of lexical highlighting', () => {
  const source = `module m @id("m"); function main @id("f")() -> I64 {
    fold // comment
    ( «idx:in» // comment
      in 2, «acc:total»: I64 = 0) { «use:in» + «acc_use:total» }
  }`
  expectTarget(source, 'use', 'idx')
  expectTarget(source, 'acc_use', 'acc')
})

test('comments, identity strings, keywords, types, and punctuation are not references', () => {
  const source = `module m @id("m"); // «comment:main»
    function «fn:main» @id("«identity:main»")() -> «type:I64» { if true { 1 } else { 0 } }`
  for (const use of ['comment', 'identity', 'type']) expectTarget(source, use, null)
  const clean = fixture(source).source
  for (const token of ['module', 'function', 'if', 'true', '@', '(', '{']) {
    assert.equal(definitionAt(clean, clean.indexOf(token)), undefined, token)
  }
})

test('duplicate declarations and malformed/incomplete sources fail closed', () => {
  const duplicate = 'module m @id("m"); function f @id("f")() -> I64 { f() } function f @id("g")() -> I64 { 1 }'
  assert.equal(definitionAt(duplicate, duplicate.indexOf('f()')), undefined)
  for (const source of ['function f(', 'module m @id("m"); function f @id("f")() -> I64 { f(',
    'module m @id("m"); function f @id("f")() -> I64 { let a = 1; let a = 2; a }',
    'module m @id("m"); function f @id("f")() -> I64 { missing.member }']) {
    assert.equal(definitionAt(source, source.lastIndexOf('f')), undefined)
  }
})

test('ranges use UTF-16 offsets through emoji comments and CRLF', () => {
  const source = '// 🐺\r\nmodule m @id("m");\r\nfunction main @id("f")(«param:n»: I64) -> I64 { «use:n» }'
  expectTarget(source, 'use', 'param')
})

test('each lookup uses new unsaved text, not a stale definition index', () => {
  expectTarget('module m @id("m"); function main @id("f")(«p:n»: I64) -> I64 { «u:n» }', 'u', 'p')
  expectTarget('module m @id("m"); function main @id("f")(n: I64) -> I64 { let «p:x» = n; «u:x» }', 'u', 'p')
})

test('Core comparison/equality chains fail closed while separate boolean operands resolve', () => {
  for (const expression of ['a < 1 < 2', 'a == 1 == true']) {
    const source = `module m @id("m"); function f @id("f")(a: I64) -> Bool { ${expression} }`
    assert.equal(definitionAt(source, source.lastIndexOf('a')), undefined)
  }
  expectTarget('module m @id("m"); function f @id("f")(«p:a»: I64) -> Bool { «u:a» < 0 || a > 1 }', 'u', 'p')
})

test('size, token, nesting and invalid-offset budgets return no result without crashing', () => {
  assert.equal(definitionAt(' '.repeat(1_048_577), 0), undefined)
  assert.equal(definitionAt('a '.repeat(8193), 0), undefined)
  const source = `module m @id("m"); function main @id("f")(n: I64) -> I64 { ${'('.repeat(40)}n${')'.repeat(40)} }`
  assert.equal(definitionAt(source, source.lastIndexOf('n')), undefined)
  for (const offset of [-1, NaN, 0.5, 9000]) assert.equal(definitionAt('n', offset), undefined)
})

for (const example of ['core', 'algorithms', 'iteration', 'actions', 'records', 'variants', 'collections', 'text', 'parser', 'tree', 'state_iteration', 'sequences', 'modules/sequences']) {
  test(`resolves executable ${example}.lobo parameters without changing compiler behavior`, async () => {
    const source = await readFile(new URL(`../../../examples/${example}.lobo`, import.meta.url), 'utf8')
    const parameter = /\(([A-Za-z_][A-Za-z0-9_]*): [A-Za-z_][A-Za-z0-9_]*/.exec(source)
    assert.ok(parameter)
    const name = parameter[1]
    const declaration = parameter.index + 1
    const start = source.indexOf('{', parameter.index)
    const reference = new RegExp(`\\b${name}\\b`, 'g')
    reference.lastIndex = start
    const use = reference.exec(source)?.index
    assert.ok(use > declaration)
    assert.deepEqual(definitionAt(source, use).target, { start: declaration, end: declaration + name.length })
  })
}

test('parser workload resolves composed functions and fields in its full source', async () => {
  const source = await readFile(new URL('../../../examples/parser.lobo', import.meta.url), 'utf8')
  for (const [use, declaration] of [
    [source.indexOf('reduce(machine, kind)'), source.indexOf('fn reduce(') + 3],
    [source.lastIndexOf('finish(scanner,'), source.indexOf('fn finish(') + 3],
    [source.indexOf('current.operators') + 'current.'.length, source.indexOf('operators: List<Operator>')]
  ]) {
    const definition = definitionAt(source, use)
    assert.equal(definition?.target.start, declaration)
  }
})

test('tree workload resolves composed search functions and parent-index fields', async () => {
  const source = await readFile(new URL('../../../examples/tree.lobo', import.meta.url), 'utf8')
  for (const [use, declaration] of [
    [source.indexOf('insert(state.tree,'), source.indexOf('fn insert(') + 3],
    [source.lastIndexOf('traverse(tree,'), source.indexOf('fn traverse(') + 3],
    [source.indexOf('node.parent') + 'node.'.length, source.indexOf('parent: I64')]
  ]) {
    const definition = definitionAt(source, use)
    assert.equal(definition?.target.start, declaration)
  }
})

test('try yields generic and native success payloads for definition navigation', () => {
  const source = `module m;
    record Item { «field:key»:I64 }
    variant Outcome<T> { Ok(T),Err(Text) }
    fn «fetch:get»()->Outcome<Item> {Ok(Item {key:42})}
    fn main()->Outcome<I64> {let «binding:value»=try «call:get»(); Ok(«local:value».«use:key»)}
    identities {module:"m",Item:"item",Outcome:"outcome",get:"get",main:"main"}`
  for (const [use,declaration] of [['call','fetch'],['local','binding'],['use','field']]) expectTarget(source,use,declaration)
})
