export interface SerializableError {
  kind: string
  message: string
}

export interface AppSettings {
  workspacePath: string | null
  javaDefaults: Record<string, string>
  memoryMb: number
  msaClientId: string | null
  rawInputEnabled: boolean
  themeId: string
}

export interface PackSummary {
  id: string
  name: string
  path: string
  packFormat: string
  version: string
  minecraftVersion: string | null
  loaders: string[]
  indexedFiles: number
  metadataFiles: number
}

export interface PackDetail {
  summary: PackSummary
  pack: Record<string, unknown>
  index: Record<string, unknown>
}

export interface InstanceSummary {
  id: string
  name: string
  path: string
  minecraftVersion: string | null
  loaders: string[]
  kind: 'modpack'
}

export type InstancePhase = 'idle' | 'starting' | 'running' | 'stopped' | 'error'

export interface InstanceStatusPayload {
  id: string
  phase: Exclude<InstancePhase, 'idle'>
  message: string | null
  jobId: string | null
  exitCode: number | null
}

export interface ModSummary {
  metadataPath: string
  name: string
  filename: string
  side: string
  pinned: boolean
  providers: string[]
}

export interface TreeEntry {
  path: string
  name: string
  directory: boolean
  size: number
}

export type JobStatus = 'running' | 'done' | 'failed' | 'cancelled'

export interface JobRecord {
  id: string
  kind: string
  label: string
  status: JobStatus
  fraction: number
  message: string | null
  logs: string[]
  error: SerializableError | null
}

export interface ExportPlan {
  packName: string
  packVersion: string
  outputStem: string
  indexedFiles: number
  metadataFiles: number
}

export interface PublishMatrixEntry { manifest: string; variant: string | null; order: number }
export interface PublishArtifact { platform: string; path: string; exists: boolean; bytes: number }
export interface PublishTarget {
  manifestPath: string; projectRoot: string; id: string; name: string; projectType: string
  variant: string | null; minecraftVersion: string; loader: string; version: string
  releaseType: string; modrinthId: string | null; curseforgeId: string | null; artifacts: PublishArtifact[]
}
export interface SyncJobReport { consumer: string; base: string; source: string; target: string; copied: string[]; deleted: string[]; excluded: string[] }
export interface SyncReport { dry_run: boolean; jobs: SyncJobReport[]; copied: number; deleted: number }
export interface AutomationPlan { id: string; enabled: boolean; version: string; nextVersion: string; tests: string[]; subdirs: string[]; steps: string[] }
export interface ApiRoute { method: string; path: string; description: string }
export interface ApiContract { transport: string; version: string; routes: ApiRoute[] }

export interface ResolvedProject {
  provider: 'modrinth' | 'curse_forge' | 'forgejo' | 'git_hub' | 'git_lab'
  id: string
  slug: string
  title: string
  project_type: string
  side: string
  repository_release: Record<string, unknown> | null
  version: Record<string, unknown>
}

export interface ProjectManifest {
  $schema?: string
  id: string
  name: string
  type: string
  loader?: string
  mc_version?: string
  variants: Array<Record<string, unknown>>
  version: string
  release_type?: string
  description?: string
  modrinth_id?: string
  curseforge_id?: string
  role?: unknown
  lifecycle?: string
  automation?: Record<string, unknown>
  [key: string]: unknown
}

export interface WorkspaceProject {
  category: 'mods' | 'modpacks' | 'datapacks' | 'resourcepacks'
  root: string
  manifest: ProjectManifest
  subdirs: string[]
}

export type ProjectRole = 'none' | 'base' | { consumes: string }

export interface NewProjectRequest {
  category: WorkspaceProject['category']
  id: string
  name: string | null
  minecraft_version: string | null
  loader: string | null
  variants: string[]
  role: ProjectRole
}

export interface DiagnosticIssue {
  severity: 'error' | 'warning'
  path: string
  message: string
}

export interface ValidationReport {
  checked: number
  issues: DiagnosticIssue[]
}

export interface VariantParityReport {
  pack: string
  variant: string
  only_mr?: string[]
  only_cf?: string[]
  file_drift?: string[]
  mr_count: number
  cf_count: number
  missing_side?: 'mr' | 'cf'
}
export interface ContentRegistry { scope: string; kind: string; generated_from: string; entries: Array<Record<string, unknown>> }

export type ProviderKind = 'modrinth' | 'curse_forge' | 'forgejo' | 'git_hub' | 'git_lab'
export interface ResolveRequest {
  project: string
  version_id?: string | null
  version_filename?: string | null
  game_versions: string[]
  loaders: string[]
  channels: Array<'release' | 'beta' | 'alpha'>
  branch: string | null
  asset_pattern: string | null
}
