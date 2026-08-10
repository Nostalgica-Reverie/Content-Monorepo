import type { JobRecord } from '../types'
import { call } from './core'

export interface PackeaterMarker {
	path: string
	directory: string
}

export interface PackeaterPreview extends PackeaterMarker {
	enabled: boolean
	output: string
	fileCount: number
	inputBytes: number
}

export const packeaterMarkers = (id: string) => call<PackeaterMarker[]>('packeater_markers', { id })
export const packeaterPreview = (id: string) =>
	call<PackeaterPreview[]>('packeater_preview', { id })
export const packeaterInitialize = (id: string) =>
	call<PackeaterMarker>('packeater_initialize', { id })
export const packeaterRun = (id: string, output?: string) =>
	call<JobRecord>('packeater_run', { id, output: output ?? null })
