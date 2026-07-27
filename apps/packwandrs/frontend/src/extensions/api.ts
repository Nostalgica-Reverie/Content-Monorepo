/**
 * The Packwand extension API.
 *
 * Extensions live in `apps/packwandrs/extensions/<id>/`, are written in
 * TypeScript, and are declared by an `extension.pw.json` beside their entry
 * point. They are first-party and bundled at build time — there is no runtime
 * loader, no sandbox, and no third-party surface.
 *
 * This is deliberately *not* the VS Code extension API. Extensions contribute to
 * Packwand's own shell — the command palette, the left sidebar, and the bottom
 * dock — because the embedded Code-OSS sidebar is being retired in favour of
 * Packwand's own.
 *
 * Analysis belongs in Rust. An extension's job is to invoke a Rust command and
 * present what comes back, never to re-implement a check in TypeScript, which
 * would leave two sources of truth to drift apart.
 */

import type { DiagnosticIssue } from '@/helpers/types'

/** Project categories an extension can scope itself to. */
export type ProjectCategory = 'mods' | 'modpacks' | 'datapacks' | 'resourcepacks'

/** What the host passes to every extension entry point. */
export interface ExtensionContext {
  /** Invokes a Rust command over the Tauri bridge. */
  invoke: <T>(command: string, args?: Record<string, unknown>) => Promise<T>
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
  run?: (context: ExtensionContext) => void | Promise<void>
}

/** Everything an extension may contribute. */
export interface ExtensionDefinition {
  commands?: ExtensionCommand[]
  views?: ExtensionView[]
  /** Called once when the extension is registered. */
  activate?: (context: ExtensionContext) => void | Promise<void>
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
  description?: string
  /** Entry point relative to the extension directory. */
  entry: string
  /** Restricts the whole extension to these project categories. */
  when?: ProjectCategory[]
}

/** A manifest paired with its loaded definition. */
export interface LoadedExtension {
  manifest: ExtensionManifest
  definition: ExtensionDefinition
}
