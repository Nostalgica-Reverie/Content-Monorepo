import type { PackDetail, PackSummary } from '../types'
import { call } from './core'

export const packsList = () => call<PackSummary[]>('packs_list')
export const packsGet = (id: string) => call<PackDetail>('packs_get', { id })
export const manifestGet = (id: string) => call<Record<string, unknown> | null>('packs_manifest_get', { id })
export const manifestPut = (id: string, manifest: Record<string, unknown>) => call<void>('packs_manifest_put', { id, manifest })
export const changelogGet = (id: string) => call<string>('packs_changelog_get', { id })
export const changelogPut = (id: string, content: string) => call<void>('packs_changelog_put', { id, content })
export const packIcon = (id: string) => call<number[] | null>('packs_icon', { id })
