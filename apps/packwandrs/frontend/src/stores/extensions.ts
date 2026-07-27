import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import type {
  ExtensionCommand,
  ExtensionContext,
  ExtensionRow,
  ExtensionView,
  ProjectCategory,
} from '@/extensions/api'
import { extensionErrors, extensions, qualifiedId } from '@/extensions/registry'
import { call } from '@/helpers/invoke/core'
import { useShellStore } from '@/stores/shell'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

/** A command paired with the extension that contributed it. */
export interface RegisteredCommand {
  id: string
  command: ExtensionCommand
  extensionId: string
  extensionName: string
}

/** A view paired with the extension that contributed it. */
export interface RegisteredView {
  id: string
  view: ExtensionView
  extensionId: string
}

/**
 * Hosts the bundled extensions: builds the context they receive, and exposes
 * their contributions for the shell to merge into the palette, sidebar and dock.
 *
 * Extensions are first-party and compiled in, so there is no sandbox. What this
 * store does guarantee is *isolation of failure*: one extension throwing during
 * activation must not stop the others from loading, or take the app down.
 */
export const useExtensionsStore = defineStore('extensions', () => {
  const shell = useShellStore()
  const toasts = useToastsStore()
  const workbench = useWorkbenchStore()

  /** Load-time problems, plus any activation failure. */
  const errors = ref<string[]>(
    extensionErrors.map((error) => `${error.directory}: ${error.message}`),
  )
  const activated = ref(false)

  const context: ExtensionContext = {
    invoke: (command, args) => call(command, args),
    output: (text, tone = 'info') => shell.appendOutput(text, tone),
    // Raising the dock on a non-empty result matches what the built-in
    // diagnostics commands do; without it an extension reports "2 issues found"
    // in a toast and the list stays hidden.
    publishProblems: (source, issues) => {
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

  /** The active project's category, used to evaluate `when` clauses. */
  const category = computed<ProjectCategory | null>(
    () => (workbench.selectedProject?.category as ProjectCategory | undefined) ?? null,
  )

  function applies(when: ProjectCategory[] | undefined) {
    if (!when?.length) return true
    return category.value !== null && when.includes(category.value)
  }

  /** Commands whose extension and own `when` clause both admit the current project. */
  const commands = computed<RegisteredCommand[]>(() =>
    extensions
      .filter((entry) => applies(entry.manifest.when))
      .flatMap((entry) =>
        (entry.definition.commands ?? [])
          .filter((command) => applies(command.when))
          .map((command) => ({
            id: qualifiedId(entry.manifest.id, command.id),
            command,
            extensionId: entry.manifest.id,
            extensionName: entry.manifest.name,
          })),
      ),
  )

  const views = computed<RegisteredView[]>(() =>
    extensions
      .filter((entry) => applies(entry.manifest.when))
      .flatMap((entry) =>
        (entry.definition.views ?? [])
          .filter((view) => applies(view.when))
          .map((view) => ({
            id: qualifiedId(entry.manifest.id, view.id),
            view,
            extensionId: entry.manifest.id,
          })),
      ),
  )

  /** Runs every extension's `activate`, once, isolating failures. */
  async function activate() {
    if (activated.value) return
    activated.value = true
    for (const entry of extensions) {
      if (!entry.definition.activate) continue
      try {
        await entry.definition.activate(context)
      } catch (error) {
        const message = `${entry.manifest.id} failed to activate: ${String(error)}`
        errors.value.push(message)
        shell.appendOutput(message, 'error')
      }
    }
    for (const error of errors.value) shell.appendOutput(`Extension error — ${error}`, 'error')
  }

  /** Invokes a contributed command, reporting rather than propagating failure. */
  async function run(id: string) {
    const entry = commands.value.find((candidate) => candidate.id === id)
    if (!entry) return
    try {
      await entry.command.run(context)
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
      return await entry.view.rows(context)
    } catch (error) {
      return [{ label: 'Could not load', detail: String(error) }]
    }
  }

  /** Activates a row's action, if it has one. */
  async function runRow(row: ExtensionRow) {
    if (!row.run) return
    try {
      await row.run(context)
    } catch (error) {
      shell.appendOutput(`${row.label} failed: ${String(error)}`, 'error')
      toasts.push(row.label, String(error), 'danger')
    }
  }

  return {
    manifests: extensions.map((entry) => entry.manifest),
    errors,
    commands,
    views,
    activate,
    run,
    rowsFor,
    runRow,
  }
})
