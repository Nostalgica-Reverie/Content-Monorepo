/**
 * Build-time extension discovery.
 *
 * Vite resolves both globs at build time, so the extension set is fixed when the
 * app is compiled. Adding an extension means adding a directory under
 * `apps/packwandrs/extensions/` and rebuilding — there is no runtime scan.
 *
 * Manifests and modules are matched by directory name, and anything malformed is
 * reported rather than skipped: an extension that silently fails to load is far
 * harder to debug than one that says why.
 */

import type { ExtensionDefinition, ExtensionManifest, LoadedExtension } from './api'

const manifests = import.meta.glob<ExtensionManifest>('../../../extensions/*/extension.pw.json', {
  eager: true,
  import: 'default',
})

const modules = import.meta.glob<{ default?: ExtensionDefinition }>(
  '../../../extensions/*/src/index.ts',
  { eager: true },
)

/** Problems found while loading, surfaced by the host rather than thrown. */
export interface ExtensionLoadError {
  directory: string
  message: string
}

const REQUIRED_STRING_FIELDS = ['id', 'name', 'version', 'entry'] as const
const REQUIRED_ARRAY_FIELDS = ['activation', 'commands', 'views', 'validators', 'capabilities'] as const

/**
 * The extension directory a glob result belongs to — the segment right after
 * `extensions/`. Taken from the front rather than the back because manifests and
 * entry points sit at different depths (`<id>/extension.pw.json` versus
 * `<id>/src/index.ts`), and counting from the end yields `src` for the latter.
 */
function directoryOf(path: string): string {
  const segments = path.split('/')
  const anchor = segments.lastIndexOf('extensions')
  return (anchor === -1 ? segments.at(-2) : segments[anchor + 1]) ?? path
}

function validate(directory: string, manifest: unknown): string | null {
  if (!manifest || typeof manifest !== 'object') return 'extension.pw.json is not an object'
  const record = manifest as Record<string, unknown>
  const missing: string[] = REQUIRED_STRING_FIELDS.filter(
    (field) => typeof record[field] !== 'string' || !(record[field] as string).trim(),
  )
  missing.push(...REQUIRED_ARRAY_FIELDS.filter((field) => !Array.isArray(record[field])))
  if (missing.length) return `extension.pw.json is missing ${missing.join(', ')}`
  if (record.apiVersion !== 1) return `unsupported extension apiVersion ${String(record.apiVersion)}`
  if (record.id !== directory) {
    return `extension.pw.json id "${String(record.id)}" does not match its directory "${directory}"`
  }
  return null
}

function load(): { extensions: LoadedExtension[]; errors: ExtensionLoadError[] } {
  const extensions: LoadedExtension[] = []
  const errors: ExtensionLoadError[] = []
  const moduleByDirectory = new Map(
    Object.entries(modules).map(([path, module]) => [directoryOf(path), module]),
  )

  for (const [path, manifest] of Object.entries(manifests)) {
    const directory = directoryOf(path)
    const invalid = validate(directory, manifest)
    if (invalid) {
      errors.push({ directory, message: invalid })
      continue
    }
    const module = moduleByDirectory.get(directory)
    if (!module) {
      errors.push({ directory, message: `no src/index.ts found for ${directory}` })
      continue
    }
    if (!module.default) {
      errors.push({
        directory,
        message: `${directory}/src/index.ts has no default export (use definePackwandExtension)`,
      })
      continue
    }
    const contributionError = validateContributions(manifest, module.default)
    if (contributionError) {
      errors.push({ directory, message: contributionError })
      continue
    }
    extensions.push({ manifest, definition: module.default })
  }

  // Stable order so the palette and sidebar do not reshuffle between builds.
  extensions.sort((left, right) => left.manifest.id.localeCompare(right.manifest.id))
  return { extensions, errors }
}

function validateContributions(
  manifest: ExtensionManifest,
  definition: ExtensionDefinition,
): string | null {
  const checks: Array<[string, string[], string[]]> = [
    ['commands', manifest.commands, (definition.commands ?? []).map((entry) => entry.id)],
    ['views', manifest.views, (definition.views ?? []).map((entry) => entry.id)],
  ]
  for (const [kind, declared, implemented] of checks) {
    const left = [...declared].sort().join(',')
    const right = [...implemented].sort().join(',')
    if (left !== right) return `${kind} declaration does not match src/index.ts`
  }
  return null
}

const loaded = load()

export const extensions: LoadedExtension[] = loaded.extensions
export const extensionErrors: ExtensionLoadError[] = loaded.errors

/** Namespaced command id, so two extensions cannot collide. */
export function qualifiedId(extensionId: string, commandId: string): string {
  return `${extensionId}.${commandId}`
}
