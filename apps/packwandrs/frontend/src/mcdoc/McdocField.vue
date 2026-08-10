<script setup lang="ts">
/**
 * One field of an mcdoc-described document, rendered recursively.
 *
 * This is the whole visual generator. Because upstream's generators are mcdoc
 * schemas rather than hand-written forms, every registry vanilla-mcdoc
 * describes — loot tables, predicates, recipes, worldgen — is drawn by this
 * one component. Adding a generator is adding a schema, not a screen.
 *
 * Values are treated as immutable: a change rebuilds the containers on the way
 * up and emits, so the owner keeps a single source of truth and undo stays
 * possible.
 */
import { computed } from 'vue'
import type { McdocType, StructTypePairField } from '@spyglassmc/mcdoc'

import type { SchemaSource } from './schema'
import { childPath, isRecord, simplifyType, structFields, type ValuePath } from './simplify'
import { defaultValue, idRegistry, selectUnionMember, typeLabel } from './value'

defineOptions({ name: 'McdocField' })

const props = withDefaults(
	defineProps<{
		type: McdocType
		/** The value being edited, together with where it sits. */
		path: ValuePath
		source: SchemaSource
		label?: string
		description?: string
		/** Guards against a schema that recurses further than any real document. */
		depth?: number
	}>(),
	{ depth: 0 },
)

const emit = defineEmits<{ (event: 'update', value: unknown): void }>()

const MAX_RENDER_DEPTH = 24

const simplified = computed(() => simplifyType(props.type, props.path, props.source))
const value = computed(() => props.path.value)
const registry = computed(() => idRegistry(simplified.value))

const fields = computed<StructTypePairField[]>(() =>
	simplified.value.kind === 'struct'
		? structFields(simplified.value, props.path, props.source)
		: [],
)

/** Required fields always show; optional ones only once the user adds them. */
const shownFields = computed(() =>
	fields.value.filter(
		(field) =>
			typeof field.key === 'string' &&
			(!field.optional || (isRecord(value.value) && Object.hasOwn(value.value, field.key))),
	),
)

const addableFields = computed(() =>
	fields.value.filter(
		(field) =>
			typeof field.key === 'string' &&
			field.optional &&
			!(isRecord(value.value) && Object.hasOwn(value.value, field.key)),
	),
)

function setKey(key: string, next: unknown) {
	emit('update', { ...(isRecord(value.value) ? value.value : {}), [key]: next })
}

function addKey(field: StructTypePairField) {
	setKey(field.key as string, defaultValue(field.type, props.source))
}

function removeKey(key: string) {
	if (!isRecord(value.value)) return
	const { [key]: _removed, ...rest } = value.value
	emit('update', rest)
}

const items = computed(() => (Array.isArray(value.value) ? value.value : []))

function setItem(index: number, next: unknown) {
	const copy = [...items.value]
	copy[index] = next
	emit('update', copy)
}

function addItem() {
	if (simplified.value.kind !== 'list') return
	emit('update', [...items.value, defaultValue(simplified.value.item, props.source)])
}

function removeItem(index: number) {
	emit(
		'update',
		items.value.filter((_, at) => at !== index),
	)
}

function moveItem(index: number, by: number) {
	const target = index + by
	if (target < 0 || target >= items.value.length) return
	const copy = [...items.value]
	;[copy[index], copy[target]] = [copy[target], copy[index]]
	emit('update', copy)
}

const members = computed(() => (simplified.value.kind === 'union' ? simplified.value.members : []))
const activeMember = computed(() => selectUnionMember(members.value, props.path, props.source))

function switchMember(index: number) {
	if (index === activeMember.value) return
	emit('update', defaultValue(members.value[index], props.source))
}

const enumValues = computed(() => (simplified.value.kind === 'enum' ? simplified.value.values : []))

const isInteger = computed(() => ['byte', 'short', 'int', 'long'].includes(simplified.value.kind))

function onNumber(event: Event) {
	const raw = (event.target as HTMLInputElement).value
	if (raw === '') return emit('update', 0)
	const parsed = isInteger.value ? Number.parseInt(raw, 10) : Number.parseFloat(raw)
	if (!Number.isNaN(parsed)) emit('update', parsed)
}

