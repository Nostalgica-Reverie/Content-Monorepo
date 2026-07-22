import type { ExportPlan, JobRecord, PublishMatrixEntry, PublishTarget } from '../types'
import { call } from './core'

export const exportPlan = (id: string) => call<ExportPlan>('exports_publish_plan', { id })
export const exportBuild = (id: string) => call<JobRecord>('exports_build', { id })
export const publishTargets = (id: string) => call<PublishMatrixEntry[]>('exports_publish_targets', { id })
export const publishInspect = (id: string, variant: string | null) => call<PublishTarget>('exports_publish_inspect', { id, variant })
export const publishBuild = (id: string, variant: string | null) => call<JobRecord>('exports_publish_build', { id, variant })
export const publishUpload = (id: string, variant: string | null, live = false) => call<JobRecord>('exports_publish_upload', { id, variant, live })
export const publishVerify = (id: string, variant: string | null) => call<JobRecord>('exports_publish_verify', { id, variant })
