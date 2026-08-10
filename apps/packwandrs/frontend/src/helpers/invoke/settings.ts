import type { AppSettings } from '../types'
import { call } from './core'

export const settingsGet = () => call<AppSettings>('settings_get')
export const settingsUpdate = (settings: AppSettings) =>
	call<AppSettings>('settings_update', { settings })
