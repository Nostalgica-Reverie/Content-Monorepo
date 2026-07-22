import type { JobRecord, ModSummary, ResolvedProject } from '../types'
import { call } from './core'

export const modsList = (id: string) => call<ModSummary[]>('mods_list', { id })
export const modAdd = (id: string, metadataPath: string, metadata: Record<string, unknown>, replace = false) => call<void>('mods_add', { id, metadataPath, metadata, replace })
export const modRemove = (id: string, metadataPath: string) => call<JobRecord>('mods_remove', { id, metadataPath })
export const modUpdate = (id: string, metadataPath: string, resolved: ResolvedProject) => call<JobRecord>('mods_update', { id, metadataPath, resolved })
export const modsRefresh = (id: string) => call<JobRecord>('mods_refresh', { id })
export const modPin = (id: string, metadataPath: string, pinned: boolean) => call<void>('mods_pin', { id, metadataPath, pinned })
export const modSideGet = (id: string, metadataPath: string) => call<string>('mods_side_get', { id, metadataPath })
export const modSideSet = (id: string, metadataPath: string, side: string) => call<void>('mods_side_set', { id, metadataPath, side })
