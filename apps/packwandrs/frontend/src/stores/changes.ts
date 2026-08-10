import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import {
	changesDescribe,
	changesEnable,
	changesLog,
	changesNew,
	changesSquash,
	type StackEntry,
} from '@/helpers/invoke/changes'

/** Short-lived Jujutsu operations and the currently rendered change stack. */
export const useChangesStore = defineStore('changes', () => {
	const entries = ref<StackEntry[]>([])
	const busy = ref(false)
	const error = ref('')
	const needsEnable = computed(() => error.value.includes('not a Jujutsu workspace'))

	async function run<T>(operation: () => Promise<T>): Promise<T> {
		busy.value = true
		error.value = ''
		try {
			return await operation()
		} catch (caught) {
			error.value = String(caught)
			throw caught
		} finally {
			busy.value = false
		}
	}

	async function refresh() {
		try {
			entries.value = await run(changesLog)
		} catch {
			entries.value = []
		}
	}

	async function enable() {
		await run(changesEnable)
		await refresh()
	}

	async function create(parent?: string) {
		await run(() => changesNew(parent))
		await refresh()
	}

	async function describe(changeId: string, message: string) {
		await run(() => changesDescribe(changeId, message))
		await refresh()
	}

	async function squash(changeId: string) {
		await run(() => changesSquash(changeId))
		await refresh()
	}

	return { entries, busy, error, needsEnable, refresh, enable, create, describe, squash }
})
