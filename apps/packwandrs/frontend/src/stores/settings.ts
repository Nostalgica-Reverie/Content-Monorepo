import { defineStore } from 'pinia'
import { ref } from 'vue'

import { settingsGet, settingsUpdate } from '@/helpers/invoke/settings'
import type { AppSettings } from '@/helpers/types'

export const useSettingsStore = defineStore('settings', () => {
	const value = ref<AppSettings | null>(null)

	async function load() {
		value.value = await settingsGet()
		return value.value
	}

	async function save(settings: AppSettings) {
		value.value = await settingsUpdate(settings)
		return value.value
	}

	return { value, load, save }
})
