import type { SerializableError } from './types'

export function normalizeBridgeError(error: unknown): SerializableError {
	if (typeof error === 'object' && error !== null) {
		const candidate = error as Partial<SerializableError>
		if (typeof candidate.kind === 'string' && typeof candidate.message === 'string') {
			return { kind: candidate.kind, message: candidate.message }
		}
	}
	if (error instanceof Error) return { kind: 'frontend', message: error.message }
	return { kind: 'unknown', message: String(error) }
}
