import { describe, expect, test } from 'bun:test'
import type { StructType } from '@spyglassmc/mcdoc'

import { fixtureSchemaSource, generatorDefinitions } from '../src/mcdoc/fixtures'
import { rootPath, simplifyType, structFields, structKeys } from '../src/mcdoc/simplify'
import { defaultValue, selectUnionMember } from '../src/mcdoc/value'

const lootTable = generatorDefinitions.find((entry) => entry.id === 'loot_table')!
const source = fixtureSchemaSource

/**
 * The fields the renderer would draw for a value at the document root,
 * following a union to the member the value selects the way the component
 * does.
 */
function keysFor(type: Parameters<typeof simplifyType>[0], value: unknown): string[] {
	const path = rootPath(value)
	let simplified = simplifyType(type, path, source)
	if (simplified.kind === 'union') {
		const member = selectUnionMember(simplified.members, path, source)
		if (member < 0) return []
		simplified = simplifyType(simplified.members[member], path, source)
	}
	if (simplified.kind !== 'struct') return []
	return structKeys(structFields(simplified as StructType, path, source))
}

describe('loot table generator', () => {
	test('starts as an empty document, not a wall of defaults', () => {
		// Every top-level field is optional, so a new table should be `{}` rather
		// than pre-filled — that is what makes generated files reviewable.
		expect(defaultValue(lootTable.type, source)).toEqual({})
	})

	test('an entry dispatches its fields off the type tag', () => {
		const entryType = { kind: 'reference' as const, path: '::entry' }
		const pools = defaultValue(lootTable.type, source)
		expect(pools).toBeDefined()

		// Reach the entry type through the schema rather than restating it.
		const table = simplifyType(lootTable.type, rootPath({}), source) as StructType
		const poolsField = structFields(table, rootPath({}), source).find(
			(field) => field.key === 'pools',
		)!
		const poolList = simplifyType(poolsField.type, rootPath([]), source)
		expect(poolList.kind).toBe('list')
		const pool = simplifyType(
			(poolList as never as { item: typeof entryType }).item,
			rootPath({}),
			source,
		) as StructType
		const entriesField = structFields(pool, rootPath({}), source).find(
			(field) => field.key === 'entries',
		)!
		const entryList = simplifyType(entriesField.type, rootPath([]), source)
		const entry = (entryList as never as { item: typeof entryType }).item

		expect(keysFor(entry, { type: 'minecraft:item' })).toContain('name')
		expect(keysFor(entry, { type: 'minecraft:tag' })).toContain('expand')
		expect(keysFor(entry, { type: 'minecraft:item' })).not.toContain('expand')
	})

	test('an unknown tag still renders the shared fields', () => {
		const table = simplifyType(lootTable.type, rootPath({}), source) as StructType
		const poolsField = structFields(table, rootPath({}), source).find(
			(field) => field.key === 'pools',
		)!
		const poolList = simplifyType(poolsField.type, rootPath([]), source) as never as { item: never }
		const pool = simplifyType(poolList.item, rootPath({}), source) as StructType
		const entriesField = structFields(pool, rootPath({}), source).find(
			(field) => field.key === 'entries',
		)!
		const entryList = simplifyType(entriesField.type, rootPath([]), source) as never as {
			item: never
		}

		const keys = keysFor(entryList.item, { type: 'modded:custom_entry' })
		// The dispatcher misses, but the fixed keys still render so the document
		// stays editable instead of collapsing to a raw JSON box.
		expect(keys).toContain('type')
		expect(keys).toContain('weight')
	})

	test('rolls accepts either a number or a min/max object', () => {
		const table = simplifyType(lootTable.type, rootPath({}), source) as StructType
		const poolsField = structFields(table, rootPath({}), source).find(
			(field) => field.key === 'pools',
		)!
		const poolList = simplifyType(poolsField.type, rootPath([]), source) as never as { item: never }
		const pool = simplifyType(poolList.item, rootPath({}), source) as StructType
		const rolls = structFields(pool, rootPath({}), source).find((field) => field.key === 'rolls')!
		const simplified = simplifyType(rolls.type, rootPath(1), source)
		expect(simplified.kind).toBe('union')

		const members = (simplified as never as { members: never[] }).members
		expect(selectUnionMember(members, rootPath(3), source)).toBe(0)
		expect(selectUnionMember(members, rootPath({ min: 1, max: 4 }), source)).toBe(1)
	})
})

describe('recursive schemas', () => {
	test('a deeply nested alternatives chain resolves at every level', () => {
		// `alternatives` holds entries, which may themselves be alternatives. A
		// recursive schema is what makes a recursive renderer overflow, so the
		// resolution underneath it has to stay bounded and correct at depth.
		const table = simplifyType(lootTable.type, rootPath({}), source) as StructType
		const poolsField = structFields(table, rootPath({}), source).find(
			(field) => field.key === 'pools',
		)!
		const poolList = simplifyType(poolsField.type, rootPath([]), source) as never as { item: never }
		const pool = simplifyType(poolList.item, rootPath({}), source) as StructType
		const entriesField = structFields(pool, rootPath({}), source).find(
			(field) => field.key === 'entries',
		)!
		const entryType = (
			simplifyType(entriesField.type, rootPath([]), source) as never as { item: never }
		).item

		let value: unknown = { type: 'minecraft:item', name: 'minecraft:stick' }
		for (let depth = 0; depth < 16; depth += 1) {
			expect(keysFor(entryType, value)).toContain('type')
			value = { type: 'minecraft:alternatives', children: [value] }
		}
		expect(keysFor(entryType, value)).toContain('children')
	})

	test('an unresolved reference degrades to a raw editor and keeps the value', () => {
		// `any` is what the component renders as a JSON textarea, so this is the
		// guarantee that a schema gap never silently discards a user's document.
		const simplified = simplifyType(
			{ kind: 'reference', path: '::not::in::the::schema' },
			rootPath({ kept: 1 }),
			source,
		)
		expect(simplified.kind).toBe('any')
	})
})

describe('predicate generator', () => {
	const predicate = generatorDefinitions.find((entry) => entry.id === 'predicate')!

	test('is a union of one condition or a list of them', () => {
		const simplified = simplifyType(predicate.type, rootPath({}), source)
		expect(simplified.kind).toBe('union')
		const members = (simplified as never as { members: never[] }).members
		expect(
			selectUnionMember(members, rootPath({ condition: 'minecraft:random_chance' }), source),
		).toBe(0)
		expect(selectUnionMember(members, rootPath([]), source)).toBe(1)
	})

	test('an inverted condition nests another condition', () => {
		expect(keysFor(predicate.type, { condition: 'minecraft:inverted', term: {} })).toContain('term')
	})
})
