import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import test from 'node:test'

const require = createRequire(import.meta.url)
const { definitionInBundle } = require('../src/navigation.cjs')
const root = new URL('../../../', import.meta.url)
const manifest = JSON.parse(readFileSync(new URL('scripts/source-migration.json', root), 'utf8'))
const texts = new Map(manifest.files.map(file => [file.path, readFileSync(new URL(file.path, root), 'utf8')]))

function identities(source) {
  return [...source.slice(source.lastIndexOf('identities {')).matchAll(/^\s*([\w.]+):\s*("[^"\n]+")/gm)]
    .map(match => [match[1], JSON.parse(match[2])])
}

test('every migrated selected import navigates to its exact owning declaration', () => {
  let checked = 0
  for (const bundle of manifest.bundles) {
    const source = texts.get(bundle.root)
    const sources = bundle.sources.map(name => ({ name, text: texts.get(name) }))
    for (const [key, id] of identities(source).filter(([key]) => key.includes('.'))) {
      const [group, alias] = key.split('.')
      const selected = new RegExp(`use ${group} \\{([^}]+)\\}`).exec(source)
      assert.ok(selected, key)
      const member = new RegExp(`\\b${alias}\\b`).exec(selected[1])
      assert.ok(member, key)
      const offset = selected.index + selected[0].indexOf('{') + 1 + member.index
      const owner = bundle.sources.find(name => identities(texts.get(name)).some(([name, value]) => !name.includes('.') && value === id))
      assert.ok(owner, id)
      const localName = identities(texts.get(owner)).find(([name, value]) => !name.includes('.') && value === id)[0]
      const declaration = new RegExp(`(?:^|\\n)\\s*(?:fn|record|variant)\\s+(${localName})\\b`).exec(texts.get(owner))
      assert.ok(declaration, id)
      const result = definitionInBundle(sources, bundle.root, offset)
      assert.equal(result?.source, owner, `${bundle.root}: ${key}`)
      assert.equal(result?.target.start, declaration.index + declaration[0].length - localName.length, key)
      checked++
    }
  }
  assert.equal(checked, manifest.files.reduce((total, file) => total + file.import_ids.length, 0))
})
