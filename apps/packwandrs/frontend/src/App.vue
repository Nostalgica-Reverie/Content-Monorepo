<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'

import Toast from '@/components/ui/Toast.vue'
import { onInstanceStatus, onJobDone, onJobFailed, onSettingsChanged } from '@/helpers/events'
import { normalizeBridgeError } from '@/helpers/errors'
import { useInstancesStore } from '@/stores/instances'
import { useSettingsStore } from '@/stores/settings'
import { useToastsStore } from '@/stores/toasts'
import { useThemeStore } from '@/stores/theme'
import { useWorkspaceStore } from '@/stores/workspace'

const workspace = useWorkspaceStore()
const settings = useSettingsStore()
const toasts = useToastsStore()
const instances = useInstancesStore()
const theme = useThemeStore()

onMounted(async () => {
	try {
		await Promise.all([workspace.load(), settings.load()])
		await theme.initialize()
	} catch (error) {
		const normalized = normalizeBridgeError(error)
		toasts.push('Could not initialize Packwand', normalized.message, 'danger')
	}
	await onSettingsChanged((next) => {
		settings.value = next
		workspace.path = next.workspacePath
		if (next.themeId !== theme.currentId) theme.applySetting(next.themeId)
	})
	await onJobFailed((job) => {
		toasts.push('Job failed', job.error?.message ?? 'An unknown job error occurred', 'danger')
		void instances.refresh()
	})
	await onJobDone(() => {
		void instances.refresh()
	})
	await instances.hydrate()
	await onInstanceStatus((payload) => {
		instances.apply(payload)
	})
})
</script>

<template>
	<RouterView />
	<TransitionGroup tag="div" name="toast" class="toast-region" aria-live="polite">
		<Toast
			v-for="toast in toasts.items"
			:key="toast.id"
			:toast="toast"
			@dismiss="toasts.dismiss(toast.id)"
		/>
	</TransitionGroup>
</template>
