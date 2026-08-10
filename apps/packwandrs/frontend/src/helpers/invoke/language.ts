import { call } from './core'

export interface EditorDiagnostic {
	path: string
	severity: 'error' | 'warning'
	message: string
	startLine: number
	startColumn: number
	endLine: number
	endColumn: number
}

export interface EditorSymbol {
	id: string
	kind: string
	registry: 'datapack' | 'config' | 'resourcepack' | 'kubejs'
	path: string
	detail: string
}

export interface EditorLanguageSnapshot {
	version: string
	diagnostics: EditorDiagnostic[]
	symbols: EditorSymbol[]
}

export const extensionLanguageSnapshot = (id: string, enabled: string[]) =>
	call<EditorLanguageSnapshot>('extension_language_snapshot', { id, enabled })
