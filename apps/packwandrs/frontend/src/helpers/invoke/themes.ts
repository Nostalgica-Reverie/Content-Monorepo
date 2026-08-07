import type { PackwandTheme } from '@/themes/types'

import { call } from './core'

export interface StoredTheme {
  fileName: string
  modifiedMs: number
  theme: PackwandTheme | null
  error: string | null
}

export const themesList = () => call<StoredTheme[]>('themes_list')
export const themesSave = (theme: PackwandTheme) => call<PackwandTheme>('themes_save', { theme })
export const themesDelete = (id: string) => call<void>('themes_delete', { id })
