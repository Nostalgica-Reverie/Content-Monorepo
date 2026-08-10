import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
	socialFriends,
	socialLinkedTangledRepos,
	socialPendingInvites,
	socialSendInvite,
	socialShareImage,
	socialSharePack,
	socialShareSnippet,
	type Friend,
	type PendingInvite,
	type StrongRef,
	type TangledRepo,
} from '@/helpers/invoke/social'

/** ATProto-backed discovery, invitations, and content sharing. */
export const useSocialStore = defineStore('social', () => {
	const friends = ref<Friend[]>([])
	const pendingInvites = ref<PendingInvite[]>([])
	const tangledRepos = ref<TangledRepo[]>([])
	const loaded = ref(false)
	const busy = ref(false)

	async function refresh() {
		if (busy.value) return
		busy.value = true
		try {
			const [nextFriends, nextInvites, nextRepos] = await Promise.all([
				socialFriends(),
				socialPendingInvites(),
				socialLinkedTangledRepos(),
			])
			friends.value = nextFriends
			pendingInvites.value = nextInvites
			tangledRepos.value = nextRepos
			loaded.value = true
		} finally {
			busy.value = false
		}
	}

	async function sendInvite(to: string, invite: string, expiresInMinutes = 60) {
		return socialSendInvite(to, invite, expiresInMinutes)
	}

	async function sharePack(
		packId: string,
		tangledRepo?: string,
		gitRemote?: string,
	): Promise<StrongRef> {
		return socialSharePack(packId, tangledRepo, gitRemote)
	}

	async function shareSnippet(text: string, language?: string): Promise<StrongRef> {
		return socialShareSnippet(text, language)
	}

	async function shareImage(path: string, caption?: string, mimeType?: string): Promise<StrongRef> {
		return socialShareImage(path, caption, mimeType)
	}

	function dismissInvite(uri: string) {
		pendingInvites.value = pendingInvites.value.filter((invite) => invite.uri !== uri)
	}

	function clear() {
		friends.value = []
		pendingInvites.value = []
		tangledRepos.value = []
		loaded.value = false
	}

	return {
		friends,
		pendingInvites,
		tangledRepos,
		loaded,
		busy,
		refresh,
		sendInvite,
		sharePack,
		shareSnippet,
		shareImage,
		dismissInvite,
		clear,
	}
})
