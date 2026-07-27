import { call } from './core'

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

export const gitStatus = () => call<GitStatus>('git_status')
export const gitStage = (paths: string[]) => call<void>('git_stage', { paths })
export const gitUnstage = (paths: string[]) => call<void>('git_unstage', { paths })
export const gitDiff = (path: string, staged: boolean) => call<string>('git_diff', { path, staged })
export const gitCommit = (message: string) => call<string>('git_commit', { message })
