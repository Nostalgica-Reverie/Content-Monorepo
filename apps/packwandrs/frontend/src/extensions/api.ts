/**
 * The Packwand extension API.
 *
 * Extensions live in `apps/packwandrs/extensions/<id>/`, are written in
 * TypeScript, and are declared by an `extension.pw.json` beside their entry
 * point. They are first-party and bundled at build time — there is no runtime
 * loader, no sandbox, and no third-party surface.
 *
 * This is deliberately not the VS Code extension API. Extensions contribute
 * to Packwand's command palette, Monaco editor, sidebar, and bottom dock.
 *
 * Analysis belongs in Rust. An extension's job is to invoke a Rust command and
 * present what comes back, never to re-implement a check in TypeScript, which
 * would leave two sources of truth to drift apart.
 */

import type { PackeaterMarker, PackeaterPreview } from '@/helpers/invoke/packeater'
import type { ContentRegistry, DiagnosticIssue, JobRecord, ValidationReport } from '@/helpers/types'

/** Project categories an extension can scope itself to. */
export type ProjectCategory = 'mods' | 'modpacks' | 'datapacks' | 'resourcepacks'

/** Privileges understood by extension API version 1. */
export type ExtensionCapability =
	| 'project.read'
	| 'project.write'
	| 'diagnostics.register'
	| 'external.krita'
	| 'external.blockbench'
	| 'process.approved'
	| 'network.minecraft-metadata'
	| 'export.transform'
	| 'credentials.publish'
	| 'native.optimizer'

export interface ExtensionAsset {
	path: string
	name: string
	kind: string
}

export interface PackGraphNode {
	id: string
	name: string
	path: string
	kind: string
	provider: string
	side: string
}

export interface PackGraphEdge {
	from: string
	to: string
	relation: string
}

export interface PackGraphSnapshot {
	nodes: PackGraphNode[]
	edges: PackGraphEdge[]
}

export interface LanguageFile {
	locale: string
	namespace: string
	path: string
	keys: number
}

export interface LanguageGap {
	locale: string
	namespace: string
	key: string
	referenceLocale: string
}

export interface LanguageSnapshot {
	files: LanguageFile[]
	gaps: LanguageGap[]
}

export interface RecipeAsset extends ExtensionAsset {
	namespace: string
	id: string
}

export interface WorldgenAsset extends ExtensionAsset {
	namespace: string
	id: string
}

/** What the host passes to every extension entry point. */
export interface ExtensionContext {
	editor: {
		/** Opens a pack-relative file in Packwand's Monaco workbench. */
		open: (packId: string, path: string) => void
	}
	/** Fixed, typed operations; extensions never receive arbitrary Tauri invoke. */
	diagnostics: {
		contentLint: (packId?: string) => Promise<ValidationReport>
		parity: () => Promise<Array<Record<string, unknown>>>
		registries: (packId?: string) => Promise<ContentRegistry[]>
	}
	game: {
		recipes: (packId: string) => Promise<RecipeAsset[]>
	}
	graph: {
		snapshot: (packId: string) => Promise<PackGraphSnapshot>
	}
	language: {
		snapshot: (packId: string) => Promise<LanguageSnapshot>
	}
	worldgen: {
		assets: (packId: string) => Promise<WorldgenAsset[]>
	}
	/**
	 * Opens the shell's schema-driven generator, optionally on a given
	 * generator id.
	 *
	 * Views contribute rows rather than markup, so an extension cannot render a
	 * form itself. Asking the shell to open one keeps that boundary intact: the
	 * extension chooses *what* to author, the shell owns *how* it is drawn.
	 */
	generator: {
		open: (generatorId?: string) => void
	}
	optimizer: {
		markers: (packId: string) => Promise<PackeaterMarker[]>
		preview: (packId: string) => Promise<PackeaterPreview[]>
		initialize: (packId: string) => Promise<PackeaterMarker>
		run: (packId: string) => Promise<JobRecord>
	}
	kubejs: {
		scripts: (packId: string) => Promise<ExtensionAsset[]>
		validate: (packId: string) => Promise<ValidationReport>
	}
	krita: {
		assets: (packId: string) => Promise<ExtensionAsset[]>
		open: (packId: string, path: string) => Promise<void>
	}
	blockbench: {
		assets: (packId: string) => Promise<ExtensionAsset[]>
		open: (packId: string, path: string) => Promise<void>
	}
	/** Appends a line to the dock's Output tab. */
	output: (text: string, tone?: 'info' | 'error' | 'success') => void
	/** Publishes issues to the dock's Problems tab, replacing this source's set. */
	publishProblems: (source: string, issues: DiagnosticIssue[]) => void
	/** Raises a toast. */
	notify: (title: string, message: string, tone?: 'info' | 'success' | 'danger') => void
	/** The currently selected project, or null when the workspace is empty. */
	activeProject: () => { id: string; category: ProjectCategory } | null
	/** The currently selected pack target, or null. */
	activePack: () => { id: string; path: string } | null
}

/** A command contributed to the palette (and optionally the status bar). */
export interface ExtensionCommand {
	/** Unique within the extension; namespaced with the extension id by the host. */
	id: string
	/** Shown in the palette. */
	title: string
	/** Palette group heading. Defaults to the extension's display name. */
	group?: string
	/** Icon name understood by AppIcon. */
	icon?: string
	/** Restricts the command to these project categories. */
	when?: ProjectCategory[]
	run: (context: ExtensionContext) => void | Promise<void>
}

/** A panel contributed to the left sidebar. */
export interface ExtensionView {
	id: string
	/** Section title in the sidebar. */
	title: string
	icon?: string
	when?: ProjectCategory[]
	/**
	 * Returns the rows to render. Kept to plain data rather than a component so
	 * extensions cannot reach into the shell's markup, and so a future runtime
	 * loader would not need to ship Vue components.
	 */
	rows: (context: ExtensionContext) => Promise<ExtensionRow[]> | ExtensionRow[]
}

export interface ExtensionRow {
	label: string
	detail?: string
	icon?: string
	/** Invoked when the row is activated. */
	run?: () => void | Promise<void>
}

/** Everything an extension may contribute. */
export interface ExtensionDefinition {
	commands?: ExtensionCommand[]
	views?: ExtensionView[]
	/** Called once when the extension is registered. */
	activate?: (context: ExtensionContext) => void | Promise<void>
	/** Called when the user uninstalls the extension in this Packwand profile. */
	deactivate?: (context: ExtensionContext) => void | Promise<void>
}

/**
 * Declares an extension. The identity (`id`, `name`, `version`) comes from
 * `extension.pw.json`, not from here, so the manifest stays the single place
 * that describes an extension to tooling.
 */
export function definePackwandExtension(definition: ExtensionDefinition): ExtensionDefinition {
	return definition
}

/** An extension's `extension.pw.json`, as authored. */
export interface ExtensionManifest {
	id: string
	name: string
	version: string
	/** Host/SDK contract selected by this extension. Version 1 is current. */
	apiVersion: 1
	description?: string
	/** Entry point relative to the extension directory. */
	entry: string
	/** Events which make the extension applicable, for example project:datapacks. */
	activation: string[]
	/** Declarative contribution ids, checked against the TypeScript definition. */
	commands: string[]
	views: string[]
	validators: string[]
	/** Maximum host authority made available to this extension. */
	capabilities: ExtensionCapability[]
}

/** A manifest paired with its loaded definition. */
export interface LoadedExtension {
	manifest: ExtensionManifest
	definition: ExtensionDefinition
}
