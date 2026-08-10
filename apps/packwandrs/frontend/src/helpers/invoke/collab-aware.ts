import { useCollabStore } from '@/stores/collab'

import { collabFsRequest, collabGitRequest } from './collab'
import { call } from './core'

export async function collabAwareEditorCall<T>(
	command: string,
	args: Record<string, unknown> = {},
): Promise<T> {
	const collab = useCollabStore()
	if (collab.isGuest) return collabFsRequest<T>(command, args)
	return call<T>(command, args)
}

export async function collabAwareGitCall<T>(
	command: string,
	args: Record<string, unknown> = {},
	requiresWrite = false,
): Promise<T> {
	const collab = useCollabStore()
	if (!collab.isGuest) return call<T>(command, args)
	if (requiresWrite && !collab.allowGitWrite) {
		throw new Error('The host has disabled guest git writes')
	}
	return collabGitRequest<T>(command, args)
}

export async function hostOnlyGitCall<T>(
	command: string,
	args: Record<string, unknown> = {},
): Promise<T> {
	if (useCollabStore().isGuest) {
		throw new Error(`${command} is available only on the host`)
	}
	return call<T>(command, args)
}
