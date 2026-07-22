import type { InstanceStatusPayload, InstanceSummary, JobRecord } from '../types'
import { call } from './core'

export const instancesList = () => call<InstanceSummary[]>('instances_list')
export const instancesStatusList = () => call<InstanceStatusPayload[]>('instances_status_list')
export const instancesLaunch = (id: string) => call<JobRecord>('instances_launch', { id })
export const instancesStop = (id: string) => call<boolean>('instances_stop', { id })
