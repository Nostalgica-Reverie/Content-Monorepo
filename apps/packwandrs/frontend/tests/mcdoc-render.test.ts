import { describe, expect, test } from 'bun:test'
import type { McdocType } from '@spyglassmc/mcdoc'
import { createSSRApp, h } from 'vue'
import { renderToString } from 'vue/server-renderer'

import McdocField from '../src/mcdoc/McdocField.vue'
import { fixtureSchemaSource, generatorDefinitions } from '../src/mcdoc/fixtures'
import { rootPath } from '../src/mcdoc/simplify'

const source = fixtureSchemaSource
const lootTable = generatorDefinitions.find((entry) => entry.id === 'loot_table')!

/**
 * Renders through the real component. A recursive renderer over a recursive
 * schema is precisely the shape that overflows the stack or hangs, so these
 * assert that it terminates and draws — something the pure-layer tests cannot
 * establish on their own.
 */
function render(type: McdocType, value: unknown) {
  return renderToString(
    createSSRApp({ render: () => h(McdocField, { type, path: rootPath(value), source }) }),
  )
}

/** Walks the fixture schema down to the loot entry type. */
function entryType(): McdocType {
  const pools = (lootTable.type as never as { fields: { key: string; type: McdocType }[] }).fields.find(
    (field) => field.key === 'pools',
  )!.type
  const pool = (pools as never as { item: McdocType }).item
  const entries = (pool as never as { fields: { key: string; type: McdocType }[] }).fields.find(
    (field) => field.key === 'entries',
  )!.type
  return (entries as never as { item: McdocType }).item
}

describe('McdocField rendering', () => {
  test('the loader compiled a real component, not a stand-in', async () => {
    // Guards the guard: if the SFC loader regresses, every other assertion
    // here would pass vacuously against an empty render.
    const html = await render({ kind: 'boolean' }, true)
    expect(html).toContain('mcdoc-check')
  })

  test('draws an empty loot table without recursing forever', async () => {
    const html = await render(lootTable.type, {})
    expect(html).toContain('mcdoc-struct')
    // Every top-level key is optional, so they appear as add buttons.
    expect(html).toContain('+ pools')
  })

  test('draws dispatched fields for a populated entry', async () => {
    const html = await render(entryType(), { type: 'minecraft:item', name: 'minecraft:diamond' })
    expect(html).toContain('minecraft:diamond')
    // `name` exists only on the item arm of the entry dispatcher.
    expect(html).toContain('>name<')
    expect(html).not.toContain('>expand<')
  })

  test('renders a registry picker as a datalist when the schema names one', async () => {
    const html = await render(entryType(), { type: 'minecraft:item', name: '' })
    expect(html).toContain('mcdoc-registry-minecraft:item')
    expect(html).toContain('minecraft:emerald')
  })

  test('survives a deeply nested alternatives chain', async () => {
    let entry: unknown = { type: 'minecraft:item', name: 'minecraft:stick' }
    for (let depth = 0; depth < 8; depth += 1) {
      entry = { type: 'minecraft:alternatives', children: [entry] }
    }
    const html = await render(entryType(), entry)
    expect(html).toContain('mcdoc-list')
    expect(html).toContain('minecraft:stick')
  })

  test('falls back to a raw JSON editor for an unresolved type', async () => {
    const html = await render({ kind: 'reference', path: '::not::in::the::schema' }, { kept: 'value' })
    // The document must survive a schema gap rather than being discarded.
    expect(html).toContain('mcdoc-input--raw')
    expect(html).toContain('kept')
  })

  test('offers a union switcher with readable member names', async () => {
    const rolls: McdocType = {
      kind: 'union',
      members: [
        { kind: 'int' },
        { kind: 'struct', fields: [{ kind: 'pair', key: 'min', type: { kind: 'int' } }] },
      ],
    }
    const html = await render(rolls, 3)
    expect(html).toContain('mcdoc-union')
    expect(html).toContain('integer')
    expect(html).toContain('object')
  })
})
