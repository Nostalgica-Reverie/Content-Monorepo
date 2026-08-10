import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import {
	type AccountProvider,
	type AccountsSnapshot,
	type AccountState,
	accountsLinkCurseforge,
	accountsLinkModrinth,
	accountsSetPublishToken,
	accountsState,
	accountsUnlink,
} from '@/helpers/invoke/accounts'

/**
 * Provider account links — Modrinth and CurseForge.
 *
 * Deliberately separate from `stores/auth`, which despite its name holds
 * Minecraft/MSA launcher identity. They are independent axes: a user can be
 * signed in to Modrinth and still launch the game offline, and collapsing them
 * would make one screen's state changes silently alter the other's.
 *
 * No credential ever reaches this store. The backend returns only whether a
 * link exists and who it belongs to.
 */
export const useAccountsStore = defineStore('accounts', () => {
	const accounts = ref<AccountState[]>([])
	const loaded = ref(false)
	const busy = ref(false)

	function apply(snapshot: AccountsSnapshot) {
		accounts.value = snapshot.accounts
		loaded.value = true
		return snapshot
	}

	function find(provider: AccountProvider) {
		return accounts.value.find((account) => account.provider === provider)
	}

	const modrinth = computed(() => find('modrinth'))
	const curseforge = computed(() => find('curse_forge'))

	/** Publishing needs at least one provider able to receive an upload. */
	const canPublish = computed(() => accounts.value.some((account) => account.canPublish))

	/** Why publishing is unavailable, for a UI that should explain rather than just disable. */
	const publishBlockers = computed(() =>
		accounts.value
			.filter((account) => !account.canPublish)
			.map((account) =>
				account.provider === 'modrinth'
					? 'Modrinth is not connected'
					: 'No CurseForge upload token',
			),
	)

	async function load() {
		if (busy.value) return
		busy.value = true
		try {
			apply(await accountsState())
		} finally {
			busy.value = false
		}
	}

	async function linkModrinth(token: string) {
		return apply(await accountsLinkModrinth(token))
	}

	async function linkCurseforge(apiKey: string) {
		return apply(await accountsLinkCurseforge(apiKey))
	}

	async function setPublishToken(token: string) {
		return apply(await accountsSetPublishToken(token))
	}

	async function unlink(provider: AccountProvider) {
		return apply(await accountsUnlink(provider))
	}

	return {
		accounts,
		loaded,
		busy,
		modrinth,
		curseforge,
		canPublish,
		publishBlockers,
		load,
		linkModrinth,
		linkCurseforge,
		setPublishToken,
		unlink,
	}
})