/** `any` and unresolved references fall back to raw JSON rather than dropping data. */
const rawJson = computed(() => {
	try {
		return value.value === undefined ? '' : JSON.stringify(value.value, null, 2)
	} catch {
		return ''
	}
})

function onRawJson(event: Event) {
	const raw = (event.target as HTMLTextAreaElement).value
	if (raw.trim() === '') return emit('update', undefined)
	try {
		emit('update', JSON.parse(raw))
	} catch {
		// Invalid JSON mid-typing is expected; the last valid value stands.
	}
}
</script>

<template>
	<div class="mcdoc-field">
		<div v-if="label" class="mcdoc-field__head">
			<span class="mcdoc-field__label">{{ label }}</span>
			<span v-if="description" class="mcdoc-field__desc">{{ description }}</span>
		</div>

		<p v-if="depth >= MAX_RENDER_DEPTH" class="mcdoc-field__note">Nested too deeply to render.</p>

		<!-- Struct -->
		<div v-else-if="simplified.kind === 'struct'" class="mcdoc-struct">
			<div v-for="field in shownFields" :key="String(field.key)" class="mcdoc-struct__row">
				<McdocField
					:type="field.type"
					:path="childPath(path, String(field.key))"
					:source="source"
					:label="String(field.key)"
					:description="field.desc"
					:depth="depth + 1"
					@update="setKey(String(field.key), $event)"
				/>
				<button
					v-if="field.optional"
					class="mcdoc-btn mcdoc-btn--quiet"
					type="button"
					:title="`Remove ${String(field.key)}`"
					@click="removeKey(String(field.key))"
				>
					Remove
				</button>
			</div>
			<div v-if="addableFields.length" class="mcdoc-struct__add">
				<button
					v-for="field in addableFields"
					:key="String(field.key)"
					class="mcdoc-btn"
					type="button"
					@click="addKey(field)"
				>
					+ {{ String(field.key) }}
				</button>
			</div>
		</div>

		<!-- Union -->
		<div v-else-if="simplified.kind === 'union'" class="mcdoc-union">
			<select
				class="mcdoc-input mcdoc-input--select"
				:value="activeMember"
				@change="switchMember(Number(($event.target as HTMLSelectElement).value))"
			>
				<option v-if="activeMember === -1" :value="-1" disabled>Select a type…</option>
				<option v-for="(member, index) in members" :key="index" :value="index">
					{{ typeLabel(member) }}
				</option>
			</select>
			<McdocField
				v-if="activeMember >= 0"
				:type="members[activeMember]"
				:path="path"
				:source="source"
				:depth="depth + 1"
				@update="emit('update', $event)"
			/>
		</div>

		<!-- List -->
		<div v-else-if="simplified.kind === 'list'" class="mcdoc-list">
			<div v-for="(_, index) in items" :key="index" class="mcdoc-list__row">
				<span class="mcdoc-list__index">{{ index }}</span>
				<McdocField
					:type="simplified.item"
					:path="childPath(path, index)"
					:source="source"
					:depth="depth + 1"
					@update="setItem(index, $event)"
				/>
				<div class="mcdoc-list__actions">
					<button
						class="mcdoc-btn mcdoc-btn--quiet"
						type="button"
						title="Move up"
						@click="moveItem(index, -1)"
					>
						↑
					</button>
					<button
						class="mcdoc-btn mcdoc-btn--quiet"
						type="button"
						title="Move down"
						@click="moveItem(index, 1)"
					>
						↓
					</button>
					<button
						class="mcdoc-btn mcdoc-btn--quiet"
						type="button"
						title="Remove"
						@click="removeItem(index)"
					>
						×
					</button>
				</div>
			</div>
			<button class="mcdoc-btn" type="button" @click="addItem">+ Add item</button>
		</div>

		<!-- Enum -->
		<select
			v-else-if="simplified.kind === 'enum'"
			class="mcdoc-input mcdoc-input--select"
			:value="value"
			@change="emit('update', ($event.target as HTMLSelectElement).value)"
		>
			<option v-for="entry in enumValues" :key="entry.identifier" :value="String(entry.value)">
				{{ entry.identifier }}
			</option>
		</select>

		<!-- Literal: the dispatch tag. Shown for orientation, changed via the union above. -->
		<input
			v-else-if="simplified.kind === 'literal'"
			class="mcdoc-input"
			readonly
			:value="String(simplified.value.value)"
		/>

		<!-- Boolean -->
		<label v-else-if="simplified.kind === 'boolean'" class="mcdoc-check">
			<input
				type="checkbox"
				:checked="value === true"
				@change="emit('update', ($event.target as HTMLInputElement).checked)"
			/>
			<span>{{ value === true ? 'true' : 'false' }}</span>
		</label>

		<!-- Numbers -->
		<input
			v-else-if="['byte', 'short', 'int', 'long', 'float', 'double'].includes(simplified.kind)"
			class="mcdoc-input"
			type="number"
			:step="isInteger ? 1 : 'any'"
			:value="typeof value === 'number' ? value : ''"
			@input="onNumber"
		/>

		<!-- Strings, with a registry-backed picker when the schema names one -->
		<template v-else-if="simplified.kind === 'string'">
			<input
				class="mcdoc-input"
				type="text"
				:list="registry ? `mcdoc-registry-${registry}` : undefined"
				:value="typeof value === 'string' ? value : ''"
				@input="emit('update', ($event.target as HTMLInputElement).value)"
			/>
			<datalist v-if="registry" :id="`mcdoc-registry-${registry}`">
				<option v-for="id in source.registry?.(registry) ?? []" :key="id" :value="id" />
			</datalist>
		</template>

		<!-- Anything the schema could not pin down -->
		<textarea
			v-else
			class="mcdoc-input mcdoc-input--raw"
			spellcheck="false"
			:value="rawJson"
			@change="onRawJson"
		/>
	</div>
