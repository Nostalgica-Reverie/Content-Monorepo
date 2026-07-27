import type { JobRecord } from '../types'
import { call } from './core'

export interface PackeaterMarker {
  path: string
  directory: string
}

export const packeaterMarkers = (id: string) => call<PackeaterMarker[]>('packeater_markers', { id })
export const packeaterRun = (id: string, output?: string) =>
  call<JobRecord>('packeater_run', { id, output: output ?? null })
