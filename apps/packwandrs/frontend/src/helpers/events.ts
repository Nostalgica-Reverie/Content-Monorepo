import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'

import type { AppSettings, InstanceStatusPayload, JobRecord, JobStatus, SerializableError } from './types'

export interface JobLogPayload { id: string; line: string }
export interface JobProgressPayload { id: string; fraction: number; message: string | null }
export interface JobFinishedPayload { id: string; status: JobStatus; error: SerializableError | null }
export interface JavaInstallProgress { id: string; fraction: number; message: string }
export interface LauncherState { session: string; phase: string; detail?: string }
export interface LauncherLog { session: string; stream: string; line: string }
export interface AuthState { state: string; profile?: unknown }
export interface WebviewEvent { kind: string; payload: unknown }
/** One record drained from the packwandc kernel's trace ring (packwandc.md 3.7). */
export interface KernelTracePayload {
  sequence: number
  tone: 'info' | 'error' | 'success'
  module: string
  message: string
  /** `file:line` in the C tree, already repo-relative. */
  origin: string
  platformCode: number | null
}

const on = <T>(name: string, handler: (payload: T) => void): Promise<UnlistenFn> =>
  listen<T>(name, (event: Event<T>) => handler(event.payload))

export const onJobStarted = (handler: (payload: JobRecord) => void) => on('job:started', handler)
export const onJobLog = (handler: (payload: JobLogPayload) => void) => on('job:log', handler)
export const onJobProgress = (handler: (payload: JobProgressPayload) => void) => on('job:progress', handler)
export const onJobDone = (handler: (payload: JobFinishedPayload) => void) => on('job:done', handler)
export const onJobFailed = (handler: (payload: JobFinishedPayload) => void) => on('job:failed', handler)
export const onPacksChanged = (handler: () => void) => on<void>('packs:changed', handler)
export const onSettingsChanged = (handler: (payload: AppSettings) => void) => on('settings:changed', handler)
export const onInstancesChanged = (handler: () => void) => on<void>('instances:changed', handler)
export const onInstanceStatus = (handler: (payload: InstanceStatusPayload) => void) => on('instance:status', handler)
export const onJavaInstallProgress = (handler: (payload: JavaInstallProgress) => void) => on('java:install-progress', handler)
export const onJavaInstallDone = (handler: (payload: unknown) => void) => on('java:install-done', handler)
export const onLauncherState = (handler: (payload: LauncherState) => void) => on('launcher:state', handler)
export const onLauncherLog = (handler: (payload: LauncherLog) => void) => on('launcher:log', handler)
export const onAuthStatus = (handler: (payload: AuthState) => void) => on('auth:status', handler)
export const onWebviewEvent = (handler: (payload: WebviewEvent) => void) => on('webview:event', handler)
export const onWebviewClosed = (handler: (payload: string) => void) => on('webview:closed', handler)
export const onKernelTrace = (handler: (payload: KernelTracePayload) => void) =>
  on('kernel:trace', handler)
