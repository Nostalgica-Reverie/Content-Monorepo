// The seam between Vue and the Gleam core.
//
// Gleam compiles to plain ES modules with generated `.d.mts` declarations, so
// this file exists only to give those modules import paths and names that read
// naturally from TypeScript. Everything here is a re-export — no logic lives
// in this file, because logic that lives here is logic that is not type-checked
// by the Gleam compiler or covered by `gleam test`.
import * as gleam from '../../core/build/dev/javascript/packwand_frontend_core/packwand_frontend_core.mjs'
import { toList } from '../../core/build/dev/javascript/packwand_frontend_core/gleam.mjs'
import * as gleamExtension from '../../core/build/dev/javascript/packwand_frontend_core/packwand/extension.mjs'
import * as gleamHost from '../../core/build/dev/javascript/packwand_frontend_core/packwand/extension_host.mjs'
import * as gleamInstancing from '../../core/build/dev/javascript/packwand_frontend_core/packwand/instancing.mjs'
import * as gleamShell from '../../core/build/dev/javascript/packwand_frontend_core/packwand/shell.mjs'
import * as gleamWorkbench from '../../core/build/dev/javascript/packwand_frontend_core/packwand/workbench.mjs'
import * as gleamTheme from '../../core/build/dev/javascript/packwand_frontend_core/packwand/theme.mjs'

export const core = gleam
export type CoreModel = gleam.Model$
export type CoreMessage = gleam.Message$
export type CoreEffect = gleam.Effect$

// Theme rules. `validateThemeId` and `validateHexColour` moved from the root
// module into `packwand/theme`; re-exported from their new home so callers did
// not have to change in the same commit that moved them.
export const validateThemeId = gleamTheme.validate_theme_id
export const validateHexColour = gleamTheme.validate_hex_colour
export const isKnownThemeToken = gleamTheme.is_known_token
export const isKnownAppearance = gleamTheme.is_known_appearance
export const isKnownFontStyle = gleamTheme.is_known_font_style
export const meetsContrast = gleamTheme.meets_contrast

/** The canonical application colour tokens, as a plain array. */
export const themeTokenNamesFromCore = (): string[] => gleamTheme.token_names().toArray()

/**
 * WCAG contrast ratio, or `null` when either colour cannot be parsed.
 *
 * Gleam's `Result` crosses the boundary as a tagged object; unwrapping it here
 * keeps `Ok`/`Error` out of the Vue layer.
 */
export function contrastRatio(left: string, right: string): number | null {
  const result = gleamTheme.contrast_ratio(left, right) as { 0?: number } & { isOk?: () => boolean }
  return typeof result.isOk === 'function' && result.isOk() ? (result[0] as number) : null
}

/** The contrast pairs every bundled theme is held to. */
export function contrastRequirements(): Array<{
  foreground: string
  background: string
  label: string
  minimum: number
}> {
  return gleamTheme
    .contrast_requirements()
    .toArray()
    .map(([foreground, background, label, minimum]: [string, string, string, number]) => ({
      foreground,
      background,
      label,
      minimum,
    }))
}

// Extension loading rules.
export const extensionDirectoryOf = gleamExtension.directory_of
export const extensionApiVersion = gleamExtension.api_version
export const requiredExtensionStringFields = (): string[] =>
  gleamExtension.required_string_fields().toArray()
export const requiredExtensionArrayFields = (): string[] =>
  gleamExtension.required_array_fields().toArray()

/**
 * Why a manifest cannot be loaded, or `null` when it can.
 *
 * Unwraps Gleam's `Result` so the host deals in "a message or nothing".
 */
export function extensionManifestProblem(
  directory: string,
  id: string,
  missingFields: string[],
  apiVersion: number,
): string | null {
  const result = gleamExtension.manifest_problem(
    directory,
    id,
    toList(missingFields),
    apiVersion,
  ) as { isOk?: () => boolean; 0?: string }
  return typeof result.isOk === 'function' && result.isOk() ? null : ((result[0] as string) ?? null)
}

// Shell chrome rules.
export const clampSize = gleamShell.clamp as (value: number, min: number, max: number) => number

/** Appends to a bounded log, dropping the oldest entries past `limit`. */
export function pushBounded<T>(lines: T[], line: T, limit: number): T[] {
  return (gleamShell.push_bounded(toList(lines), line, limit) as { toArray(): T[] }).toArray()
}

/**
 * Adds a tab unless one with the same name is already open.
 *
 * The ordering rules live in `packwand/shell.gleam` and operate on names; this
 * maps the resulting names back to the caller's own tab objects.
 */
export function openTabIn<T extends { name: string }>(tabs: T[], tab: T): T[] {
  const names = (gleamShell.open_tab(toList(tabs.map(t => t.name)), tab.name) as {
    toArray(): string[]
  }).toArray()
  const known = new Map([...tabs, tab].map(candidate => [candidate.name, candidate]))
  return names.map(name => known.get(name)!).filter(Boolean)
}

/**
 * Removes a tab and reports which should take focus.
 *
 * The neighbour rule — the tab that slid into the closed one's place, or the
 * one before it when the closed tab was last — lives in
 * `packwand/shell.gleam` and is tested there.
 */
export function closeTabIn<T extends { name: string }>(
  tabs: T[],
  name: string,
): { tabs: T[]; focus: T | null } {
  const [remainingNames, successor] = gleamShell.close_tab(
    toList(tabs.map(tab => tab.name)),
    name,
  ) as [{ toArray(): string[] }, { isOk?: () => boolean; 0?: string }]
  const known = new Map(tabs.map(tab => [tab.name, tab]))
  const focusName =
    typeof successor.isOk === 'function' && successor.isOk() ? (successor[0] as string) : null
  return {
    tabs: remainingNames.toArray().map(candidate => known.get(candidate)!).filter(Boolean),
    focus: focusName ? (known.get(focusName) ?? null) : null,
  }
}

// Workbench scoping rules.
export const normalizeWorkspacePath = gleamWorkbench.normalize_path
export const packBelongsTo = gleamWorkbench.pack_belongs_to
export const selectOrFirst = (candidates: string[], selected: string): string =>
  gleamWorkbench.select_or_first(toList(candidates), selected)
export const firstPresent = (candidates: string[], fallback: string): string =>
  gleamWorkbench.first_present(toList(candidates), fallback)
export const summaryLine = (parts: string[]): string =>
  gleamWorkbench.summary_line(toList(parts))

// Instance presentation/inheritance rules.
export const instanceVersionLabel = gleamInstancing.version_label
export const inheritedPlaceholder = gleamInstancing.inherited_placeholder

// Extension host rules.
export const extensionApplies = (when: string[] | undefined, category: string): boolean =>
  gleamHost.applies(toList(when ?? []), category)
export const extensionActivatedBy = (events: string[], category: string): boolean =>
  gleamHost.activated_by(toList(events), category)
export const reconcileInstalledExtensions = (requested: string[], known: string[]): string[] =>
  (gleamHost.reconcile_installed(toList(requested), toList(known)) as {
    toArray(): string[]
  }).toArray()

/**
 * Normalizes a pack-relative path an extension asked to open.
 *
 * Throws on refusal rather than returning a result: this is a security
 * boundary, and a caller that forgets to check a returned `null` would open
 * the path anyway.
 */
export function safeRelativePath(path: string): string {
  const result = gleamHost.safe_relative_path(path) as {
    isOk?: () => boolean
    0?: string
  }
  if (typeof result.isOk === 'function' && result.isOk()) return result[0] as string
  throw new Error((result[0] as string) ?? 'Invalid pack-relative editor path')
}
