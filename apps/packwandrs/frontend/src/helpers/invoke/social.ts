import { call } from './core'

export interface StrongRef {
	uri: string
	cid: string
}

export interface Friend {
	did: string
	handle: string
	displayName: string
	avatar: string
	pds: string
	sources: string[]
}

export interface PendingInvite {
	from: string
	fromHandle: string
	invite: string
	createdAt: string
	expiresAt: string
	uri: string
	cid: string
}

export interface TangledRepo {
	uri: string
	cid: string
	value: Record<string, unknown>
}

export const socialFriends = () => call<Friend[]>('social_friends')
export const socialPendingInvites = () => call<PendingInvite[]>('social_pending_invites')
export const socialLinkedTangledRepos = () => call<TangledRepo[]>('social_linked_tangled_repos')
export const socialSendInvite = (to: string, invite: string, expiresInMinutes = 60) =>
	call<StrongRef>('social_send_invite', { to, invite, expiresInMinutes })
export const socialSharePack = (packId: string, tangledRepo?: string, gitRemote?: string) =>
	call<StrongRef>('social_share_pack', { packId, tangledRepo, gitRemote })
export const socialShareSnippet = (text: string, language?: string) =>
	call<StrongRef>('social_share_snippet', { text, language })
export const socialShareImage = (path: string, caption?: string, mimeType?: string) =>
	call<StrongRef>('social_share_image', { path, caption, mimeType })
