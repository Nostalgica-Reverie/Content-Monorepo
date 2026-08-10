/**
 * Turning mcdoc types into values, and values back into a choice of type.
 *
 * Two jobs the form layer cannot do without. Adding a list item or filling a
 * newly required key needs a *default* shaped like the type. Rendering a union
 * needs the opposite — given a value already on disk, decide which member the
 * user is currently editing, so switching members is an explicit act rather
 * than something the renderer guesses wrong on every keystroke.
 */

import type { Attribute, AttributeValue, McdocType, NumericRange } from '@spyglassmc/mcdoc'

import type { SchemaSource } from './schema'
import {
	childPath,
	isRecord,
	rootPath,
	simplifyType,
	structFields,
	type ValuePath,
} from './simplify'

const MAX_DEFAULT_DEPTH = 12

/**
 * Builds a value shaped like `type`.
 *
 * Optional struct fields are left out: a generator that pre-fills every
 * optional key produces enormous JSON full of defaults, which is the opposite
 * of what a datapack author wants committed.
 */
export function defaultValue(type: McdocType, source: SchemaSource, depth = 0): unknown {
	if (depth >= MAX_DEFAULT_DEPTH) return undefined
	const simplified = simplifyType(type, rootPath(undefined), source)
	switch (simplified.kind) {
		case 'struct': {
			const result: Record<string, unknown> = {}
			for (const field of structFields(simplified, rootPath(result), source)) {
				if (field.optional || typeof field.key !== 'string') continue
				const value = defaultValue(field.type, source, depth + 1)
				if (value !== undefined) result[field.key] = value
			}
			return result
		}
		case 'list':
		case 'byte_array':
		case 'int_array':
		case 'long_array':
			return []
		case 'tuple':
			return simplified.items.map((item) => defaultValue(item, source, depth + 1))
		case 'string':
			return ''
		case 'boolean':
			return false
		case 'byte':
		case 'short':
		case 'int':
		case 'float':
		case 'double':
			return clampToRange(0, simplified.valueRange)
		case 'long':
			return 0
		case 'literal':
			return simplified.value.kind === 'long'
				? Number(simplified.value.value)
				: simplified.value.value
		case 'enum':
			return simplified.values[0]?.value
		case 'union':
			return simplified.members.length
				? defaultValue(simplified.members[0], source, depth + 1)
				: undefined
		default:
			return undefined
	}
}

function clampToRange(value: number, range: NumericRange<number> | undefined): number {
	if (!range) return value
	if (range.min !== undefined && value < Number(range.min)) return Number(range.min)
	if (range.max !== undefined && value > Number(range.max)) return Number(range.max)
	return value
}

/**
 * Picks which union member `value` currently satisfies, as an index into
 * `members`, or `-1` when nothing matches.
 *
 * Scoring rather than first-match: `{"type": "a"}` may satisfy several struct
 * members structurally, and the one that agrees on the most keys is the one
 * the user meant.
 */
export function selectUnionMember(
	members: readonly McdocType[],
	path: ValuePath,
	source: SchemaSource,
): number {
	let best = -1
	let bestScore = 0
	for (let index = 0; index < members.length; index += 1) {
		const score = matchScore(members[index], path, source)
		if (score > bestScore) {
			best = index
			bestScore = score
		}
	}
	return best
}

/** How well `path.value` fits `type`. Zero means "not this member". */
function matchScore(type: McdocType, path: ValuePath, source: SchemaSource, depth = 0): number {
	if (depth >= MAX_DEFAULT_DEPTH) return 0
	const simplified = simplifyType(type, path, source)
	const value = path.value
	switch (simplified.kind) {
		case 'literal': {
			const literal =
				simplified.value.kind === 'long' ? Number(simplified.value.value) : simplified.value.value
			// A literal is the strongest possible signal: it is the tag that
			// Minecraft's own dispatchers key on.
			return value === literal ? 100 : 0
		}
		case 'string':
			return typeof value === 'string' ? 2 : 0
		case 'boolean':
			return typeof value === 'boolean' ? 2 : 0
		case 'byte':
		case 'short':
		case 'int':
		case 'long':
		case 'float':
		case 'double':
			return typeof value === 'number' ? 2 : 0
		case 'enum':
			return simplified.values.some((entry) => entry.value === value) ? 10 : 0
		case 'list':
		case 'byte_array':
		case 'int_array':
		case 'long_array':
			return Array.isArray(value) ? 2 : 0
		case 'tuple':
			return Array.isArray(value) && value.length === simplified.items.length ? 3 : 0
		case 'struct': {
			if (!isRecord(value)) return 0
			const fields = structFields(simplified, path, source)
			let score = 1
			for (const field of fields) {
				if (typeof field.key !== 'string') continue
				const present = Object.hasOwn(value, field.key)
				if (present) {
					score += 1
					// A matching tag field decides the member outright.
					score +=
						matchScore(field.type, childPath(path, field.key), source, depth + 1) >= 100 ? 100 : 0
				} else if (!field.optional) {
					score -= 1
				}
			}
			return Math.max(score, 0)
		}
		case 'union': {
			let best = 0
			for (const member of simplified.members) {
				best = Math.max(best, matchScore(member, path, source, depth + 1))
			}
			return best
		}
		case 'any':
		case 'unsafe':
			return 1
		default:
			return 0
	}
}

/** Finds an attribute such as `#[id="minecraft:block"]` on a type. */
export function findAttribute(type: McdocType, name: string): Attribute | undefined {
	return type.attributes?.find((attribute) => attribute.name === name)
}

/**
 * Reads an attribute's string payload, accepting both spellings mcdoc allows:
 * `#[id="minecraft:block"]` and `#[id(registry="minecraft:block")]`.
 */
export function attributeString(
	value: AttributeValue | undefined,
	treeKey = 'registry',
): string | undefined {
	if (!value) return undefined
	if (value.kind === 'tree') {
		const nested = value.values[treeKey] ?? value.values[0]
		return nested ? attributeString(nested, treeKey) : undefined
	}
	if (value.kind === 'literal' && value.value.kind === 'string') return value.value.value
	if (value.kind === 'string') return undefined
	return undefined
}

/** The registry an id field points at, when the schema declares one. */
export function idRegistry(type: McdocType): string | undefined {
	const attribute = findAttribute(type, 'id')
	return attribute ? attributeString(attribute.value) : undefined
}

/** A short human label for a type, used on union member switchers. */
export function typeLabel(type: McdocType): string {
	switch (type.kind) {
		case 'literal':
			return String(type.value.value)
		case 'reference':
			return type.path?.split('::').pop() ?? 'reference'
		case 'struct':
			return 'object'
		case 'list':
			return 'list'
		case 'byte':
		case 'short':
		case 'int':
		case 'long':
			return 'integer'
		case 'float':
		case 'double':
			return 'number'
		case 'dispatcher':
			return type.registry.replace(/^minecraft:/, '')
		default:
			return type.kind
	}
}
