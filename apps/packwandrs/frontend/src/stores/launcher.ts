import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import type { LauncherState } from '@/helpers/events'

export const useLauncherStore = defineStore('launcher', () => {
	const sessions = ref<Record<string, LauncherState>>({})
	const active = computed(() =>
		Object.values(sessions.value).filter(
			(session) => !['exited', 'failed', 'cancelled'].includes(session.phase),
		),
	)

	function update(payload: LauncherState) {
		sessions.value[payload.session] = payload
	}

	return { sessions, active, update }
})
