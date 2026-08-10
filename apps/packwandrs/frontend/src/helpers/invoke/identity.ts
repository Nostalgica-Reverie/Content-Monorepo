import { call } from './core'

export interface Identity {
	did: string
	handle: string
	pds: string
}

export const accountLogin = (identifier: string) => call<Identity>('account_login', { identifier })
export const accountWhoami = () => call<Identity | null>('account_whoami')
export const accountLogout = () => call<void>('account_logout')
