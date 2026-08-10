import { describe, expect, test } from 'bun:test'
import type { McdocType } from '@spyglassmc/mcdoc'

import { createMemorySchemaSource, emptySchemaSource } from '../src/mcdoc/schema'
import { rootPath, simplifyType, structFields, structKeys } from '../src/mcdoc/simplify'
import { defaultValue, idRegistry, selectUnionMember, typeLabel } from '../src/mcdoc/value'

const stringField = (key: string, optional = false) =>
	({ kind: 'pair', key, type: { kind: 'string' }, optional }) as const

describe('simplifyType', () => {
	test('follows a reference to its definition', () => {
		const source = createMemorySchemaSource({
			references: { '::test::Name': { kind: 'string' } },
		})
		const simplified = simplifyType(
			{ kind: 'reference', path: '::test::Name' },
			rootPath(''),
			source,
		)
		expect(simplified.kind).toBe('string')
	})

	test('degrades an unresolvable reference to any rather than throwing', () => {
		const simplified = simplifyType(
			{ kind: 'reference', path: '::missing' },
			rootPath(''),
			emptySchemaSource,
		)
		expect(simplified.kind).toBe('any')
	})

	test('stops on a reference cycle instead of hanging', () => {
		const source = createMemorySchemaSource({
			references: {
				'::a': { kind: 'reference', path: '::b' },
				'::b': { kind: 'reference', path: '::a' },
			},
		})
		expect(simplifyType({ kind: 'reference', path: '::a' }, rootPath(''), source).kind).toBe('any')
	})

	test('dispatches on a sibling field, the way Minecraft tags its JSON', () => {
		const source = createMemorySchemaSource({
			dispatchers: {
				'minecraft:loot_function': {
					set_count: { kind: 'struct', fields: [stringField('count')] },
					set_name: { kind: 'struct', fields: [stringField('name')] },
				},
			},
		})
		const dispatcher: McdocType = {
			kind: 'dispatcher',
			registry: 'minecraft:loot_function',
			parallelIndices: [{ kind: 'dynamic', accessor: ['function'] }],
		}
		const value = { function: 'set_name', name: 'x' }
		const simplified = simplifyType(dispatcher, rootPath(value), source)
		expect(simplified.kind).toBe('struct')
		expect(structKeys(structFields(simplified as never, rootPath(value), source))).toEqual(['name'])
	})

	test('falls back to a static index when the dynamic one is absent', () => {
		const source = createMemorySchemaSource({
			dispatchers: { 'minecraft:thing': { fallback: { kind: 'string' } } },
		})
		const dispatcher: McdocType = {
			kind: 'dispatcher',
			registry: 'minecraft:thing',
			parallelIndices: [
				{ kind: 'dynamic', accessor: ['type'] },
				{ kind: 'static', value: 'fallback' },
			],
		}
		expect(simplifyType(dispatcher, rootPath({}), source).kind).toBe('string')
	})

	test('resolves a parent accessor by walking up the value path', () => {
		const source = createMemorySchemaSource({
			dispatchers: { 'minecraft:thing': { outer: { kind: 'boolean' } } },
		})
		const dispatcher: McdocType = {
			kind: 'dispatcher',
			registry: 'minecraft:thing',
			parallelIndices: [{ kind: 'dynamic', accessor: [{ keyword: 'parent' }, 'type'] }],
		}
		const parent = rootPath({ type: 'outer', child: {} })
		const path = { value: {}, key: 'child', parent }
		expect(simplifyType(dispatcher, path, source).kind).toBe('boolean')
	})
})

describe('structFields', () => {
	test('inlines spread fields', () => {
		const source = createMemorySchemaSource({
			references: { '::shared': { kind: 'struct', fields: [stringField('shared')] } },
		})
		const struct = {
			kind: 'struct',
			fields: [
				stringField('own'),
				{ kind: 'spread', type: { kind: 'reference', path: '::shared' } },
			],
		} as const
		const fields = structFields(struct as never, rootPath({}), source)
		expect(structKeys(fields)).toEqual(['own', 'shared'])
	})

	test('a later field with the same key overrides an earlier one', () => {
		const struct = {
			kind: 'struct',
			fields: [
				stringField('id'),
				{ kind: 'pair', key: 'id', type: { kind: 'boolean' }, optional: true },
			],
		} as const
		const fields = structFields(struct as never, rootPath({}), emptySchemaSource)
		expect(fields).toHaveLength(1)
		expect(fields[0].type.kind).toBe('boolean')
	})
})

