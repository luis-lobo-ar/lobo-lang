'use strict'
const fs=require('node:fs')
const path=require('node:path')
const os=require('node:os')
const crypto=require('node:crypto')
const {spawnSync}=require('node:child_process')
const ROOT=path.resolve(__dirname,'..')
const sha=text=>crypto.createHash('sha256').update(text).digest('hex')
function maintainedPath(file) {
  return typeof file==='string' && /^(?:examples|libraries|\.agents\/skills\/lobo-programming\/references)\/.+\.lobo$/.test(file) &&
    !file.includes('\\') && !file.split('/').some(p=>!p || p==='.' || p==='..') &&
    !file.startsWith('examples/v1/') && !['examples/mvc.lobo','examples/transfer.lobo'].includes(file)
}
function lex(source) {
  const re=/\s+|\/\/[^\r\n]*|"(?:[^"\\\r\n]|\\[^\r\n])*"|[A-Za-z_][A-Za-z_0-9]*|[0-9]+(?:\.[0-9]+)?(?:[eE][+\-]?[0-9]+)?|\.\.\.|::|->|=>|==|!=|<=|>=|&&|\|\||[@();,:{}.+*/%<>=!&\-]/gy
  const t=[]
  while(re.lastIndex<source.length) {
    const start=re.lastIndex,m=re.exec(source)
    if(!m) throw new SyntaxError(`Unsupported token at ${start}`)
    if(!/^\s|^\/\//.test(m[0])) t.push({text:m[0],start,end:re.lastIndex})
  }
  return t
}
function pairs(t) {
  const out=new Map(),stack=[]
  for(const [i,v] of t.entries()) {
    if(['(','{'].includes(v.text)) stack.push(i)
    if([')','}'].includes(v.text)) {
      const open=stack.pop()
      if(open===undefined || (t[open].text==='('?')':'}')!==v.text) throw new SyntaxError('Unbalanced source')
      out.set(open,i)
    }
  }
  if(stack.length) throw new SyntaxError('Unclosed source')
  return out
}
function angle(t,start) {
  let depth=0
  for(let i=start;i<t.length;i++) {
    if(t[i].text==='<') depth++
    if(t[i].text==='>' && --depth===0) return i
  }
  return undefined
}
function erase(source,t,first,last) {
  let text=source
  for(let i=last;i>=first;i--) text=text.slice(0,t[i].start)+text.slice(t[i].end)
  return text
}
function candidates(source) {
  const t=lex(source),p=pairs(t),out=[]
  const add=(kind,text,at,removeIdentity)=>{if(text!==source) out.push({kind,text,at,...(removeIdentity?{removeIdentity}:{})})}
  for(let i=0;i<t.length;i++) {
    if(t[i].text==='&' && /^[A-Za-z_]/.test(t[i+1]?.text??'')) {
      const end=t[i+2]?.text==='::' && t[i+3]?.text==='<'?angle(t,i+3):i+1
      if(end!==undefined) {
        let text=erase(source,t,i+2,end)
        text=text.slice(0,t[i].start)+text.slice(t[i].end)
        add('callback',text,t[i].start)
      }
    }
    if(t[i].text==='::' && t[i+1]?.text==='<' && !(t[i-2]?.text==='::' && ['List','Bytes','Text','F64','Native'].includes(t[i-3]?.text))) {
      const end=angle(t,i+1)
      if(end!==undefined) add('type-arguments',erase(source,t,i,end),t[i].start)
    }
    if(/^[A-Z]/.test(t[i].text) && !['List','Bytes','Text','F64','Native'].includes(t[i].text) && t[i+1]?.text==='::') {
      let end=i+1
      if(t[i+2]?.text==='<') {const close=angle(t,i+2);if(close===undefined) continue;end=close+1}
      if(t[end]?.text==='::' && /^[A-Z]/.test(t[end+1]?.text??'')) add('constructor-owner',erase(source,t,i,end),t[i].start)
    }
    if(t[i].text==='Text' && t[i+1]?.text==='::' && t[i+2]?.text==='concat' && t[i+3]?.text==='(') {
      const close=p.get(i+3)
      let comma,level=0
      for(let n=i+4;n<close;n++) {
        if(['(','{'].includes(t[n].text)) level++
        if([')','}'].includes(t[n].text)) level--
        if(t[n].text===',' && level===0) {comma=n;break}
      }
      if(comma!==undefined) {
        const left=source.slice(t[i+3].end,t[comma].start),right=source.slice(t[comma].end,t[close].start)
        add('text-concat',source.slice(0,t[i].start)+`(${left} + (${right}))`+source.slice(t[close].end),t[i].start)
      }
    }
    if(t[i].text==='use' && t[i+2]?.text==='{') {
      const end=p.get(i+2)
      for(let n=i+3;n<end;n++) if(['object','field'].includes(t[n].text)) {
        let first=n,last=n
        if(t[n+1]?.text===',') last++
        else if(t[n-1]?.text===',') first--
        add('json-builder-import',erase(source,t,first,last),t[n].start,`${t[i+1].text}.${t[n].text}`)
      }
    }
  }
  for(const [open,close] of p) {
    if(t[open].text==='{' && t[open-1]?.text==='=>') {
      let text=source.slice(0,t[close].start)+source.slice(t[close].end)
      text=text.slice(0,t[open].start)+text.slice(t[open].end)
      add('arm-block',text,t[open].start)
    }
    if(t[open].text==='(' && ['{','=',',','=>','(','+','-','*','/','%','&&','||','!','match'].includes(t[open-1]?.text)) {
      let text=source.slice(0,t[close].start)+source.slice(t[close].end)
      text=text.slice(0,t[open].start)+(t[open-1]?.text==='match'?' ':'')+text.slice(t[open].end)
      add('group',text,t[open].start)
    }
  }
  const order={'callback':0,'type-arguments':1,'constructor-owner':2,'text-concat':3,'json-builder-import':4,'arm-block':5,'group':6}
  return out.sort((a,b)=>order[a.kind]-order[b.kind] || a.at-b.at)
}
function tidy(source) {
  const original=lex(source),edits=[]
  for(let i=0;i+1<original.length;i++) {
    const left=original[i],right=original[i+1],gap=source.slice(left.end,right.start)
    if(!/^\s*$/.test(gap)) continue
    let replacement=gap
    if(left.text==='=>') replacement=' '
    else if(!gap.includes('\n') && !gap.includes('\r')) {
      if([',',';',')'].includes(right.text) || left.text==='(') replacement=''
      else if(gap.length>1) replacement=' '
    } else if(right.text==='}' && /\n[ \t]*\n/.test(gap)) replacement='\n'
    if(replacement!==gap) edits.push({start:left.end,end:right.start,text:replacement})
  }
  for(const e of edits.reverse()) source=source.slice(0,e.start)+e.text+source.slice(e.end)
  const stack=[]
  const lines=source.split('\n').map(line=>{
    const content=line.trim()
    if(!content) return ''
    const tokens=lex(content),leading=[...stack]
    for(const token of tokens) {
      if(['}',')'].includes(token.text)) leading.pop()
      else break
    }
    let braces=0,continuation=0
    for(const symbol of leading) {if(symbol==='{') {braces++;continuation=0} else continuation++}
    const result=' '.repeat(2*(braces+continuation))+content
    for(const token of tokens) {
      if(['{','('].includes(token.text)) stack.push(token.text)
      else if(['}',')'].includes(token.text)) stack.pop()
    }
    return result
  }).join('\n')
  if(JSON.stringify(lex(lines).map(t=>t.text))!==JSON.stringify(original.map(t=>t.text))) throw new Error('Layout changed lexical tokens')
  return lines
}
function identityMap(source,sidecar) {
  if(sidecar) return sidecar.identities
  const t=lex(source),start=t.findIndex((v,i)=>v.text==='identities' && t[i+1]?.text==='{'),ids={}
  if(start<0) throw new Error('Missing explicit identities')
  for(let i=start+2;t[i] && t[i].text!=='}';) {
    let name=''
    while(t[i].text!==':') name+=t[i++].text
    ids[name]=JSON.parse(t[++i].text);i++
    if(t[i]?.text===',') i++
  }
  return ids
}
function fingerprint(program) {
  const value=structuredClone(program)
  for(const scope of value.scopes) scope.imports=[...new Set(scope.imports.map(v=>v.id))].sort()
  return sha(JSON.stringify(value))
}
function registry() {
  const files=[]
  function walk(dir) {
    for(const p of fs.readdirSync(path.join(ROOT,dir),{withFileTypes:true})) {
      const n=`${dir}/${p.name}`
      if(p.isDirectory()) walk(n)
      else if(maintainedPath(n)) {if(!p.isFile()) throw new Error(`Not a regular maintained file: ${n}`);files.push(n)}
    }
  }
  for(const d of ['examples','libraries','.agents/skills/lobo-programming/references']) walk(d)
  return new Map(files.sort().map(name=>{
    const text=fs.readFileSync(path.join(ROOT,name),'utf8'),sidecar=fs.existsSync(path.join(ROOT,name+'.ids.json'))?JSON.parse(fs.readFileSync(path.join(ROOT,name+'.ids.json'),'utf8')):undefined
    return [name,{text,sidecar,ids:identityMap(text,sidecar)}]
  }))
}
function run({compiler,out,apply=false}) {
  const all=registry(),owners=new Map()
  for(const [name,file] of all) for(const [key,id] of Object.entries(file.ids)) if(!key.includes('.') && key!=='module') {
    if(owners.has(id)) throw new Error(`Duplicate maintained declaration ${id}`)
    owners.set(id,name)
  }
  const tmp=fs.mkdtempSync(path.join(os.tmpdir(),'lobo-modernize-'))
  for(const [name,file] of all) {
    const dest=path.join(tmp,name);fs.mkdirSync(path.dirname(dest),{recursive:true});fs.writeFileSync(dest,file.text)
    if(file.sidecar) fs.writeFileSync(dest+'.ids.json',JSON.stringify(file.sidecar))
  }
  fs.mkdirSync(out,{recursive:true});let calls=0
  function closure(root) {
    const seen=new Set()
    function visit(name) {
      if(seen.has(name)) return
      seen.add(name)
      for(const [key,id] of Object.entries(all.get(name).ids)) if(key.includes('.')) {
        const owner=owners.get(id)
        if(!owner) throw new Error(`No maintained owner for ${name}: ${id}`)
        visit(owner)
      }
    }
    visit(root);return [root,...[...seen].filter(n=>n!==root).sort()]
  }
  function compile(name,text,sidecar) {
    const dest=path.join(tmp,name),object=path.join(tmp,'candidate.loboc')
    fs.writeFileSync(dest,text)
    if(sidecar) fs.writeFileSync(dest+'.ids.json',JSON.stringify(sidecar))
    fs.rmSync(object,{force:true});calls++
    const sources=closure(name),args=['compile',dest,...sources.slice(1).flatMap(s=>['--source',path.join(tmp,s)]),'--emit','checked','-o',object]
    const p=spawnSync(compiler,args,{cwd:ROOT,encoding:'utf8',timeout:30000,maxBuffer:8*1024*1024})
    if(p.error || p.signal) throw new Error(`Compiler infrastructure failed: ${p.error??p.signal}`)
    if(p.status!==0) return {error:JSON.parse(p.stdout).diagnostic}
    const value=JSON.parse(fs.readFileSync(object,'utf8'))
    return {hash:fingerprint(value.program),program:value.program}
  }
  const report={format:'lobo-modernize-source/0.1',language:'core/1.11',compiler_sha256:sha(fs.readFileSync(compiler)),files:[],compiler_calls:0}
  try {
    for(const [name,file] of all) {
      let source=file.text,sidecar=file.sidecar
      const before=compile(name,source,sidecar)
      if(before.error) throw new Error(`${name}: baseline invalid ${JSON.stringify(before.error)}`)
      const accepted=[],rejected=[]
      for(;;) {
        let changed=false
        for(const candidate of candidates(source)) {
          let metadata=sidecar && structuredClone(sidecar)
          if(candidate.removeIdentity) {
            if(!metadata) continue
            delete metadata.identities[candidate.removeIdentity]
          }
          const result=compile(name,candidate.text,metadata)
          if(!result.error && result.hash===before.hash) {
            accepted.push({kind:candidate.kind,at:candidate.at,...(candidate.removeIdentity?{removed_import:candidate.removeIdentity}:{})})
            source=candidate.text;sidecar=metadata;changed=true;break
          }
          rejected.push({kind:candidate.kind,at:candidate.at,reason:result.error?.code??'checked-program-differs'})
        }
        if(!changed) break
      }
      const cleaned=tidy(source)
      if(cleaned!==source) {accepted.push({kind:'layout'});source=cleaned}
      // Restore the accepted source after the last rejected probe.
      const after=compile(name,source,sidecar)
      if(after.error || after.hash!==before.hash) throw new Error(`Semantic mismatch ${name}`)
      report.files.push({path:name,before_sha256:sha(file.text),after_sha256:sha(source),checked_fingerprint:after.hash,identities_unchanged:Object.entries(identityMap(source,sidecar)).filter(([k])=>!k.includes('.')).every(([k,v])=>file.ids[k]===v),accepted,rejected})
      if(apply && source!==file.text) {
        if(sha(fs.readFileSync(path.join(ROOT,name)))!==sha(file.text)) throw new Error(`Concurrent source edit ${name}`)
        fs.writeFileSync(path.join(ROOT,name),source)
        if(sidecar) fs.writeFileSync(path.join(ROOT,name+'.ids.json'),JSON.stringify(sidecar,null,2)+'\n')
      }
      all.set(name,{text:source,sidecar,ids:identityMap(source,sidecar)})
    }
    report.compiler_calls=calls;report.status='pass'
  } finally {fs.writeFileSync(path.join(out,'results.json'),JSON.stringify(report,null,2)+'\n');fs.rmSync(tmp,{recursive:true,force:true})}
  return {status:'pass',files:report.files.length,changed:report.files.filter(f=>f.accepted.length).length,rewrites:report.files.reduce((n,f)=>n+f.accepted.length,0),compiler_calls:calls}
}
module.exports={candidates,maintainedPath,fingerprint,tidy}
if(require.main===module) {
  try {
    const args=process.argv.slice(2),mode=args.shift()
    if(!['--audit','--apply'].includes(mode)) throw new Error('Use --audit or --apply')
    const apply=mode==='--apply'
    const options={apply,compiler:path.join(ROOT,'target/release/lobo')}
    for(let i=0;i<args.length;i+=2) {if(args[i]==='--compiler') options.compiler=path.resolve(args[i+1]);else if(args[i]==='--out') options.out=path.resolve(args[i+1]);else throw new Error('Use --audit|--apply --out NEW_DIRECTORY [--compiler PATH]')}
    if(!options.out || fs.existsSync(options.out)) throw new Error('A new --out directory is required')
    console.log(JSON.stringify(run(options)))
  } catch(error) {console.error(error.stack);process.exitCode=1}
}
