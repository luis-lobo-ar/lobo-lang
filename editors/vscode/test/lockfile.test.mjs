import assert from 'node:assert/strict'
import {readFile} from 'node:fs/promises'
import test from 'node:test'

test('extension version bumps retain a lockfile matching the installed development tools',async()=>{
 const manifest=JSON.parse(await readFile(new URL('../package.json',import.meta.url),'utf8'))
 const lock=JSON.parse(await readFile(new URL('../package-lock.json',import.meta.url),'utf8'))
 assert.equal(lock.version,manifest.version)
 assert.equal(lock.packages[''].version,manifest.version)
 assert.deepEqual(lock.packages[''].devDependencies,manifest.devDependencies)
 for(const [location,entry] of Object.entries(lock.packages)) {
  if(!location) continue
  let installed
  try {installed=JSON.parse(await readFile(new URL(`../${location}/package.json`,import.meta.url),'utf8'))}
  catch(error) {if(error.code==='ENOENT' && (entry.optional || entry.devOptional)) continue; throw error}
  assert.equal(entry.version,installed.version,`${location}: use the pinned installed version or explicitly update the dependency`)
  for(const field of ['dependencies','optionalDependencies','peerDependencies']) {
   assert.deepEqual(entry[field]??{},installed[field]??{},`${location}: ${field} must match the installed package metadata`)
  }
 }
})
