import type { JobRecord } from '../types'
import { call } from './core'

export const jobsList = () => call<JobRecord[]>('jobs_list')
export const jobsGet = (id: string) => call<JobRecord>('jobs_get', { id })
export const jobCancel = (id: string) => call<boolean>('jobs_cancel', { id })
export const startDemoJob = () => call<JobRecord>('jobs_start_demo')