</template>

<style scoped>
.mcdoc-field {
	display: flex;
	flex-direction: column;
	gap: 4px;
	min-width: 0;
	flex: 1;
}

.mcdoc-field__head {
	display: flex;
	align-items: baseline;
	gap: 8px;
}

.mcdoc-field__label {
	font-size: 12px;
	font-weight: 600;
	color: var(--text);
}

.mcdoc-field__desc,
.mcdoc-field__note {
	font-size: 11px;
	color: var(--muted);
	margin: 0;
}

.mcdoc-struct,
.mcdoc-list,
.mcdoc-union {
	display: flex;
	flex-direction: column;
	gap: 8px;
	border-left: 1px solid var(--line-soft);
	padding-left: 10px;
}

.mcdoc-struct__row,
.mcdoc-list__row {
	display: flex;
	align-items: flex-start;
	gap: 8px;
}

.mcdoc-struct__add {
	display: flex;
	flex-wrap: wrap;
	gap: 4px;
}

.mcdoc-list__index {
	font-family: var(--mono);
	font-size: 11px;
	color: var(--muted);
	padding-top: 6px;
	min-width: 18px;
}

.mcdoc-list__actions {
	display: flex;
	gap: 2px;
}

.mcdoc-input {
	background: var(--surface-2);
	border: 1px solid var(--line);
	border-radius: var(--radius);
	color: var(--text);
	font-family: inherit;
	font-size: 12px;
	padding: 5px 8px;
	width: 100%;
	min-width: 0;
}

.mcdoc-input:focus-visible {
	outline: 1px solid var(--accent);
	border-color: var(--accent);
}

.mcdoc-input[readonly] {
	color: var(--muted);
	background: var(--surface-3);
}

.mcdoc-input--raw {
	font-family: var(--mono);
	min-height: 72px;
	resize: vertical;
}

.mcdoc-check {
	display: flex;
	align-items: center;
	gap: 6px;
	font-size: 12px;
	color: var(--text);
}

.mcdoc-btn {
	background: var(--surface-2);
	border: 1px solid var(--line);
	border-radius: var(--radius);
	color: var(--text);
	cursor: pointer;
	font-size: 11px;
	padding: 4px 8px;
}

.mcdoc-btn:hover {
	background: var(--hover);
}

.mcdoc-btn--quiet {
	background: transparent;
	color: var(--muted);
}
</style>
