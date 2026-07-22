import type { NewProjectRequest, ProjectManifest, WorkspaceProject } from '../types'
import { call } from './core'

export const projectsList = () => call<WorkspaceProject[]>('projects_list')
export const projectsGet = (id: string) => call<WorkspaceProject>('projects_get', { id })
export const projectsCreate = (request: NewProjectRequest) => call<WorkspaceProject>('projects_create', { request })
export const projectManifestUpdate = (id: string, manifest: ProjectManifest) =>
  call<ProjectManifest>('projects_manifest_update', { id, manifest })
export const projectBump = (id: string, version: string) =>
  call<[string, string]>('projects_bump', { id, version })
export const projectFreeze = (id: string, subdir: string, slugs: string[], frozen: boolean) =>
  call<string[]>('projects_freeze', { id, subdir, slugs, frozen })
