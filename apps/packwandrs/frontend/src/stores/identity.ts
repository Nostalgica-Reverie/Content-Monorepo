import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import {
	accountLogin,
	accountLogout,
	accountWhoami,
	type Identity,
} from '@/helpers/invoke/identity'

/**
 * The ATProto identity used for Packwand social and sharing features.
 *
 * This is distinct from Minecraft/MSA authentication and from provider upload
 * credentials. OAuth tokens remain in the local helper and never cross IPC.
 */
export const useIdentityStore = defineStore('identity', () => {
	const identity = ref<Identity | null>(null)
	const loaded = ref(false)
	const busy = ref(false)
	const signedIn = computed(() => identity.value !== null)

	async function load() {
		if (busy.value) return identity.value
		busy.value = true
		try {
			identity.value = await accountWhoami()
			loaded.value = true
			return identity.value
		} finally {
			busy.value = false
		}
	}

	async function login(identifier: string) {
		busy.value = true
		try {
			identity.value = await accountLogin(identifier)
			loaded.value = true
			return identity.value
		} finally {
			busy.value = false
		}
	}

	async function logout() {
		busy.value = true
		try {
			await accountLogout()
			identity.value = null
			loaded.value = true
		} finally {
			busy.value = false
		}
	}

	return { identity, loaded, busy, signedIn, load, login, logout }
})
