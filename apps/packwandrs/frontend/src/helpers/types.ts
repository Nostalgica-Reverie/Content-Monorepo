export interface SerializableError {
	kind: string
	message: string
}

export interface ShellLayout {
	/** Bumped when the shape changes; a mismatch resets to the default. */
	version: 2
	/** Which side of the editor the sidebar occupies. */
	sidebarSide: 'left' | 'right'
	/** Region sizes in pixels. */
	sizes?: Partial<Record<'side' | 'bottom', number>>
}

export interface AppSettings {
	workspacePath: string | null
	javaDefaults: Record<string, string>
	memoryMb: number
	msaClientId: string | null
	rawInputEnabled: boolean
	themeId: string
	/** Collapse UI transitions regardless of the OS `prefers-reduced-motion`. */
	reduceMotion: boolean
	/** `null` means the default arrangement. */
	layout: ShellLayout | null
	/** Whether the shell may be rearranged. Off by default and unsupported. */
	layoutEditing: boolean
	/** Concurrent downloads while installing; `0` follows the machine. */
	downloadJobs: number
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

export type InstanceSource = { kind: 'linked'; packDir: string } | { kind: 'owned' }
export type InstallStage =
	| { state: 'not_installed' }
	| { state: 'installing' }
	| { state: 'ready' }
	| { state: 'failed'; message: string }

export interface InstanceSettings {
	javaPath?: string
	memoryMinMb?: number
	memoryMaxMb?: number
	extraJvmArgs?: string[]
	extraGameArgs?: string[]
	env?: Record<string, string>
	windowWidth?: number
	windowHeight?: number
	fullscreen?: boolean
	/** Concurrent downloads for this instance; omitted inherits the app value. */
	downloadJobs?: number
}

export interface InstanceSummary {
	schemaVersion: number
	id: string
	name: string
	source: InstanceSource
	gameVersion: string
	loader: string
	loaderVersion: string | null
	stage: InstallStage
	settings: InstanceSettings
	createdMs: number
	lastPlayedMs: number | null
	icon: string | null
	group: string | null
}

export interface InstanceContent {
	path: string
	name: string
	enabled: boolean
	packSourced: boolean
	bytes: number
}

/** A CurseForge file the author excluded from third-party distribution; the
 * install finished without it and it needs a human to place by hand. */
export interface PendingManualDownload {
	name: string
	target: string
	pageUrl: string | null
}

export interface CreateInstanceSpec {
	name: string
	source: 'linked' | 'owned'
	packId?: string
	gameVersion?: string
	loader?: string
	loaderVersion?: string
}

export interface InstanceExportResult {
	path: string
	files: number
	bytes: number
	excludedHandAdded: number
}

export interface InstanceFileEntry {
	path: string
	name: string
	directory: boolean
	size: number
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

export interface PublishMatrixEntry {
	manifest: string
	variant: string | null
	order: number
}
export interface PublishArtifact {
	platform: string
	path: string
	exists: boolean
	bytes: number
}
export interface PublishTarget {
	manifestPath: string
	projectRoot: string
	id: string
	name: string
	projectType: string
	variant: string | null
	minecraftVersion: string
	loader: string
	version: string
	releaseType: string
	modrinthId: string | null
	curseforgeId: string | null
	artifacts: PublishArtifact[]
}
export interface SyncJobReport {
	consumer: string
	base: string
	source: string
	target: string
	copied: string[]
	deleted: string[]
	excluded: string[]
}
export interface SyncReport {
	dry_run: boolean
	jobs: SyncJobReport[]
	copied: number
	deleted: number
}
export interface AutomationPlan {
	id: string
	enabled: boolean
	version: string
	nextVersion: string
	tests: string[]
	subdirs: string[]
	steps: string[]
}
export interface ApiRoute {
	method: string
	path: string
	description: string
}
export interface ApiContract {
	transport: string
	version: string
	routes: ApiRoute[]
}

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
export interface ContentRegistry {
	scope: string
	kind: string
	generated_from: string
	entries: Array<Record<string, unknown>>
}

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

/** What the Browse page asks a provider for. Mirrors `packwand_providers::BrowseQuery`. */
export interface BrowseQuery {
	text: string
	loaders: string[]
	gameVersions: string[]
	projectType?: string
	offset: number
	limit: number
}

/** One search result. Mirrors `packwand_providers::BrowseProject`. */
export interface BrowseProject {
	id: string
	slug: string
	title: string
	summary: string
	iconUrl?: string
	author: string
	downloads: number
	loaders: string[]
	gameVersions: string[]
	license?: string
	pageUrl: string
	/** The same project on Legacy CurseForge, when that provider applies. */
	legacyPageUrl?: string
}

export interface BrowsePage {
	projects: BrowseProject[]
	total: number
	offset: number
}

/**
 * A creator, from `packwand_providers::CreatorProfile`.
 *
 * `partial` is load-bearing rather than cosmetic: CurseForge has no user
 * endpoint, so its profiles are reconstructed from search and are missing the
 * avatar, bio, join date, and any project a name search did not surface. The
 * UI must say so instead of rendering an empty-looking profile.
 */
export interface CreatorProfile {
	handle: string
	name: string
	avatarUrl?: string
	bio: string
	joined?: string
	pageUrl?: string
	projects: BrowseProject[]
	partial: boolean
}

export interface GalleryImage {
	url: string
	title: string
	description: string
}

/** One project, with its description already sanitized server-side. */
export interface ProjectPage {
	project: BrowseProject
	body: string
	bodyFormat: 'markdown' | 'html'
	gallery: GalleryImage[]
	sourceUrl?: string
	issuesUrl?: string
	wikiUrl?: string
	discordUrl?: string
	/**
	 * The description as HTML, sanitized in Rust before it reached the webview.
	 * Safe for `v-html` — that is the whole reason it is rendered there.
	 */
	bodyHtml: string
}
