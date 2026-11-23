import assert from 'node:assert/strict'
import test from 'node:test'
import {createRequire} from 'node:module'
const require=createRequire(import.meta.url)
const {review}=require('./check-http-refactor.cjs')
const engine='libraries/http-server/transport.lobo'
const app='examples/native/rest/app.lobo'
const roles={[engine]:'http_transport',[app]:'http_application'}
const valid={[engine]:'fn next() {Native::accept(listener,2000); Native::close_listener(listener); Native::close_stream(stream)}',[app]:'fn main() {serve(cap,port,count,handle)}'}
test('review accepts one shared accept policy and a delegated app',()=>{
 assert.deepEqual(review(valid,roles).errors,[])
})
test('review rejects application-owned listeners and duplicate accept policies',()=>{
 const failed=review({...valid,[app]:'fn main() {Native::listen(cap,port); serve(cap,port,count,handle)}',[engine]:valid[engine]+valid[engine]},roles)
 assert.ok(failed.errors.some(e=>e.includes('application-owned')))
 assert.ok(failed.errors.some(e=>e.includes('accept policy')))
})
test('new maintained files require a recorded review role',()=>{
 assert.ok(review({...valid,'examples/new.lobo':'fn main() {42}'},roles).errors.some(e=>e.includes('unreviewed')))
 assert.ok(review(valid,{...roles,'examples/deleted.lobo':'value_example'}).errors.some(e=>e.includes('missing')))
})
test('comments and literals do not create false IO findings',()=>{
 const source=valid[app]+' // Native::listen(cap,port)\n let text="Native::accept(listener,1)";'
 assert.deepEqual(review({...valid,[app]:source},roles).errors,[])
})
test('only the explicit low-level demonstrations can keep raw server operations',()=>{
 assert.deepEqual(review({...valid,'examples/native/loopback.lobo':'Native::listen(cap,0)'},{...roles,'examples/native/loopback.lobo':'raw_abi_demo'}).errors,[])
 assert.ok(review({...valid,[app]:'Native::listen(cap,0)'},{...roles,[app]:'raw_abi_demo'}).errors.length>0)
})

test('typed permission audit rejects raw delegation in maintained consumers',()=>{
 for(const call of ['Native::restrict_capability(root,5)','Native::sqlite_restrict(root,1)','restrict_mask(root,5)']) {
  const result=review({'examples/core.lobo':`module app; fn bad() {${call}}`},{'examples/core.lobo':'value_example'})
  assert.equal(result.status,'fail',call)
  assert.ok(result.errors.some(e=>e.includes('raw permission boundary')),result.errors.join('\n'))
 }
})
