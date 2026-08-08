import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import type {
  ExtensionCommand,
  ExtensionCapability,
  ExtensionContext,
  ExtensionManifest,
  ExtensionAsset,
  LanguageSnapshot,
  PackGraphSnapshot,
  RecipeAsset,
  WorldgenAsset,
  ExtensionRow,
  ExtensionView,
  ProjectCategory,
} from '@/extensions/api'
import {
  extensionActivatedBy,
  extensionApplies,
  reconcileInstalledExtensions,
  safeRelativePath,
} from '@/core/packwand'
import { extensionErrors, extensions, qualifiedId } from '@/extensions/registry'
import { call } from '@/helpers/invoke/core'
import type { PackeaterMarker, PackeaterPreview } from '@/helpers/invoke/packeater'
import type { ContentRegistry, JobRecord, ValidationReport } from '@/helpers/types'
import { useShellStore } from '@/stores/shell'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

/** A command paired with the extension that contributed it. */
export interface RegisteredCommand {
  id: string
  command: ExtensionCommand
  extensionId: string
  extensionName: string
  context: ExtensionContext
}

/** A view paired with the extension that contributed it. */
export interface RegisteredView {
  id: string
  view: ExtensionView
  extensionId: string
  context: ExtensionContext
}

/**
 * Hosts the bundled extensions: builds the context they receive, and exposes
 * their contributions for the shell to merge into the palette, sidebar and dock.
 *
 * Extensions are first-party and compiled in. They receive a typed,
 * capability-checked facade rather than raw Tauri invoke access. One extension
 * throwing during activation must not stop the others from loading.
 */
