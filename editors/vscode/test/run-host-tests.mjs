import { spawn } from 'node:child_process'
import { mkdtemp, mkdir, writeFile, readFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'

// Use an installed VS Code, never the user's normal profile. No test dependency
// or editor download is needed; wrap this command in xvfb-run on headless Linux.
const scratch = await mkdtemp(join(tmpdir(), 'lobo-navigation-host-'))
const root = resolve(import.meta.dirname, '..')
const executable = process.env.LOBO_VSCODE_EXECUTABLE
if (!executable) throw new Error('Set LOBO_VSCODE_EXECUTABLE to the VS Code executable, not the code CLI wrapper')
const environment = { ...process.env }
const workspace = join(scratch, 'workspace')
await mkdir(workspace)
await writeFile(join(workspace, 'entry.lobo'), 'module app @id("app"); import make @id("wrap"); function main @id("main")() -> I64 { match (make(5)) { Ok(p) { p.value }, Empty { 0 } } }')
await writeFile(join(workspace, 'shared.lobo'), 'module lib @id("lib"); record Pair @id("pair") { value: I64 } variant Answer @id("answer") { Ok(Pair), Empty } function wrap @id("wrap")(n: I64) -> Answer { Answer::Ok(Pair { value: n }) }')
await writeFile(join(workspace, 'other.lobo'), 'module other @id("other"); record Pair @id("other.pair") { value: Bool } function f @id("other.f")() -> I64 { 42 }')
await writeFile(join(workspace, 'managed.lobo'), 'module managed; fn increment(n:I64)->I64 {n+1} fn answer()->I64 {increment(41)}')
await writeFile(join(workspace, 'managed.lobo.ids.json'), JSON.stringify({format:'lobo-identities/0.1',identities:{module:'managed',increment:'managed.increment',answer:'managed.answer'}}))
await writeFile(join(workspace, 'inference.lobo'), 'module inference; record Item {value:I64} fn identity<T>(x:T)->T {x} fn apply<T,U>(f:Fn(T)->U,x:T)->U {f(x)} fn extract(item:Item)->I64 {item.value} fn main()->I64 {let item=identity(Item {value:42}); apply(extract,item)} identities {module:"inference",Item:"inference.item",identity:"inference.identity",apply:"inference.apply",extract:"inference.extract",main:"inference.main"}')
await writeFile(join(workspace, 'json-auto.lobo'), 'module json_auto; use js {Json,stringify}; record User {name:Text} fn encode(user:User)->Text {js.stringify(json {"name":user.name})} identities {module:"json.auto",js.Json:"lobo.json.value",js.stringify:"lobo.json.stringify",User:"json.auto.user",encode:"json.auto.encode"}')
for (const [name, source] of [['json-library.lobo', 'libraries/json/json.lobo'], ['http-library.lobo', 'libraries/http/http.lobo']]) {
  await writeFile(join(workspace, name), await readFile(resolve(root, '../..', source)))
}
environment.LOBO_TEST_WORKSPACE = workspace
await writeFile(join(workspace, 'constructors.lobo'), 'module constructors; record Box<T> {value:T} variant Maybe<T> {None,Some(T)} variant Other<T> {None,Some(T)} fn create(flag:Bool)->Maybe<Box<I64>> {if flag {Some(Box {value:42})} else {None}} fn main()->I64 {match create(true) {None=>0,Some(box)=>box.value}} identities {module:"constructors",Box:"constructors.box",Maybe:"constructors.maybe",Other:"constructors.other",create:"constructors.create",main:"constructors.main"}')
await writeFile(join(workspace, 'native.lobo'), 'module native_host; record Input {value:I64} fn integer(input:Input)->I64 {Native::roundtrip_i64(input.value)} fn inspect(listener:Native::Listener)->I64 actions {net.inspect} {match Native::port(listener) {Ok(port)=>port,Err(code)=>code}} identities {module:"native.host",Input:"native.host.input",integer:"native.host.integer",inspect:"native.host.inspect"}')
await writeFile(join(workspace, 'try.lobo'), 'module try_host; record Item {key:I64} variant R<T> {Ok(T),Err(Text)} fn get()->R<Item> {Ok(Item {key:42})} fn main()->R<I64> {let value=try get(); Ok(value.key)} identities {module:"try.host",Item:"try.host.item",R:"try.host.result",get:"try.host.get",main:"try.host.main"}')
for (const [name, source] of [
  ['rest-app.lobo','examples/native/rest/app.lobo'],
  ['rest-controller.lobo','examples/native/rest/controller.lobo'],
  ['rest-permissions.lobo','libraries/native/permissions.lobo'],
  ['rest-transport.lobo','libraries/http-server/transport.lobo'],
  ['rest-parser.lobo','libraries/http-server/server.lobo'],
  ['rest-response.lobo','libraries/http/http.lobo'],
  ['rest-json.lobo','libraries/json/json.lobo']
]) {
  await writeFile(join(workspace,name),await readFile(resolve(root,'../..',source)))
  try { await writeFile(join(workspace,name+'.ids.json'),await readFile(resolve(root,'../..',source+'.ids.json'))) }
  catch(error) { if(error.code!=='ENOENT') throw error }
}
delete environment.ELECTRON_RUN_AS_NODE
const child = spawn(executable, [
  `--user-data-dir=${join(scratch, 'profile')}`,
  `--extensions-dir=${join(scratch, 'extensions')}`,
  `--extensionDevelopmentPath=${root}`,
  `--extensionTestsPath=${join(root, 'test', 'host-tests.cjs')}`,
  '--disable-workspace-trust', '--disable-gpu', '--skip-welcome', '--skip-release-notes', workspace
], { env: environment, stdio: ['ignore', 'pipe', 'pipe'], timeout: 45_000, killSignal: 'SIGTERM' })
let output = ''
child.stdout.on('data', chunk => { output += chunk; process.stdout.write(chunk) })
child.stderr.on('data', chunk => { output += chunk; process.stderr.write(chunk) })
child.on('error', error => { console.error(error); process.exitCode = 1 })
child.on('close', code => {
  const complete = output.includes('LOBO_HOST_TESTS_PASSED:')
  if (!complete) console.error('Extension host did not report completed definition tests')
  process.exitCode = code === 0 && complete ? 0 : 1
})
