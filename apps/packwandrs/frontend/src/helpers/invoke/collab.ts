import type { DiagnosticIssue, JobRecord } from '../types'
import { call } from './core'

export type ParticipantId = number

export interface Participant {
	id: ParticipantId
	displayName: string
	gitName: string
	gitEmail: string
}

export interface SessionInfo {
	packId: string
	packName: string
	allowGitWrite: boolean
}

export interface CollabState {
	role: 'host' | 'guest' | null
	participants: Participant[]
	connection: 'disconnected' | 'connecting' | 'connected'
	session: SessionInfo | null
	allowGitWrite: boolean
}

export interface CollabIdentity {
	displayName: string
	gitName: string
	gitEmail: string
}

export type TextOp =
	| { kind: 'insert'; offset: number; text: string }
	| { kind: 'delete'; offset: number; length: number }

export interface Selection {
	start: number
	end: number
}

export interface ParticipantEvent {
	event: 'joined' | 'left' | 'updated'
	participant: Participant | null
	id: ParticipantId | null
}

export interface PresenceUpdate {
	origin: ParticipantId
	path: string | null
	selections: Selection[]
}

export type DocumentUpdate =
	| { type: 'open'; path: string }
	| { type: 'close'; path: string }
	| { type: 'snapshot'; path: string; revision: number; text: string }
	| {
			type: 'applied'
			path: string
			revision: number
			ops: TextOp[]
			origin: ParticipantId
	  }
	| { type: 'save'; path: string }
	| { type: 'fsChanged'; path: string; kind: 'created' | 'modified' | 'deleted' }

export type CollabOutput =
	| { type: 'output'; channel: string; line: string }
	| { type: 'problems'; snapshot: { source: string; issues: DiagnosticIssue[] } }
	| {
			type: 'jobEvent'
			event: string
			payload: JobRecord | Record<string, unknown>
	  }

export const collabHostStart = (packId: string, allowGitWrite = true) =>
	call<string>('collab_host_start', { packId, allowGitWrite })
export const collabHostStop = () => call<void>('collab_host_stop')
export const collabJoin = (invite: string) => call<CollabState>('collab_join', { invite })
export const collabLeave = () => call<void>('collab_leave')
export const collabState = () => call<CollabState>('collab_state')
export const collabSetIdentity = (displayName: string) =>
	call<CollabIdentity>('collab_set_identity', { displayName })
export const collabSetGitWrite = (allow: boolean) =>
	call<CollabState>('collab_set_git_write', { allow })

export const collabFsRequest = <T>(method: string, parameters: Record<string, unknown>) =>
	call<T>('collab_fs_request', { method, parameters })
export const collabGitRequest = <T>(method: string, parameters: Record<string, unknown>) =>
	call<T>('collab_git_request', { method, parameters })

export const collabDocumentOpen = (path: string) => call<void>('collab_document_open', { path })
export const collabDocumentClose = (path: string) => call<void>('collab_document_close', { path })
export const collabDocumentSnapshot = (path: string, text: string) =>
	call<void>('collab_document_snapshot', { path, text })
export const collabDocumentEdit = (path: string, baseRevision: number, ops: TextOp[]) =>
	call<void>('collab_document_edit', { path, baseRevision, ops })
export const collabDocumentSave = (path: string) => call<void>('collab_document_save', { path })
export const collabPresence = (path: string | null, selections: Selection[]) =>
	call<void>('collab_presence', { path, selections })
export const collabFollow = (target: ParticipantId) => call<void>('collab_follow', { target })

export const collabOutput = (channel: string, line: string) =>
	call<void>('collab_output', { channel, line })
export const collabProblems = (source: string, issues: DiagnosticIssue[]) =>
	call<void>('collab_problems', { snapshot: { source, issues } })
export const collabJobEvent = (event: string, payload: unknown) =>
	call<void>('collab_job_event', { event, payload })
