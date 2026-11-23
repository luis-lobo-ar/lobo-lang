'use strict'

const assert = require('node:assert/strict')
const path = require('node:path')
const vscode = require('vscode')

async function expectDefinition(document, offset, expected, targetDocument = document) {
  assert.ok(offset >= 0, 'definition use must exist in its fixture')
  if (expected !== undefined) assert.ok(expected >= 0, 'definition target must exist in its fixture')
  const locations = await vscode.commands.executeCommand(
    'vscode.executeDefinitionProvider', document.uri, document.positionAt(offset)
  )
  if (expected === undefined) return assert.equal(locations.length, 0)
  assert.equal(locations.length, 1)
  const location = locations[0]
  assert.equal((location.targetUri ?? location.uri).toString(), targetDocument.uri.toString())
  const range = location.targetSelectionRange ?? location.range
  assert.equal(targetDocument.offsetAt(range.start), expected)
  if (targetDocument === document) assert.equal(document.getText(range), document.getText().slice(offset, offset + range.end.character - range.start.character))
}

async function waitForDiagnostics(document, predicate) {
  const existing = vscode.languages.getDiagnostics(document.uri)
  if (predicate(existing)) return existing
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => { subscription.dispose(); reject(new Error('Compiler diagnostic did not arrive')) }, 5000)
    const subscription = vscode.languages.onDidChangeDiagnostics(() => {
      const diagnostics = vscode.languages.getDiagnostics(document.uri)
      if (!predicate(diagnostics)) return
      clearTimeout(timeout)
      subscription.dispose()
      resolve(diagnostics)
    })
  })
}

