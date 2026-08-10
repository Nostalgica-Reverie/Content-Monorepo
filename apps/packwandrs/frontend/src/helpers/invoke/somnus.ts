import type { JobRecord } from '../types'
import { call } from './core'

export interface SomnusWorkflowEntry {
	path: string
	name: string
	trigger: boolean
}

export const somnusRun = (workflow?: string) =>
	call<JobRecord>('somnus_run', { workflow: workflow ?? null })
export const somnusList = () => call<SomnusWorkflowEntry[]>('somnus_list')
