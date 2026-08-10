<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import SideSection from '@/components/shell/SideSection.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import type { ExtensionRow } from '@/extensions/api'
import { useExtensionsStore } from '@/stores/extensions'
import type { RegisteredView } from '@/stores/extensions'
import { useWorkbenchStore } from '@/stores/workbench'
import { onPacksChanged } from '@/helpers/events'

const props = defineProps<{ entry: RegisteredView }>()

const extensionsStore = useExtensionsStore()
const workbench = useWorkbenchStore()
const rows = ref<ExtensionRow[]>([])
const loading = ref(false)
let stopWatching: (() => void) | undefined

async function load() {
	loading.value = true
	try {
		rows.value = await extensionsStore.rowsFor(props.entry.id)
	} finally {
		loading.value = false
	}
}

// Rows are derived from the selected pack, so refresh when the selection moves.
watch(
	() => [props.entry.id, workbench.selectedPackId, workbench.selectedProjectId],
	() => void load(),
	{ immediate: true },
)

onMounted(async () => {
	stopWatching = await onPacksChanged(() => void load())
})
onBeforeUnmount(() => stopWatching?.())
</script>

<template>
	<SideSection :title="entry.view.title" :count="rows.length" :open="false">
		<p v-if="loading" class="side-empty">Loading…</p>
		<p v-else-if="!rows.length" class="side-empty">Nothing to show.</p>
		<button
			v-for="(row, index) in rows"
			:key="index"
			class="tree-row"
			:class="{ 'tree-row--static': !row.run }"
			:disabled="!row.run"
			:title="row.detail ?? row.label"
			@click="extensionsStore.runRow(row)"
		>
			<AppIcon :name="row.icon ?? entry.view.icon ?? 'package'" :size="15" class="tree-row__icon" />
			<span class="tree-row__label">{{ row.label }}</span>
			<span v-if="row.detail" class="tree-row__meta">{{ row.detail }}</span>
		</button>
	</SideSection>
</template>