async function run() {
  const uri = vscode.Uri.file(path.resolve(__dirname, '../../../examples/algorithms.lobo'))
  const example = await vscode.workspace.openTextDocument(uri)
  assert.equal(example.languageId, 'lobo')
  // Opening a contributed language triggers activation asynchronously. Await
  // completion before invoking commands, rather than race provider registration.
  const extension = vscode.extensions.getExtension('lobo-lang.lobo-syntax')
  assert.ok(extension)
  await extension.activate()
  const constructors = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(process.env.LOBO_TEST_WORKSPACE, 'constructors.lobo')))
  const constructorText = constructors.getText()
  await expectDefinition(constructors, constructorText.indexOf('Some(Box'), constructorText.indexOf('Some(T)'))
  await expectDefinition(constructors, constructorText.indexOf('else {None') + 6, constructorText.indexOf('None,Some(T)'))
  await expectDefinition(constructors, constructorText.indexOf('box.value') + 4, constructorText.indexOf('value:T'))
  const managed = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(process.env.LOBO_TEST_WORKSPACE, 'managed.lobo')))
  const managedText = managed.getText()
  await expectDefinition(managed, managedText.lastIndexOf('increment'), managedText.indexOf('increment'))
  const inferred = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(process.env.LOBO_TEST_WORKSPACE, 'inference.lobo')))
  const inferredText = inferred.getText()
  await expectDefinition(inferred, inferredText.indexOf('identity(Item'), inferredText.indexOf('identity<T>'))
  await expectDefinition(inferred, inferredText.indexOf('extract,item'), inferredText.indexOf('extract(item:'))
  await expectDefinition(inferred, inferredText.indexOf('item.value') + 5, inferredText.indexOf('value:I64'))
  const metadata = await vscode.workspace.openTextDocument(vscode.Uri.file(`${managed.uri.fsPath}.ids.json`))
  const metadataText = metadata.getText()
  const metadataEdit = new vscode.WorkspaceEdit()
  metadataEdit.replace(metadata.uri, new vscode.Range(metadata.positionAt(0), metadata.positionAt(metadataText.length)), metadataText.replace('"increment":', '"stale":'))
  assert.ok(await vscode.workspace.applyEdit(metadataEdit))
  await expectDefinition(managed, managedText.lastIndexOf('increment'), undefined)
  const metadataRestore = new vscode.WorkspaceEdit()
  metadataRestore.replace(metadata.uri, new vscode.Range(metadata.positionAt(0), metadata.positionAt(metadata.getText().length)), metadataText)
  assert.ok(await vscode.workspace.applyEdit(metadataRestore))
  await expectDefinition(managed, managedText.lastIndexOf('increment'), managedText.indexOf('increment'))
  const compactSource = 'module demo;\nfn value(n: I64) -> I64 { n + 1 }\nfn show(n: I64) -> I64 { value(n) }\nidentities { module: "compact.host", value: "compact.host.value", show: "compact.host.show" }'
  const compact = await vscode.workspace.openTextDocument({ language: 'lobo', content: compactSource })
  await expectDefinition(compact, compactSource.lastIndexOf('value(n)'), compactSource.indexOf('value(n:'))
  await expectDefinition(compact, compactSource.indexOf('n +'), compactSource.indexOf('n: I64'))
  const source = example.getText()
  await expectDefinition(example, source.lastIndexOf('gcd('), source.indexOf('fn gcd(') + 3)
  assert.ok(extension.isActive)

  const operatorSource = 'module m @id("host.ops"); record V @id("host.v") { x:I64 } operator +(V,V)->V = add; function add @id("host.add")(a:V,b:V)->V { V {x:a.x+b.x} } function f @id("host.f")(v:V)->I64 { (v + v).x }'
  const operatorDocument = await vscode.workspace.openTextDocument({ language:'lobo', content:operatorSource })
  const operatorUse = operatorSource.indexOf('v + v') + 2
  const operatorLocations = await vscode.commands.executeCommand('vscode.executeDefinitionProvider', operatorDocument.uri, operatorDocument.positionAt(operatorUse))
  assert.equal(operatorLocations.length, 1)
  assert.equal(operatorDocument.offsetAt((operatorLocations[0].targetSelectionRange ?? operatorLocations[0].range).start), operatorSource.indexOf('add @'))
  await expectDefinition(operatorDocument, operatorSource.indexOf('= add;') + 2, operatorSource.indexOf('add @'))

  await vscode.workspace.getConfiguration('lobo').update('compilerPath',
    path.resolve(__dirname, '../../../target/debug/lobo'), vscode.ConfigurationTarget.Global)
  const sqliteSource='module m; record User {id:I64} fn f(cap:Native::SqliteCapability,user:User)->Native::I64Result actions {db.open,db.close} {let db=try Native::sqlite_open(cap); let closed=try Native::sqlite_close(db); Ok(user.id)} identities {module:"host.sqlite",User:"host.sqlite.user",f:"host.sqlite.f"}'
  const sqliteDocument=await vscode.workspace.openTextDocument({language:'lobo',content:sqliteSource})
  await expectDefinition(sqliteDocument,sqliteSource.indexOf('user.id')+5,sqliteSource.indexOf('id:I64'))
  await expectDefinition(sqliteDocument,sqliteSource.indexOf('sqlite_close(db)')+13,sqliteSource.indexOf('db=try'))
  const sqliteEdit=new vscode.WorkspaceEdit()
  sqliteEdit.replace(sqliteDocument.uri,new vscode.Range(sqliteDocument.positionAt(0),sqliteDocument.positionAt(sqliteSource.length)),sqliteSource.replace('sqlite_close(db)','sqlite_close(1)'))
  assert.ok(await vscode.workspace.applyEdit(sqliteEdit))
  const sqliteDiagnostic=await waitForDiagnostics(sqliteDocument,items=>items.some(item=>item.code==='compile.type.mismatch'))
  assert.ok(sqliteDiagnostic.some(item=>item.code==='compile.type.mismatch'))
  const ast = await vscode.workspace.openTextDocument(vscode.Uri.file(path.resolve(__dirname, '../../../examples/ast.lobo')))
  const astText = ast.getText()
  const generic = await vscode.workspace.openTextDocument(vscode.Uri.file(path.resolve(__dirname, '../../../examples/generics.lobo')))
  const genericText = generic.getText()
  const data = await vscode.workspace.openTextDocument(vscode.Uri.file(path.resolve(__dirname, '../../../examples/generic_data.lobo')))
  const dataText = data.getText()
  await expectDefinition(data, dataText.indexOf('= Box {') + 2, dataText.indexOf('Box<T>'))
  await expectDefinition(data, dataText.indexOf('present.value') + 8, dataText.indexOf('value: T'))
  await expectDefinition(data, dataText.indexOf('(identity,') + 1, dataText.indexOf('identity<T>'))
  const floats = 'module m @id("float.host"); record Box<T> @id("float.box") { value: T } function negative @id("float.neg")(x: F64) -> F64 { -x } function f @id("float.f")() -> F64 { let callback: Fn(F64) -> F64 = &negative; let b = Box::<F64> { value: callback(1.5e-2 + F64::from_i64(2)) }; b.value }'
  const floatDocument = await vscode.workspace.openTextDocument({ language: 'lobo', content: floats })
  await expectDefinition(floatDocument, floats.indexOf('&negative') + 1, floats.indexOf('negative @'))
  await expectDefinition(floatDocument, floats.lastIndexOf('b.value') + 2, floats.indexOf('value: T'))
  await expectDefinition(generic, genericText.indexOf('value: T') + 7, genericText.indexOf('<T>') + 1)
  await expectDefinition(generic, genericText.indexOf('identity(value)'), genericText.indexOf('identity<T>'))
  await expectDefinition(generic, genericText.indexOf('value: Item') + 7, genericText.indexOf('record Item ') + 7)
  await expectDefinition(ast, astText.indexOf('List<Node>') + 5, astText.indexOf('variant Node ') + 8)
  await expectDefinition(ast, astText.indexOf('Num(n)'), astText.indexOf('Num(I64)'))

  const content = '// 🐺\r\nmodule m @id("m");\r\nfunction main @id("f")(n: I64) -> I64 { fold (i in n, total: I64 = 0) { total + i } }'
  const document = await vscode.workspace.openTextDocument({ language: 'lobo', content })
  await expectDefinition(document, content.lastIndexOf('i }'), content.indexOf('i in'))
  await expectDefinition(document, content.lastIndexOf('total +'), content.indexOf('total:'))
  await expectDefinition(document, content.indexOf('in n') + 3, content.indexOf('n:'))
  await expectDefinition(document, content.indexOf('🐺'), undefined)

  const edited = content.replace('total', 'changed').replace('total +', 'changed +')
  const edit = new vscode.WorkspaceEdit()
  edit.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(content.length)), edited)
  assert.ok(await vscode.workspace.applyEdit(edit))
  assert.ok(document.isDirty)
  await expectDefinition(document, edited.lastIndexOf('changed +'), edited.indexOf('changed:'))
  await vscode.workspace.getConfiguration('lobo').update('compilerPath',
    path.resolve(__dirname, '../../../target/debug/lobo'), vscode.ConfigurationTarget.Global)
  const native = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(process.env.LOBO_TEST_WORKSPACE, 'native.lobo')))
  const nativeText = native.getText()
  await expectDefinition(native, nativeText.indexOf('input.value') + 6, nativeText.indexOf('value:I64'))
  await expectDefinition(native, nativeText.indexOf('=>port') + 2, nativeText.indexOf('Ok(port)') + 3)
  await expectDefinition(native, nativeText.indexOf('roundtrip_i64'), undefined)
  const nativeEdit = new vscode.WorkspaceEdit()
  nativeEdit.replace(native.uri, new vscode.Range(native.positionAt(0), native.positionAt(nativeText.length)), nativeText.replace('input.value', 'true'))
  const nativeFailure = waitForDiagnostics(native, items => items.some(item => item.code === 'compile.type.mismatch'))
  assert.ok(await vscode.workspace.applyEdit(nativeEdit))
  const [nativeError] = await nativeFailure
  assert.equal(native.getText(nativeError.range), 'true')
  const nativeRestore = new vscode.WorkspaceEdit()
  nativeRestore.replace(native.uri, new vscode.Range(native.positionAt(0), native.positionAt(native.getText().length)), nativeText)
  const nativeRepaired = waitForDiagnostics(native, items => items.length === 0)
  assert.ok(await vscode.workspace.applyEdit(nativeRestore))
  await nativeRepaired
  const propagation = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(process.env.LOBO_TEST_WORKSPACE, 'try.lobo')))
  const propagationText = propagation.getText()
  await expectDefinition(propagation, propagationText.indexOf('try get')+4, propagationText.indexOf('get()->'))
  await expectDefinition(propagation, propagationText.indexOf('value.key')+6, propagationText.indexOf('key:I64'))
  await expectDefinition(propagation, propagationText.indexOf('try get'), undefined)
  const propagationEdit = new vscode.WorkspaceEdit()
  propagationEdit.replace(propagation.uri, new vscode.Range(propagation.positionAt(0),propagation.positionAt(propagationText.length)),propagationText.replace('try get()','try 42'))
  const propagationFailure = waitForDiagnostics(propagation,items => items.some(item => item.code === 'compile.try.operand'))
  assert.ok(await vscode.workspace.applyEdit(propagationEdit))
  const [propagationError] = await propagationFailure
  assert.equal(propagation.getText(propagationError.range),'try 42')
  const propagationRestore = new vscode.WorkspaceEdit()
  propagationRestore.replace(propagation.uri,new vscode.Range(propagation.positionAt(0),propagation.positionAt(propagation.getText().length)),propagationText)
  const propagationRepaired = waitForDiagnostics(propagation,items => items.length === 0)
  assert.ok(await vscode.workspace.applyEdit(propagationRestore))
  await propagationRepaired
  const constructorEdit = new vscode.WorkspaceEdit()
  constructorEdit.replace(constructors.uri, new vscode.Range(constructors.positionAt(0), constructors.positionAt(constructorText.length)), constructorText.replace('Some(Box {value:42})', 'Some(true)'))
  const constructorFailure = waitForDiagnostics(constructors, items => items.some(item => item.code === 'compile.type.mismatch'))
  assert.ok(await vscode.workspace.applyEdit(constructorEdit))
  const [constructorError] = await constructorFailure
  assert.equal(constructors.getText(constructorError.range), 'Some(true)')
  const constructorRestore = new vscode.WorkspaceEdit()
  constructorRestore.replace(constructors.uri, new vscode.Range(constructors.positionAt(0), constructors.positionAt(constructors.getText().length)), constructorText)
  const constructorRepaired = waitForDiagnostics(constructors, items => items.length === 0)
  assert.ok(await vscode.workspace.applyEdit(constructorRestore))
  await constructorRepaired
  const inferenceEdit = new vscode.WorkspaceEdit()
  inferenceEdit.replace(inferred.uri, new vscode.Range(inferred.positionAt(0), inferred.positionAt(inferredText.length)), inferredText.replace('apply(extract,item)', 'apply(extract,true)'))
  const inferenceFailure = waitForDiagnostics(inferred, items => items.some(item => item.code === 'compile.type.mismatch'))
  assert.ok(await vscode.workspace.applyEdit(inferenceEdit))
  const [inferenceError] = await inferenceFailure
  assert.equal(inferred.getText(inferenceError.range), 'apply(extract,true)')
  const inferenceRestore = new vscode.WorkspaceEdit()
  inferenceRestore.replace(inferred.uri, new vscode.Range(inferred.positionAt(0), inferred.positionAt(inferred.getText().length)), inferredText)
  const inferenceRepaired = waitForDiagnostics(inferred, items => items.length === 0)
  assert.ok(await vscode.workspace.applyEdit(inferenceRestore))
  await inferenceRepaired
  const compactBad = compactSource.replace('n + 1', 'false')
  const compactInvalid = await vscode.workspace.openTextDocument({ language: 'lobo', content: compactBad })
  const [compactError] = await waitForDiagnostics(compactInvalid, items => items.some(item => item.code === 'compile.type.mismatch'))
  assert.equal(compactInvalid.getText(compactError.range), 'false')
  const compactRepaired = waitForDiagnostics(compactInvalid, items => items.length === 0)
  const compactEdit = new vscode.WorkspaceEdit()
  compactEdit.replace(compactInvalid.uri, compactError.range, 'n + 1')
  assert.ok(await vscode.workspace.applyEdit(compactEdit))
  await compactRepaired
  const bad = 'module m @id("m"); function f @id("f")() -> I64 { false }'
  const invalid = await vscode.workspace.openTextDocument({ language: 'lobo', content: bad })
  const [diagnostic] = await waitForDiagnostics(invalid, items => items.some(item => item.code === 'compile.type.mismatch'))
  assert.equal(invalid.getText(diagnostic.range), 'false')
  const updated = waitForDiagnostics(invalid, items => items.some(item => invalid.getText(item.range) === 'true'))
  const correction = new vscode.WorkspaceEdit()
  correction.replace(invalid.uri, diagnostic.range, 'true')
  assert.ok(await vscode.workspace.applyEdit(correction))
  await updated
  assert.ok(invalid.isDirty)
  const actionText = 'module a @id("a"); function main @id("main")() -> I64 { leaf() } function leaf @id("leaf")() -> I64 actions { demo.observe } { 42 }'
  const actionDocument = await vscode.workspace.openTextDocument({ language: 'lobo', content: actionText })
  await expectDefinition(actionDocument, actionText.indexOf('leaf()'), actionText.indexOf('leaf @'))
  const [actionError] = await waitForDiagnostics(actionDocument, items => items.some(item => item.code === 'compile.action.undeclared'))
  assert.equal(actionDocument.getText(actionError.range), 'leaf()')
  const cleared = waitForDiagnostics(actionDocument, items => items.length === 0)
  const declaration = new vscode.WorkspaceEdit()
  const insertion = actionDocument.positionAt(actionText.indexOf(' { leaf()'))
  declaration.insert(actionDocument.uri, insertion, ' actions { demo.observe }')
  assert.ok(await vscode.workspace.applyEdit(declaration))
  await cleared
  assert.ok(actionDocument.isDirty)
  const records = 'module r @id("r"); record Pair @id("pair") { left: I64 } function main @id("main")(p: Pair) -> I64 { p.left }'
  const recordDocument = await vscode.workspace.openTextDocument({ language: 'lobo', content: records })
  await expectDefinition(recordDocument, records.lastIndexOf('Pair'), records.indexOf('Pair @'))
  await expectDefinition(recordDocument, records.lastIndexOf('left'), records.indexOf('left:'))
  const mistake = new vscode.WorkspaceEdit()
  const fieldRange = new vscode.Range(recordDocument.positionAt(records.lastIndexOf('left')), recordDocument.positionAt(records.lastIndexOf('left') + 4))
  const missing = waitForDiagnostics(recordDocument, items => items.some(item => item.code === 'compile.record.field'))
  mistake.replace(recordDocument.uri, fieldRange, 'missing')
  assert.ok(await vscode.workspace.applyEdit(mistake))
  const [fieldError] = await missing
  assert.equal(recordDocument.getText(fieldError.range), 'p.missing')
  const variants = 'module v @id("v"); variant Answer @id("answer") { Ok(I64), Empty } function main @id("main")(a: Answer) -> I64 { match (a) { Ok(n) { n }, Empty { 0 } } }'
  const variantDocument = await vscode.workspace.openTextDocument({ language: 'lobo', content: variants })
  await expectDefinition(variantDocument, variants.lastIndexOf('Answer'), variants.indexOf('Answer @'))
  await expectDefinition(variantDocument, variants.lastIndexOf('Ok('), variants.indexOf('Ok('))
  await expectDefinition(variantDocument, variants.lastIndexOf('n }'), variants.lastIndexOf('n)'))
  const incomplete = new vscode.WorkspaceEdit()
  const caseStart = variants.lastIndexOf('Empty {')
  const caseRange = new vscode.Range(variantDocument.positionAt(caseStart), variantDocument.positionAt(caseStart + 5))
  const matchError = waitForDiagnostics(variantDocument, items => items.some(item => item.code === 'compile.match.invalid'))
  incomplete.replace(variantDocument.uri, caseRange, 'Unknown')
  assert.ok(await vscode.workspace.applyEdit(incomplete))
  const [armError] = await matchError
  assert.equal(variantDocument.getText(armError.range), 'Unknown')
  const lists = 'module l @id("l"); record Row @id("row") { value: I64 } function main @id("main")(xs: List<Row>) -> I64 { List::head(xs).value }'
  const listDocument = await vscode.workspace.openTextDocument({ language: 'lobo', content: lists })
  await expectDefinition(listDocument, lists.lastIndexOf('Row'), lists.indexOf('Row @'))
  await expectDefinition(listDocument, lists.lastIndexOf('value'), lists.indexOf('value:'))
  await expectDefinition(listDocument, lists.indexOf('head('), undefined)
  const invalidList = new vscode.WorkspaceEdit()
  const argument = lists.lastIndexOf('xs)')
  const invalidOperation = waitForDiagnostics(listDocument, items => items.some(item => item.code === 'compile.list.invalid'))
  invalidList.replace(listDocument.uri, new vscode.Range(listDocument.positionAt(argument),listDocument.positionAt(argument + 2)), '1')
  assert.ok(await vscode.workspace.applyEdit(invalidList))
  const [listError] = await invalidOperation
  assert.equal(listDocument.getText(listError.range), 'List::head(1)')
  const textSource = String.raw`module t @id("t"); function frame @id("frame")(name: Text) -> Text { Text::concat("\"🐺",name) }`
  const textDocument = await vscode.workspace.openTextDocument({ language: 'lobo', content: textSource })
  await expectDefinition(textDocument,textSource.lastIndexOf('name)'),textSource.indexOf('name:'))
  await expectDefinition(textDocument,textSource.indexOf('concat('),undefined)
  await expectDefinition(textDocument,textSource.indexOf('🐺'),undefined)
  const textMistake = new vscode.WorkspaceEdit()
  const operandStart = textSource.lastIndexOf('name)')
  const badText = waitForDiagnostics(textDocument,items=>items.some(item=>item.code==='compile.type.mismatch'))
  textMistake.replace(textDocument.uri,new vscode.Range(textDocument.positionAt(operandStart),textDocument.positionAt(operandStart+4)),'true')
  assert.ok(await vscode.workspace.applyEdit(textMistake))
  const [textError] = await badText
  assert.equal(textDocument.getText(textError.range),'true')
  const parserDocument = await vscode.workspace.openTextDocument(vscode.Uri.file(path.resolve(__dirname, '../../../examples/parser.lobo')))
  const parserSource = parserDocument.getText()
  await expectDefinition(parserDocument, parserSource.indexOf('reduce(machine, kind)'), parserSource.indexOf('fn reduce(') + 3)
  await expectDefinition(parserDocument, parserSource.lastIndexOf('finish(scanner,'), parserSource.indexOf('fn finish(') + 3)
  await expectDefinition(parserDocument, parserSource.indexOf('current.operators') + 'current.'.length, parserSource.indexOf('operators: List<Operator>'))
  const treeDocument = await vscode.workspace.openTextDocument(vscode.Uri.file(path.resolve(__dirname, '../../../examples/tree.lobo')))
  const treeSource = treeDocument.getText()
  await expectDefinition(treeDocument, treeSource.indexOf('insert(state.tree,'), treeSource.indexOf('fn insert(') + 3)
  await expectDefinition(treeDocument, treeSource.lastIndexOf('traverse(tree,'), treeSource.indexOf('fn traverse(') + 3)
  await expectDefinition(treeDocument, treeSource.indexOf('node.parent') + 'node.'.length, treeSource.indexOf('parent: I64'))
  const whileSource = 'module w @id("w"); function main @id("main")() -> I64 { while(state: I64 = 0; state < 3) { state + 1 } }'
  const whileDocument = await vscode.workspace.openTextDocument({ language: 'lobo', content: whileSource })
  await expectDefinition(whileDocument, whileSource.indexOf('state <'), whileSource.indexOf('state:'))
  await expectDefinition(whileDocument, whileSource.lastIndexOf('state +'), whileSource.indexOf('state:'))
  const whileMistake = new vscode.WorkspaceEdit()
  const conditionStart = whileSource.indexOf('state <')
  const whileError = waitForDiagnostics(whileDocument, items => items.some(item => item.code === 'compile.type.mismatch'))
  whileMistake.replace(whileDocument.uri, new vscode.Range(whileDocument.positionAt(conditionStart), whileDocument.positionAt(conditionStart + 'state < 3'.length)), '42')
  assert.ok(await vscode.workspace.applyEdit(whileMistake))
  const [conditionError] = await whileError
  assert.equal(whileDocument.getText(conditionError.range), '42')
  const importSource = 'module imports @id("imports"); import reverse @id("lib.reverse"); function main @id("main")(xs: List<I64>) -> List<I64> { reverse(xs) }'
  const importDocument = await vscode.workspace.openTextDocument({ language: 'lobo', content: importSource })
  await expectDefinition(importDocument, importSource.lastIndexOf('reverse('), importSource.indexOf('reverse @'))
  await vscode.workspace.getConfiguration('lobo').update('sourceFiles',
    ['json-auto.lobo', 'json-library.lobo', 'http-library.lobo'], vscode.ConfigurationTarget.Workspace)
  const jsonAuto = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(process.env.LOBO_TEST_WORKSPACE, 'json-auto.lobo')))
  const jsonText = jsonAuto.getText()
  await expectDefinition(jsonAuto, jsonText.indexOf('user.name') + 5, jsonText.indexOf('name:Text'))
  const jsonLibrary = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(process.env.LOBO_TEST_WORKSPACE, 'json-library.lobo')))
  await expectDefinition(jsonAuto, jsonText.indexOf('Json,stringify'), jsonLibrary.getText().indexOf('Json {'), jsonLibrary)
  const badJson = new vscode.WorkspaceEdit()
  const jsonValue = jsonText.indexOf('user.name')
  badJson.replace(jsonAuto.uri, new vscode.Range(jsonAuto.positionAt(jsonValue), jsonAuto.positionAt(jsonValue + 9)), 'true')
  const jsonFailure = waitForDiagnostics(jsonAuto, items => items.some(item => item.code === 'compile.type.mismatch'))
  assert.ok(await vscode.workspace.applyEdit(badJson))
  const [jsonError] = await jsonFailure
  assert.equal(jsonAuto.getText(jsonError.range), 'true')
  const repairJson = new vscode.WorkspaceEdit()
  repairJson.replace(jsonAuto.uri, new vscode.Range(jsonAuto.positionAt(0), jsonAuto.positionAt(jsonAuto.getText().length)), jsonText)
  const jsonRepaired = waitForDiagnostics(jsonAuto, items => items.length === 0)
  assert.ok(await vscode.workspace.applyEdit(repairJson))
  await jsonRepaired
  await vscode.workspace.getConfiguration('lobo').update('sourceFiles',
    ['entry.lobo', 'shared.lobo', 'other.lobo'], vscode.ConfigurationTarget.Workspace)
  const projectDirectory = process.env.LOBO_TEST_WORKSPACE
  assert.ok(projectDirectory)
  const entry = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(projectDirectory, 'entry.lobo')))
  const shared = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(projectDirectory, 'shared.lobo')))
  const entryText = entry.getText()
  const sharedText = shared.getText()
  await expectDefinition(entry, entryText.lastIndexOf('make('), sharedText.indexOf('wrap @'), shared)
  await expectDefinition(entry, entryText.lastIndexOf('value'), sharedText.indexOf('value:'), shared)
  await expectDefinition(entry, entryText.lastIndexOf('Ok(p)'), sharedText.indexOf('Ok(Pair)'), shared)
  const rootVersion = entry.version
  const rename = new vscode.WorkspaceEdit()
  rename.replace(shared.uri, new vscode.Range(shared.positionAt(0), shared.positionAt(sharedText.length)), sharedText.replaceAll('value', 'renamed'))
  const changedDependency = waitForDiagnostics(entry, items => items.some(item => item.code === 'compile.record.field'))
  assert.ok(await vscode.workspace.applyEdit(rename))
  await changedDependency
  assert.equal(entry.version, rootVersion)
  await expectDefinition(entry, entryText.lastIndexOf('value'), undefined)
  const reset = new vscode.WorkspaceEdit()
  reset.replace(shared.uri, new vscode.Range(shared.positionAt(0), shared.positionAt(shared.getText().length)), sharedText.replace('value: n', 'value: true'))
  const dependencyError = waitForDiagnostics(shared, items => items.some(item => item.code === 'compile.type.mismatch'))
  assert.ok(await vscode.workspace.applyEdit(reset))
  const [remoteError] = await dependencyError
  assert.equal(shared.getText(remoteError.range), 'true')
  await waitForDiagnostics(entry, items => items.length === 0)
  const restore = new vscode.WorkspaceEdit()
  restore.replace(shared.uri, new vscode.Range(shared.positionAt(0), shared.positionAt(shared.getText().length)), sharedText)
  const repaired = waitForDiagnostics(shared, items => items.length === 0)
  assert.ok(await vscode.workspace.applyEdit(restore))
  await repaired
  const otherUri = vscode.Uri.file(path.join(projectDirectory, 'other.lobo'))
  const other = 'module other @id("other"); record Pair @id("other.pair") { value: Bool } function f @id("other.f")() -> I64 { false }'
  const external = waitForDiagnostics({ uri: otherUri }, items => items.some(item => item.code === 'compile.type.mismatch'))
  await vscode.workspace.fs.writeFile(otherUri, Buffer.from(other))
  await external
  assert.equal(entry.version, rootVersion)
  assert.equal(vscode.languages.getDiagnostics(entry.uri).length, 0)
  await vscode.workspace.fs.writeFile(otherUri, Buffer.from(other.replace('{ false }', '{ 42 }')))
  const compactEntry = 'module app;\nuse shared { make };\nfn main() -> I64 { match (shared.make(5)) { Ok(p) => p.value, Empty => 0 } }\nidentities { module: "app", main: "main", shared.make: "wrap" }'
  const compactProjectEdit = new vscode.WorkspaceEdit()
  compactProjectEdit.replace(entry.uri, new vscode.Range(entry.positionAt(0), entry.positionAt(entryText.length)), compactEntry)
  assert.ok(await vscode.workspace.applyEdit(compactProjectEdit))
  await expectDefinition(entry, compactEntry.indexOf('shared.make') + 7, sharedText.indexOf('wrap @'), shared)
  await expectDefinition(entry, compactEntry.indexOf('p.value') + 2, sharedText.indexOf('value:'), shared)
  const compactProjectBad = waitForDiagnostics(entry, items => items.some(item => entry.getText(item.range) === 'false'))
  const breakCompact = new vscode.WorkspaceEdit()
  const emptyValue = compactEntry.indexOf('Empty => 0') + 9
  breakCompact.replace(entry.uri, new vscode.Range(entry.positionAt(emptyValue), entry.positionAt(emptyValue + 1)), 'false')
  assert.ok(await vscode.workspace.applyEdit(breakCompact))
  await compactProjectBad
  console.log('LOBO_HOST_TESTS_PASSED: try payload navigation/unsaved diagnostics, native bindings/resource navigation/unsaved diagnostics, automatic JSON helpers/navigation/unsaved diagnostics, activation, compact local/project navigation and mapped unsaved diagnostics, operator symbols/targets, local navigation/squiggles, unsaved values/actions/loops, explicit-project remote function/field/case targets, dependency overlays/errors and external invalidation')
}

module.exports = { run }
