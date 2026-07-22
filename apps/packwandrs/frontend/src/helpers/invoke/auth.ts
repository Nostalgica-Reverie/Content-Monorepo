import { call } from './core'

export const authStatus = () => call<unknown>('auth_status')
export const authLogin = () => call<unknown>('auth_login')
export const authLogout = () => call<void>('auth_logout')
