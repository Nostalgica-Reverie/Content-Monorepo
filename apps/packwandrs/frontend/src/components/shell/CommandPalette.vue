<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import AppIcon from '@/components/ui/AppIcon.vue'

export interface PaletteCommand {
	id: string
	label: string
	group: string
	icon: string
	hint?: string
	run: () => void | Promise<void>
}

const props = defineProps<{ open: boolean; seed: string; commands: PaletteCommand[] }>()
const emit = defineEmits<{ close: [] }>()

const query = ref('')
const cursor = ref(0)
const input = ref<HTMLInputElement | null>(null)

const matches = computed(() => {
	const term = query.value.trim().toLowerCase()
	if (!term) return props.commands
	return props.commands.filter((command) =>
		(command.group + ' ' + command.label).toLowerCase().includes(term),
	)
})

/** Group headers are emitted whenever the group changes down the flat list. */
const rows = computed(() => {
	let previous = ''
	return matches.value.map((command) => {
		const header = command.group === previous ? null : command.group
		previous = command.group
		return { command, header }
	})
})

watch(
	() => props.open,
	async (open) => {
		if (!open) return
		query.value = props.seed
		cursor.value = 0
		await nextTick()
		input.value?.focus()
		input.value?.select()
	},
)

watch(matches, () => {
	if (cursor.value >= matches.value.length) cursor.value = Math.max(0, matches.value.length - 1)
})

function move(delta: number) {
	if (!matches.value.length) return
	cursor.value = (cursor.value + delta + matches.value.length) % matches.value.length
}

async function accept(command = matches.value[cursor.value]) {
	if (!command) return
	emit('close')
	await command.run()
}
</script>

<template>
	<Transition name="fade">
		<div v-if="open" class="palette-backdrop" @click.self="emit('close')">
			<Transition name="scale-fade" appear>
				<div class="palette" role="dialog" aria-modal="true" aria-label="Command palette">
					<input
						ref="input"
						v-model="query"
						class="palette__input"
						placeholder="Search commands, projects, and pack targets…"
						spellcheck="false"
						@keydown.down.prevent="move(1)"
						@keydown.up.prevent="move(-1)"
						@keydown.enter.prevent="accept()"
						@keydown.esc.prevent="emit('close')"
					/>
					<div class="palette__list">
						<p v-if="!rows.length" class="palette__empty">No matching commands.</p>
						<template v-for="(row, index) in rows" :key="row.command.id">
							<p v-if="row.header" class="palette__group">{{ row.header }}</p>
							<button
								class="palette__item"
								:class="{ active: index === cursor }"
								@click="accept(row.command)"
								@mouseenter="cursor = index"
							>
								<AppIcon :name="row.command.icon" :size="15" class="palette__item-icon" />
								<span class="palette__item-label">{{ row.command.label }}</span>
								<span v-if="row.command.hint" class="palette__item-hint">{{
									row.command.hint
								}}</span>
							</button>
						</template>
					</div>
				</div>
			</Transition>
		</div>
	</Transition>
</template>