export const useExtensionsStore = defineStore('extensions', () => {
  const shell = useShellStore()
  const toasts = useToastsStore()
  const workbench = useWorkbenchStore()

  /** Load-time problems, plus any activation failure. */
  const errors = ref<string[]>(
    extensionErrors.map((error) => `${error.directory}: ${error.message}`),
  )
  const installedIds = ref<string[]>(readInstalledExtensions())
  const hostActivated = ref(false)
  const activatedIds = new Set<string>()

  function readInstalledExtensions(): string[] {
    try {
      const value = JSON.parse(localStorage.getItem('packwand:installed-extensions') ?? '[]')
      if (!Array.isArray(value)) return []
      const requested = value.filter((id): id is string => typeof id === 'string')
      // Implied extensions, unknown-id filtering, de-duplication and ordering
      // all live in `packwand/extension_host.gleam`.
      const installed = reconcileInstalledExtensions(
        requested,
        extensions.map(entry => entry.manifest.id),
      )
      localStorage.setItem('packwand:installed-extensions', JSON.stringify(installed))
      return installed
    } catch {
      return []
    }
  }

  function persistInstalledExtensions() {
    localStorage.setItem('packwand:installed-extensions', JSON.stringify(installedIds.value))
  }

  function isInstalled(id: string) {
    return installedIds.value.includes(id)
  }

  function contextFor(manifest: ExtensionManifest): ExtensionContext {
    const requireCapability = (capability: ExtensionCapability) => {
      if (!manifest.capabilities.includes(capability)) {
        throw new Error(`${manifest.id} did not declare capability ${capability}`)
      }
    }
    return {
      editor: {
        open: (packId: string, path: string) => {
          requireCapability('project.read')
          if (workbench.selectedPack?.id !== packId) throw new Error('The requested pack is not active')
          // Security boundary — see `extension_host.safe_relative_path`, which
          // refuses traversal and absolute paths and is tested against both.
          // Note it *rejects* a leading slash rather than stripping it: an
          // extension asking for `/etc/passwd` has made a mistake worth
          // surfacing, not one worth silently reinterpreting as a relative path.
          workbench.requestFile(safeRelativePath(path))
        },
      },
      generator: {
        open: (generatorId?: string) => {
          requireCapability('project.read')
          shell.requestGenerator(generatorId ?? '')
        },
      },
      diagnostics: {
      contentLint: (packId?: string) => {
        requireCapability('project.read')
        return packId
          ? call<ValidationReport>('extension_content_lint', { id: packId })
          : call<ValidationReport>('diagnostics_content_lint')
      },
      parity: () => {
        requireCapability('project.read')
        return call<Array<Record<string, unknown>>>('diagnostics_parity')
      },
      registries: (packId?: string) => {
        requireCapability('project.read')
        return packId
          ? call<ContentRegistry[]>('extension_registries', { id: packId })
          : call<ContentRegistry[]>('diagnostics_registries')
      },
    },
      game: {
        recipes: (packId: string) => {
          requireCapability('project.read')
          return call<RecipeAsset[]>('extension_recipes', { id: packId })
        },
      },
      graph: {
        snapshot: (packId: string) => {
          requireCapability('project.read')
          return call<PackGraphSnapshot>('extension_pack_graph', { id: packId })
        },
      },
      language: {
        snapshot: (packId: string) => {
          requireCapability('project.read')
          return call<LanguageSnapshot>('extension_language_files', { id: packId })
        },
      },
      worldgen: {
        assets: (packId: string) => {
          requireCapability('project.read')
          return call<WorldgenAsset[]>('extension_worldgen_assets', { id: packId })
        },
      },
      optimizer: {
      markers: (packId: string) => {
        requireCapability('native.optimizer')
        return call<PackeaterMarker[]>('packeater_markers', { id: packId })
      },
      preview: (packId: string) => {
        requireCapability('native.optimizer')
        return call<PackeaterPreview[]>('packeater_preview', { id: packId })
      },
      initialize: (packId: string) => {
        requireCapability('project.write')
        requireCapability('native.optimizer')
        return call<PackeaterMarker>('packeater_initialize', { id: packId })
      },
      run: (packId: string) => {
        requireCapability('native.optimizer')
        return call<JobRecord>('packeater_run', { id: packId, output: null })
      },
    },
      kubejs: {
        scripts: (packId: string) => {
          requireCapability('project.read')
          return call<ExtensionAsset[]>('extension_kubejs_scripts', { id: packId })
        },
        validate: (packId: string) => {
          requireCapability('project.read')
          return call<ValidationReport>('extension_kubejs_validate', { id: packId })
        },
      },
      krita: {
        assets: (packId: string) => {
          requireCapability('project.read')
          return call<ExtensionAsset[]>('extension_krita_assets', { id: packId })
        },
        open: (packId: string, path: string) => {
          requireCapability('external.krita')
          return call<void>('extension_krita_open', { id: packId, path })
        },
      },
      blockbench: {
        assets: (packId: string) => {
          requireCapability('project.read')
          return call<ExtensionAsset[]>('extension_blockbench_assets', { id: packId })
        },
        open: (packId: string, path: string) => {
          requireCapability('external.blockbench')
          return call<void>('extension_blockbench_open', { id: packId, path })
        },
      },
      output: (text, tone = 'info') => shell.appendOutput(text, tone),
    // Raising the dock on a non-empty result matches what the built-in
    // diagnostics commands do; without it an extension reports "2 issues found"
    // in a toast and the list stays hidden.
      publishProblems: (source, issues) => {
      requireCapability('diagnostics.register')
      shell.setProblems(source, issues)
      if (issues.length) shell.showDock('problems')
    },
      notify: (title, message, tone = 'info') =>
        toasts.push(title, message, tone === 'info' ? 'success' : tone),
      activeProject: () => {
      const project = workbench.selectedProject
      return project
        ? { id: project.manifest.id, category: project.category as ProjectCategory }
        : null
    },
      activePack: () => {
      const pack = workbench.selectedPack
      return pack ? { id: pack.id, path: pack.path } : null
    },
    }
  }

  /** The active project's category, used to evaluate `when` clauses. */
  const category = computed<ProjectCategory | null>(
    () => (workbench.selectedProject?.category as ProjectCategory | undefined) ?? null,
  )

  // Activation rules live in `packwand/extension_host.gleam`; `null` category
  // crosses as an empty string, which those rules treat as "no project open".
  function applies(when: ProjectCategory[] | undefined) {
    return extensionApplies(when, category.value ?? '')
  }

  function activatedBy(events: string[]) {
    return extensionActivatedBy(events, category.value ?? '')
  }

  /** Commands whose extension and own `when` clause both admit the current project. */
  const commands = computed<RegisteredCommand[]>(() =>
    extensions
      .filter((entry) => isInstalled(entry.manifest.id))
      .filter((entry) => activatedBy(entry.manifest.activation))
      .flatMap((entry) =>
        (entry.definition.commands ?? [])
          .filter((command) => applies(command.when))
          .map((command) => ({
            id: qualifiedId(entry.manifest.id, command.id),
            command,
            extensionId: entry.manifest.id,
            extensionName: entry.manifest.name,
            context: contextFor(entry.manifest),
          })),
      ),
  )

  const views = computed<RegisteredView[]>(() =>
    extensions
      .filter((entry) => isInstalled(entry.manifest.id))
      .filter((entry) => activatedBy(entry.manifest.activation))
      .flatMap((entry) =>
        (entry.definition.views ?? [])
          .filter((view) => applies(view.when))
          .map((view) => ({
            id: qualifiedId(entry.manifest.id, view.id),
            view,
            extensionId: entry.manifest.id,
            context: contextFor(entry.manifest),
          })),
      ),
  )

  /** Runs every extension's `activate`, once, isolating failures. */
  async function activate() {
    if (hostActivated.value) return
    hostActivated.value = true
    for (const entry of extensions.filter((candidate) => isInstalled(candidate.manifest.id))) {
      if (activatedIds.has(entry.manifest.id)) continue
      activatedIds.add(entry.manifest.id)
      if (!entry.definition.activate) continue
      try {
        await entry.definition.activate(contextFor(entry.manifest))
      } catch (error) {
        const message = `${entry.manifest.id} failed to activate: ${String(error)}`
        errors.value.push(message)
        shell.appendOutput(message, 'error')
      }
    }
    for (const error of errors.value) shell.appendOutput(`Extension error — ${error}`, 'error')
  }

  async function install(id: string) {
    const entry = extensions.find((candidate) => candidate.manifest.id === id)
    if (!entry) throw new Error(`Unknown extension ${id}`)
    if (isInstalled(id)) return
    installedIds.value = [...installedIds.value, id].sort()
    persistInstalledExtensions()
    if (hostActivated.value && !activatedIds.has(id)) {
      activatedIds.add(id)
      if (entry.definition.activate) await entry.definition.activate(contextFor(entry.manifest))
    }
    shell.appendOutput(`Installed extension ${entry.manifest.name}.`, 'success')
  }

  async function uninstall(id: string) {
    const entry = extensions.find((candidate) => candidate.manifest.id === id)
    if (!entry || !isInstalled(id)) return
    installedIds.value = installedIds.value.filter((installed) => installed !== id)
    persistInstalledExtensions()
    if (activatedIds.delete(id) && entry.definition.deactivate) {
      try {
        await entry.definition.deactivate(contextFor(entry.manifest))
      } catch (error) {
        const message = `${entry.manifest.id} failed to deactivate: ${String(error)}`
        errors.value.push(message)
        shell.appendOutput(message, 'error')
      }
    }
    shell.appendOutput(`Uninstalled extension ${entry.manifest.name}.`)
  }

  /** Invokes a contributed command, reporting rather than propagating failure. */
  async function run(id: string) {
    const entry = commands.value.find((candidate) => candidate.id === id)
    if (!entry) return
    try {
      await entry.command.run(entry.context)
    } catch (error) {
      const message = `${id} failed: ${String(error)}`
      shell.appendOutput(message, 'error')
      toasts.push(entry.extensionName, String(error), 'danger')
    }
  }

  /** Resolves a view's rows, yielding an error row rather than throwing. */
  async function rowsFor(id: string): Promise<ExtensionRow[]> {
    const entry = views.value.find((candidate) => candidate.id === id)
    if (!entry) return []
    try {
      return await entry.view.rows(entry.context)
    } catch (error) {
      return [{ label: 'Could not load', detail: String(error) }]
    }
  }

  /** Activates a row's action, if it has one. */
  async function runRow(row: ExtensionRow) {
    if (!row.run) return
    try {
      await row.run()
    } catch (error) {
      shell.appendOutput(`${row.label} failed: ${String(error)}`, 'error')
      toasts.push(row.label, String(error), 'danger')
    }
  }

  return {
    manifests: extensions.map((entry) => entry.manifest),
    installedIds,
    isInstalled,
    install,
    uninstall,
    errors,
    commands,
    views,
    activate,
    run,
    rowsFor,
    runRow,
  }
})
