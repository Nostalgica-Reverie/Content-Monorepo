import { computed, ref } from 'vue'

import { instancesLaunch, instancesStop } from '@/helpers/invoke/instances'
import type { InstancePhase } from '@/helpers/types'
import { useInstancesStore } from '@/stores/instances'
import { useToastsStore } from '@/stores/toasts'

/**
 * Per-instance play/stop/status logic, ported from the shape of mrapp's
 * `components/ui/Instance.vue` script (a local `loading` ref layered on top
 * of listener-driven state, and an optimistic `stop()` that updates the UI
 * before the backend round-trip resolves) — adapted to Packwand's
 * job-based launch pipeline instead of Modrinth's dedicated process plugin.
 */
export function useInstanceLaunch(instanceId: string) {
	const store = useInstancesStore()
	const toasts = useToastsStore()
	const loading = ref(false)

	const status = computed(() => store.statuses[instanceId])
	const phase = computed<InstancePhase>(() => status.value?.phase ?? 'idle')
	const message = computed(() => status.value?.message ?? null)
	const playing = computed(() => phase.value === 'running')
	const starting = computed(() => phase.value === 'starting')
	// Mirrors mrapp's `modLoading`: true while waiting on the invoke
	// round-trip *or* the backend has already reported it's mid-launch.
	const modLoading = computed(() => loading.value || starting.value)

	async function play() {
		loading.value = true
		try {
			const job = await instancesLaunch(instanceId)
			toasts.push('Launching', job.label, 'success')
		} catch (error) {
			toasts.push('Launch failed', String(error), 'danger')
		} finally {
			loading.value = false
		}
	}

	async function stop() {
		// Optimistic update — mirrors mrapp's `Instance.vue` `stop()`, which
		// flips `playing.value = false` before `kill()` resolves, so the
		// button responds immediately instead of waiting on a round trip plus
		// the next `instance:status` event.
		const wasActive = phase.value === 'running' || phase.value === 'starting'
		if (status.value) store.apply({ ...status.value, phase: 'stopped', message: 'Stopping…' })
		try {
			const cancelled = await instancesStop(instanceId)
			if (!cancelled && !wasActive) {
				toasts.push('Nothing to stop', 'This instance is not currently running.', 'neutral')
			}
		} catch (error) {
			toasts.push('Stop failed', String(error), 'danger')
		}
	}

	return { status, phase, message, playing, starting, modLoading, loading, play, stop }
}
