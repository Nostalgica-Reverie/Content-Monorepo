<script setup lang="ts">
// Layout customization, and the two controls that make it safe to offer.
//
// Rearranging the shell is explicitly unsupported — panels can end up in docks
// their components were never sized for. That is a fine thing to offer as long
// as it is (a) opt-in, (b) honest about being unsupported, and (c) always
// reversible. "Reset layout" therefore lives here rather than in the shell
// itself: if an arrangement is broken badly enough that the shell is unusable,
// the way out must not be inside the shell.
import { computed } from 'vue'
import Button from '@/components/ui/Button.vue'
import { useLayoutStore, type SidebarSide } from '@/stores/layout'
import { useSettingsStore } from '@/stores/settings'
import { useToastsStore } from '@/stores/toasts'

const layout = useLayoutStore()
const settings = useSettingsStore()
const toasts = useToastsStore()

const SIDES: Array<{ id: SidebarSide; label: string }> = [
	{ id: 'left', label: 'Left of the editor' },
	{ id: 'right', label: 'Right of the editor' },
]

const reduceMotion = computed(() => settings.value?.reduceMotion === true)

async function toggleEditing(value: boolean) {
	await layout.setEditing(value)
	if (value) toasts.push('Layout editing on', 'Rearranging the shell is unsupported.', 'neutral')
}

async function resetLayout() {
	await layout.reset()
	toasts.push('Layout reset', 'The shell is back to its default arrangement.', 'success')
}
</script>

<template>
	<article class="panel span-6">
		<div class="panel-head">
			<h2>Layout &amp; motion</h2>
			<span :class="['status-badge', { integrated: layout.editing }]">
				{{ layout.editing ? 'customizable' : 'default' }}
			</span>
		</div>

		<label class="raw-input-setting">
			<input
				type="checkbox"
				:checked="layout.editing"
				@change="toggleEditing(($event.target as HTMLInputElement).checked)"
			/>
			<span>
				<strong>Let me rearrange the editor</strong>
				<small>
					Move the sidebar to either side of the editor. This is offered as-is: if a custom
					arrangement breaks something, that configuration is not supported — use Reset layout below
					to get back.
				</small>
			</span>
		</label>

		<label class="raw-input-setting">
			<input
				type="checkbox"
				:checked="reduceMotion"
				@change="layout.setReduceMotion(($event.target as HTMLInputElement).checked)"
			/>
			<span>
				<strong>Reduce motion</strong>
				<small>
					Collapse transitions and animations. Applies on top of your system's reduced-motion
					setting, so you can have a still editor without changing the rest of your desktop.
				</small>
			</span>
		</label>

		<div v-if="layout.editing" class="layout-slots">
			<label class="layout-slot">
				<span>Sidebar</span>
				<select
					:value="layout.sidebarSide"
					@change="layout.setSidebarSide(($event.target as HTMLSelectElement).value as SidebarSide)"
				>
					<option v-for="side in SIDES" :key="side.id" :value="side.id">{{ side.label }}</option>
				</select>
			</label>
		</div>

		<!-- Always rendered, never gated behind `layout.editing`: the whole point
         is that it works when the arrangement does not. -->
		<div class="action-row panel-bottom-actions">
			<Button variant="quiet" @click="resetLayout">Reset layout</Button>
		</div>
	</article>
</template>

<style scoped>
.layout-slots {
	display: flex;
	flex-direction: column;
	gap: 0.4rem;
	margin-top: 0.5rem;
}

.layout-slot {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 0.75rem;
	font-size: 0.8rem;
}
.layout-slot select {
	min-width: 9rem;
}
</style>
