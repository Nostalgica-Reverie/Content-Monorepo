import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const useAuthStore = defineStore('auth', () => {
	const state = ref<'offline' | 'signed_in' | 'unavailable'>('offline')
	const profile = ref<unknown>(null)
	const label = computed(() => (state.value === 'signed_in' ? 'Microsoft account' : 'Offline mode'))
	return { state, profile, label }
})
