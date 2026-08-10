/**
 * Reducing an mcdoc type to something a form can draw.
 *
 * Seven of mcdoc's seventeen kinds are indirections rather than shapes:
 * `reference`, `dispatcher`, `indexed`, `concrete`, `template`, `mapped`, and
 * spread fields inside a struct. None of them can be rendered — each has to be
 * followed to the concrete type underneath first, and following some of them
 * needs the surrounding *value*, because Minecraft's JSON dispatches on
 * sibling fields (`{"function": "set_count", ...}` selects which other fields
 * are legal).
 *
 * That is why simplification takes a value path rather than a bare value: an
 * accessor may walk up to a parent or read the key a value sits under.
 */

import type { Index, McdocType, StructType, StructTypePairField } from '@spyglassmc/mcdoc'

import type { SchemaSource } from './schema'

/**
 * A value together with where it sits, so dynamic index accessors can walk
 * upwards. Built as the renderer descends; the root has no parent.
 */
export interface ValuePath {
	value: unknown
	key?: string | number
	parent?: ValuePath
}

/** Creates a root path. */
export function rootPath(value: unknown): ValuePath {
	return { value }
}

/** Descends into a property, recording the step for `parent` accessors. */
export function childPath(parent: ValuePath, key: string | number): ValuePath {
	const container = parent.value
	const value = isRecord(container)
		? container[String(key)]
		: Array.isArray(container)
			? container[Number(key)]
			: undefined
	return { value, key, parent }
}

export function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

/**
 * References can point at each other in cycles (`LootEntry` contains a list of
 * `LootEntry`). Simplification stops after this many hops and returns `any`,
 * which renders as a raw JSON field rather than hanging the UI.
 */
const MAX_RESOLUTION_DEPTH = 32

/**
 * Follows indirections until `type` is a kind the renderer can draw.
 *
 * Returns `{ kind: 'any' }` when a reference cannot be resolved — an
 * unresolved reference means the schema source is incomplete, which should
 * degrade to a raw editor rather than lose the user's data.
 */
export function simplifyType(type: McdocType, path: ValuePath, source: SchemaSource): McdocType {
	let current = type
	for (let depth = 0; depth < MAX_RESOLUTION_DEPTH; depth += 1) {
		switch (current.kind) {
			case 'reference': {
				if (!current.path) return { kind: 'any' }
				const resolved = source.reference(current.path)
				if (!resolved) return { kind: 'any' }
				current = mergeAttributes(resolved, current)
				break
			}
			case 'dispatcher': {
				const index = resolveIndices(current.parallelIndices, path)
				if (index === undefined) return { kind: 'any' }
				const resolved = source.dispatch(current.registry, index)
				if (!resolved) return { kind: 'any' }
				current = mergeAttributes(resolved, current)
				break
			}
			case 'indexed': {
				const index = resolveIndices(current.parallelIndices, path)
				const child = simplifyType(current.child, path, source)
				if (index === undefined || child.kind !== 'struct') return child
				const field = findPairField(child, index)
				if (!field) return { kind: 'any' }
				current = field.type
				break
			}
			// Generics are not parameterised by anything the form layer varies, so
			// both collapse to the type they wrap.
			case 'concrete':
			case 'template':
				current = current.child
				break
			case 'mapped':
				current = current.child
				break
			default:
				return current
		}
	}
	return { kind: 'any' }
}

/**
 * Resolves the first index that yields a value. Parallel indices exist so a
 * dispatcher can fall back — `[[id, fallback]]` tries `id` and then a static
 * `fallback` arm.
 */
function resolveIndices(indices: readonly Index[], path: ValuePath): string | undefined {
	for (const index of indices) {
		const resolved = resolveIndex(index, path)
		if (resolved !== undefined) return resolved
	}
	return undefined
}

function resolveIndex(index: Index, path: ValuePath): string | undefined {
	if (index.kind === 'static') return index.value

	let cursor: ValuePath | undefined = path
	for (const step of index.accessor) {
		if (!cursor) return undefined
		if (typeof step === 'string') {
			cursor = isRecord(cursor.value)
				? { value: cursor.value[step], key: step, parent: cursor }
				: undefined
			continue
		}
		if (step.keyword === 'parent') {
			cursor = cursor.parent
			continue
		}
		// `key` replaces the cursor with the key the current value sits under.
		cursor = cursor.key === undefined ? undefined : { value: String(cursor.key) }
	}
	const value = cursor?.value
	return typeof value === 'string' ? value : undefined
}

/**
 * Attributes on the indirection (`#[id="block"]` on a reference) have to
 * survive being followed, or id pickers lose the registry they point at. The
 * indirection's own attributes win, being the more specific of the two.
 */
function mergeAttributes(resolved: McdocType, indirection: McdocType): McdocType {
	if (!indirection.attributes?.length) return resolved
	return {
		...resolved,
		attributes: [...(resolved.attributes ?? []), ...indirection.attributes],
	} as McdocType
}

function findPairField(struct: StructType, key: string): StructTypePairField | undefined {
	for (const field of struct.fields) {
		if (field.kind !== 'pair') continue
		if (typeof field.key === 'string' && field.key === key) return field
	}
	return undefined
}

/**
 * Flattens a struct's `spread` fields into their pairs.
 *
 * A spread inlines another struct's fields at this level, and it may itself be
 * a dispatcher — which is exactly how `{"type": "x", ...}` mixes fixed keys
 * with type-dependent ones. Resolving spreads is therefore value-dependent and
 * has to happen per render rather than once at load.
 */
export function structFields(
	struct: StructType,
	path: ValuePath,
	source: SchemaSource,
	depth = 0,
): StructTypePairField[] {
	const fields: StructTypePairField[] = []
	for (const field of struct.fields) {
		if (field.kind === 'pair') {
			fields.push(field)
			continue
		}
		if (depth >= MAX_RESOLUTION_DEPTH) continue
		const spread = simplifyType(field.type, path, source)
		if (spread.kind === 'struct') {
			fields.push(...structFields(spread, path, source, depth + 1))
		}
	}
	// A later field with the same key overrides an earlier one, matching how a
	// spread that reopens a key behaves in mcdoc.
	const byKey = new Map<string, StructTypePairField>()
	const dynamic: StructTypePairField[] = []
	for (const field of fields) {
		if (typeof field.key === 'string') byKey.set(field.key, field)
		else dynamic.push(field)
	}
	return [...byKey.values(), ...dynamic]
}

/** The literal string keys a struct declares, in order. */
export function structKeys(fields: readonly StructTypePairField[]): string[] {
	return fields.filter((field) => typeof field.key === 'string').map((field) => field.key as string)
}