describe('defaultValue', () => {
	test('fills required struct fields and omits optional ones', () => {
		const struct: McdocType = {
			kind: 'struct',
			fields: [stringField('required'), stringField('optional', true)],
		}
		expect(defaultValue(struct, emptySchemaSource)).toEqual({ required: '' })
	})

	test('respects a numeric minimum', () => {
		const type: McdocType = { kind: 'int', valueRange: { kind: 0, min: 5 } as never }
		expect(defaultValue(type, emptySchemaSource)).toBe(5)
	})

	test('uses the first enum value', () => {
		const type: McdocType = {
			kind: 'enum',
			enumKind: 'string',
			values: [
				{ identifier: 'first', value: 'first' },
				{ identifier: 'second', value: 'second' },
			],
		}
		expect(defaultValue(type, emptySchemaSource)).toBe('first')
	})

	test('terminates on a self-referential struct', () => {
		const source = createMemorySchemaSource({
			references: {
				'::node': {
					kind: 'struct',
					fields: [{ kind: 'pair', key: 'child', type: { kind: 'reference', path: '::node' } }],
				},
			},
		})
		expect(() => defaultValue({ kind: 'reference', path: '::node' }, source)).not.toThrow()
	})
})

describe('selectUnionMember', () => {
	const members: McdocType[] = [
		{ kind: 'string' },
		{ kind: 'struct', fields: [stringField('name')] },
		{ kind: 'list', item: { kind: 'string' } },
	]

	test('picks the member matching the value shape', () => {
		expect(selectUnionMember(members, rootPath('text'), emptySchemaSource)).toBe(0)
		expect(selectUnionMember(members, rootPath({ name: 'x' }), emptySchemaSource)).toBe(1)
		expect(selectUnionMember(members, rootPath([]), emptySchemaSource)).toBe(2)
	})

	test('a literal tag outweighs a structurally similar member', () => {
		const tagged: McdocType[] = [
			{
				kind: 'struct',
				fields: [
					{
						kind: 'pair',
						key: 'type',
						type: { kind: 'literal', value: { kind: 'string', value: 'a' } },
					},
				],
			},
			{
				kind: 'struct',
				fields: [
					{
						kind: 'pair',
						key: 'type',
						type: { kind: 'literal', value: { kind: 'string', value: 'b' } },
					},
				],
			},
		]
		expect(selectUnionMember(tagged, rootPath({ type: 'b' }), emptySchemaSource)).toBe(1)
	})

	test('reports no match for a value nothing accepts', () => {
		expect(selectUnionMember([{ kind: 'boolean' }], rootPath('text'), emptySchemaSource)).toBe(-1)
	})
})

describe('attributes', () => {
	test('reads a registry off an id attribute', () => {
		const type: McdocType = {
			kind: 'string',
			attributes: [
				{
					name: 'id',
					value: { kind: 'literal', value: { kind: 'string', value: 'minecraft:block' } },
				},
			],
		}
		expect(idRegistry(type)).toBe('minecraft:block')
	})

	test('survives a reference being followed', () => {
		const source = createMemorySchemaSource({ references: { '::id': { kind: 'string' } } })
		const type: McdocType = {
			kind: 'reference',
			path: '::id',
			attributes: [
				{
					name: 'id',
					value: { kind: 'literal', value: { kind: 'string', value: 'minecraft:item' } },
				},
			],
		}
		expect(idRegistry(simplifyType(type, rootPath(''), source))).toBe('minecraft:item')
	})
})

describe('typeLabel', () => {
	test('names union members readably', () => {
		expect(typeLabel({ kind: 'literal', value: { kind: 'string', value: 'set_count' } })).toBe(
			'set_count',
		)
		expect(typeLabel({ kind: 'reference', path: '::java::data::LootPool' })).toBe('LootPool')
		expect(typeLabel({ kind: 'int' })).toBe('integer')
	})
})
