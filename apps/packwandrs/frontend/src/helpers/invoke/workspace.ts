import { call } from './core'
import type { JobRecord, SyncReport } from '../types'

export const workspaceGet = () => call<string | null>('workspace_get')
export const workspaceSelect = () => call<string | null>('workspace_select')
export const workspaceSet = (path: string) => call<string>('workspace_set', { path })
export const workspaceSyncPreview = () => call<SyncReport>('workspace_sync_preview')
export const workspaceSync = () => call<JobRecord>('workspace_sync')
