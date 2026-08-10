import { call } from './core'

export type AccountProvider = 'modrinth' | 'curse_forge'

export interface AccountState {
	provider: AccountProvider
	linked: boolean
	/** Username for Modrinth. Always null for CurseForge, which exposes no user behind an API key. */
	identity: string | null
	canPublish: boolean
}

export interface AccountsSnapshot {
	accounts: AccountState[]
	canPublish: boolean
}

export const accountsState = () => call<AccountsSnapshot>('accounts_state')
export const accountsLinkModrinth = (token: string) =>
	call<AccountsSnapshot>('accounts_link_modrinth', { token })
export const accountsLinkCurseforge = (apiKey: string) =>
	call<AccountsSnapshot>('accounts_link_curseforge', { apiKey })
export const accountsSetPublishToken = (token: string) =>
	call<AccountsSnapshot>('accounts_set_publish_token', { token })
export const accountsUnlink = (provider: AccountProvider) =>
	call<AccountsSnapshot>('accounts_unlink', { provider })
export const accountsPreparePublish = () => call<boolean>('accounts_prepare_publish')
