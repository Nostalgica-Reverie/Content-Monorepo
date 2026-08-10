<script setup lang="ts">
import { ref, watch } from 'vue'
import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import { changelogGet, changelogPut } from '@/helpers/invoke/packs'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

const workbench = useWorkbenchStore()
const toasts = useToastsStore()
const content = ref('')
const saved = ref('')
const loading = ref(false)
const saving = ref(false)

async function load() {
	if (!workbench.selectedPack) {
		content.value = ''
		saved.value = ''
		return
	}
	loading.value = true
	try {
		content.value = await changelogGet(workbench.selectedPack.id)
		saved.value = content.value
	} catch (error) {
		toasts.push('Could not open changelog', String(error), 'danger')
	} finally {
		loading.value = false
	}
}
async function save() {
	if (!workbench.selectedPack) return
	saving.value = true
	try {
		await changelogPut(workbench.selectedPack.id, content.value)
		saved.value = content.value
		toasts.push('Changelog saved', workbench.selectedPack.id, 'success')
	} catch (error) {
		toasts.push('Could not save changelog', String(error), 'danger')
	} finally {
		saving.value = false
	}
}
async function copy() {
	await navigator.clipboard.writeText(content.value)
	toasts.push('Copied', 'Changelog copied to the clipboard.', 'success')
}
watch(() => workbench.selectedPack?.id, load, { immediate: true })
</script>

<template>
	<section class="grid view-grid">
		<div class="panel span-12 changelog-panel">
			<div class="panel-head">
				<div>
					<h2>Changelog</h2>
					<p class="panel-copy">Edit the release notes for the active pack target.</p>
				</div>
				<div class="panel-actions">
					<Button variant="quiet" @click="copy">Copy</Button
					><Button :busy="saving" :disabled="content === saved" @click="save"
						>Save changelog</Button
					>
				</div>
			</div>
			<EmptyState
				v-if="!workbench.selectedPack"
				title="No pack target"
				message="Select a project with at least one pack target."
			/>
			<div v-else class="changelog-workbench">
				<div class="editor-tabs">
					<span class="editor-tab active">changelog.md</span
					><span v-if="content !== saved" class="modified-dot">●</span>
				</div>
				<textarea
					v-model="content"
					:disabled="loading"
					class="changelog-editor"
					spellcheck="true"
				/>
			</div>
		</div>
	</section>
</template>
