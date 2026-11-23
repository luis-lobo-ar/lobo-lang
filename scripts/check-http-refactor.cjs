'use strict'
const fs=require('node:fs')
const path=require('node:path')
const crypto=require('node:crypto')
const {maintainedPath}=require('./modernize-source.cjs')
const ROOT=path.resolve(__dirname,'..')
const TRANSPORT='libraries/http-server/transport.lobo'
const RAW=new Map([['examples/native/loopback.lobo','raw_abi_demo'],['libraries/native/native.lobo','native_abi_wrapper']])
const ROLES=new Set(['http_transport','http_protocol','http_application','http_database_application','raw_abi_demo','native_abi_wrapper','native_boundary_example','database_example','hosted_handler','shared_library','value_example'])
function mask(source) {return source.replace(/\/\/[^\r\n]*|"(?:[^"\\]|\\.)*"/g,s=>s.replace(/[^\r\n]/g,' '))}
function operations(source) {
 const masked=mask(source),counts={}
 for(const op of ['listen','accept','close_listener','close_stream']) counts[op]=[...masked.matchAll(new RegExp(`\\bNative\\s*::\\s*${op}\\s*\\(`,'g'))].length
 return {masked,counts}
}
function review(files,roles) {
 const errors=[],records=[]
 for(const file of Object.keys(roles)) if(!Object.hasOwn(files,file)) errors.push(`missing reviewed source: ${file}`)
 for(const [file,source] of Object.entries(files)) {
  const role=Object.hasOwn(roles,file)?roles[file]:undefined
  if(role===undefined) {errors.push(`unreviewed maintained source: ${file}`);continue}
  if(!ROLES.has(role)) {errors.push(`unknown review role: ${file}`);continue}
  const {masked,counts}=operations(source)
  const nativeBoundary=['libraries/native/native.lobo','libraries/native/permissions.lobo'].includes(file)
  const sqlBoundary=file==='libraries/sql/sql.lobo'
  if((/\bNative\s*::\s*restrict_capability\s*\(/.test(masked) && !nativeBoundary) ||
     (/\bNative\s*::\s*sqlite_restrict\s*\(/.test(masked) && !sqlBoundary) ||
     (/\brestrict_mask\s*\(/.test(masked) && !nativeBoundary && !sqlBoundary)) errors.push(`raw permission boundary in maintained consumer: ${file}`)

  if(role==='http_transport' && file!==TRANSPORT) errors.push(`transport policy must live in ${TRANSPORT}: ${file}`)
  if(RAW.has(file)) {
   if(role!==RAW.get(file)) errors.push(`raw boundary role changed: ${file}`)
  } else if(file!==TRANSPORT) {
   if(Object.values(counts).some(n=>n>0)) errors.push(`application-owned or unreviewed HTTP lifecycle: ${file}`)
   if(role==='raw_abi_demo' || role==='native_abi_wrapper') errors.push(`unapproved raw boundary exception: ${file}`)
  }
  if(file===TRANSPORT) {
   if(role!=='http_transport') errors.push(`transport role changed: ${file}`)
   if(counts.accept!==1) errors.push(`accept policy must have one native call site: ${file}`)
   for(const op of ['close_listener','close_stream']) if(counts[op]!==1) errors.push(`${op} policy must have one native call site: ${file}`)
  }
  if(role==='http_application' && !/\bserve\s*\(/.test(masked)) errors.push(`pure application must delegate to serve: ${file}`)
  if(role==='http_database_application' && !/\bserve_with\s*\(/.test(masked)) errors.push(`database application must delegate to serve_with: ${file}`)
  records.push({path:file,role,native_call_sites:counts,sha256:crypto.createHash('sha256').update(source).digest('hex')})
 }
 return {format:'lobo-http-refactor-audit/0.1',status:errors.length?'fail':'pass',reviewed:records.length,errors,files:records}
}
function sources() {
 const files={}
 function walk(relative) {
  for(const entry of fs.readdirSync(path.join(ROOT,relative),{withFileTypes:true})) {
   const name=`${relative}/${entry.name}`
   if(entry.isDirectory()) walk(name)
   else if(maintainedPath(name)) {
    if(!entry.isFile()) throw new Error(`Not a regular maintained source: ${name}`)
    files[name]=fs.readFileSync(path.join(ROOT,name),'utf8')
   }
  }
 }
 for(const dir of ['examples','libraries','.agents/skills/lobo-programming/references']) walk(dir)
 return files
}
if(require.main===module) {
 const args=process.argv.slice(2)
 if(args.length!==2 || args[0]!=='--out') throw new Error('Usage: node scripts/check-http-refactor.cjs --out NEW_JSON_FILE')
 const roles=JSON.parse(fs.readFileSync(path.join(ROOT,'scripts/http-refactor-scope.json'),'utf8')).files
 const result=review(sources(),roles)
 fs.writeFileSync(path.resolve(args[1]),JSON.stringify(result,null,2)+'\n',{flag:'wx'})
 process.stdout.write(JSON.stringify({status:result.status,reviewed:result.reviewed,errors:result.errors})+'\n')
 if(result.errors.length) process.exitCode=1
}
module.exports={review}
