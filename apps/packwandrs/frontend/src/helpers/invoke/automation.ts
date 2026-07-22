import type { AutomationPlan, JobRecord } from '../types'
import { call } from './core'

export const automationPlan = (id: string) => call<AutomationPlan>('automation_plan', { id })
export const automationRun = (id: string, dryRun = true) => call<JobRecord>('automation_run', { id, dryRun })
