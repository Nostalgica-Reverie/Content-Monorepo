import { call } from './core'

export interface StackEntry {
	changeId: string
	commitId: string
	description: string
	isWorkingCopy: boolean
	divergent: boolean
	parentChangeId?: string
}

export const changesEnable = () => call<void>('changes_enable')
export const changesLog = () => call<StackEntry[]>('changes_log')
export const changesNew = (parent?: string) => call<StackEntry>('changes_new', { parent })
export const changesDescribe = (changeId: string, message: string) =>
	call<void>('changes_describe', { changeId, message })
export const changesSquash = (changeId: string) =>
	call<void>('changes_squash', { changeId, intoParent: true })
