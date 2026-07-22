import { call } from './core'

export const launcherLaunch = (instance: string) => call<string>('launcher_launch', { instance })
export const launcherCancel = (session: string) => call<void>('launcher_cancel', { session })
export const launcherSessionsList = () => call<unknown[]>('launcher_sessions_list')
