import { collabAwareGitCall, hostOnlyGitCall } from './collab-aware'

export interface GitChange {
	path: string
	indexStatus: string
	worktreeStatus: string
	staged: boolean
	untracked: boolean
}

export interface GitStatus {
	branch: string
	ahead: number
	behind: number
	changes: GitChange[]
}

export interface GitDiffDocument {
	path: string
	original: string
	modified: string
}

export interface GitRemote {
	name: string
	url: string
}

export interface GitIdentity {
	name: string | null
	email: string | null
}

export interface GitRepository {
	isRepo: boolean
	root: string | null
	branch: string | null
	remotes: GitRemote[]
	identity: GitIdentity
}

export interface GitCommit {
	hash: string
	shortHash: string
	author: string
	email: string
	timestamp: number
	subject: string
}

export interface GitBranches {
	current: string
	local: string[]
	remote: string[]
}

export const gitStatus = () => collabAwareGitCall<GitStatus>('git_status')
export const gitStage = (paths: string[]) => collabAwareGitCall<void>('git_stage', { paths }, true)
export const gitUnstage = (paths: string[]) =>
	collabAwareGitCall<void>('git_unstage', { paths }, true)
export const gitDiff = (path: string, staged: boolean) =>
	collabAwareGitCall<string>('git_diff', { path, staged })
export const gitDiffDocument = (path: string, staged: boolean) =>
	collabAwareGitCall<GitDiffDocument>('git_diff_document', { path, staged })
export const gitCommit = (message: string) =>
	collabAwareGitCall<string>('git_commit', { message }, true)

export const gitRepository = () => collabAwareGitCall<GitRepository>('git_repository')
export const gitInit = () => hostOnlyGitCall<GitRepository>('git_init')
export const gitClone = (url: string, directory: string) =>
	hostOnlyGitCall<string>('git_clone', { url, directory })
export const gitRemoteAdd = (name: string, url: string) =>
	hostOnlyGitCall<GitRemote[]>('git_remote_add', { name, url })
export const gitSetIdentity = (name: string, email: string) =>
	hostOnlyGitCall<GitIdentity>('git_set_identity', { name, email })
export const gitFetch = () => hostOnlyGitCall<void>('git_fetch')
export const gitPull = () => hostOnlyGitCall<string>('git_pull')
export const gitPush = () => hostOnlyGitCall<string>('git_push')
export const gitBranches = () => collabAwareGitCall<GitBranches>('git_branches')
export const gitCheckout = (branch: string) => hostOnlyGitCall<void>('git_checkout', { branch })
export const gitLog = (limit?: number) => collabAwareGitCall<GitCommit[]>('git_log', { limit })
