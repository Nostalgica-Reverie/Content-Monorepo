<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'

import ActivityRail from '@/components/shell/ActivityRail.vue'
import BottomPanel from '@/components/shell/BottomPanel.vue'
import CommandPalette from '@/components/shell/CommandPalette.vue'
import type { PaletteCommand } from '@/components/shell/CommandPalette.vue'
import SideBar from '@/components/shell/SideBar.vue'
import StatusBar from '@/components/shell/StatusBar.vue'
import TabStrip from '@/components/shell/TabStrip.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import Button from '@/components/ui/Button.vue'
import { apiInspect } from '@/helpers/invoke/api'
import { diagnosticsContentLint, diagnosticsLint, diagnosticsPreflight, diagnosticsValidate } from '@/helpers/invoke/diagnostics'
import { modsRefresh } from '@/helpers/invoke/mods'
import { workspaceSync, workspaceSyncPreview } from '@/helpers/invoke/workspace'
import { onKernelTrace } from '@/helpers/events'
import type { NavItem } from '@/helpers/navigation'
import { endNav, navByName, primaryNav } from '@/helpers/navigation'
import type { SyncReport, ValidationReport } from '@/helpers/types'
import { useExtensionsStore } from '@/stores/extensions'
import { useShellStore } from '@/stores/shell'
import type { ShellTab } from '@/stores/shell'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'
import { useWorkspaceStore } from '@/stores/workspace'

const workbench = useWorkbenchStore()
const workspace = useWorkspaceStore()
const shell = useShellStore()
const extensionsStore = useExtensionsStore()
const toasts = useToastsStore()
const route = useRoute()
const router = useRouter()

const busy = ref('')
const logoUrl = new URL('../../../src-tauri/icons/icon.png', import.meta.url).href
const currentName = computed(() => String(route.name ?? ''))

/** Views that carry the project header and the diagnostics command row. */
const showHeader = computed(() => !['editor'].includes(currentName.value))
const showCommandRow = computed(() => ['overview', 'logs', 'settings'].includes(currentName.value))

/** Torn down on unmount so a hot reload does not stack duplicate listeners. */
let stopKernelTrace: (() => void) | null = null

onMounted(async () => {
  window.addEventListener('keydown', onKeydown)

  // The packwandc kernel records every failure into a fixed ring and never
  // blocks to do it; src-tauri drains that ring and emits each record here.
  // Routing them into the existing output dock keeps the UI contract unchanged
  // (packwandc.md 3.7) rather than inventing a second log surface.
  try {
    stopKernelTrace = await onKernelTrace((record) => {
      const origin = record.platformCode === null ? record.origin : `${record.origin} code ${record.platformCode}`
      shell.appendOutput(`[${record.module}] ${record.message} (${origin})`, record.tone)
    })
  } catch (error) {
    // A missing native core is not fatal to the workbench, so this is a note
    // rather than a toast.
    shell.appendOutput(`Kernel trace unavailable: ${String(error)}`, 'error')
  }

  try {
    await workbench.refresh()
    shell.appendOutput(`Indexed ${workbench.projects.length} projects, ${workbench.packs.length} pack targets.`)
  } catch (error) {
    toasts.push('Could not index workspace', String(error), 'danger')
    shell.appendOutput(`Workspace index failed: ${String(error)}`, 'error')
  }
  // After the index, so an extension's activate() sees a populated workspace.
  await extensionsStore.activate()
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
  stopKernelTrace?.()
})

/** Every visited view is held as a tab so several tasks can stay in flight. */
watch(
  currentName,
  (name) => {
    const item = navByName(name)
    if (item) shell.openTab({ name: item.name, path: item.path, label: item.label, icon: item.icon })
  },
  { immediate: true },
)

/** Sidebar file click: seed the editor with the file and switch to that view. */
function openFileInEditor(path: string) {
  workbench.requestFile(path)
  if (currentName.value !== 'editor') void router.push({ name: 'editor' })
}

function onKeydown(event: KeyboardEvent) {
  const meta = event.ctrlKey || event.metaKey
  if (!meta) return
  const key = event.key.toLowerCase()
  if (key === 'p' && event.shiftKey) {
    event.preventDefault()
    shell.openPalette('')
  } else if (key === 'p') {
    event.preventDefault()
    shell.openPalette('')
  } else if (key === 'b') {
    event.preventDefault()
    shell.toggleSidebar()
  } else if (key === 'j') {
    event.preventDefault()
    shell.toggleDock()
  } else if (key === 'w') {
    event.preventDefault()
    closeTab(currentName.value)
  }
}

function go(item: NavItem) {
  void router.push(item.path)
}

function selectTab(tab: ShellTab) {
  void router.push(tab.path)
}

function closeTab(name: string) {
  const next = shell.closeTab(name)
  if (name !== currentName.value) return
  void router.push(next?.path ?? '/')
}

function isValidationReport(value: unknown): value is ValidationReport {
  return !!value && typeof value === 'object' && Array.isArray((value as ValidationReport).issues)
}

/**
 * Runs a command, narrating it into the Output panel. Commands that return a
 * validation report also publish their issues to Problems and raise the dock,
 * so a failed gate is visible without leaving the current view.
 */
async function run(name: string, work: () => Promise<unknown>, summarize?: (result: unknown) => string) {
  busy.value = name
  shell.appendOutput(`> ${name}`)
  try {
    const result = await work()
    if (isValidationReport(result)) {
      const { issues, checked } = result
      shell.setProblems(name, issues)
      if (issues.length) shell.showDock('problems')
      shell.appendOutput(
        `${name}: ${issues.length} issue(s) across ${checked} checked.`,
        issues.length ? 'error' : 'success',
      )
      toasts.push(
        name,
        issues.length ? `${issues.length} issues found.` : 'All checks passed.',
        issues.length ? 'danger' : 'success',
      )
    } else {
      const summary = summarize?.(result) ?? 'Completed successfully.'
      shell.appendOutput(`${name}: ${summary}`, 'success')
      toasts.push(name, summary, 'success')
    }
  } catch (error) {
    shell.appendOutput(`${name} failed: ${String(error)}`, 'error')
    toasts.push(`${name} failed`, String(error), 'danger')
  } finally {
    busy.value = ''
  }
}

/** Dry sync returns copy/delete counts rather than a validation report. */
function syncSummary(result: unknown) {
  const report = result as SyncReport
  return `${report.jobs?.length ?? 0} job(s), ${report.copied} copied, ${report.deleted} deleted.`
}

const commands = computed<PaletteCommand[]>(() => [
  ...primaryNav.concat(endNav).map((item) => ({
    id: `go:${item.name}`,
    label: `Go to ${item.label}`,
    group: 'Go',
    icon: item.icon,
    run: () => go(item),
  })),
  {
    id: 'run:validate',
    label: 'Validate workspace',
    group: 'Run',
    icon: 'shield',
    run: () => run('Validate workspace', diagnosticsValidate),
  },
  {
    id: 'run:lint',
    label: 'Lint manifests',
    group: 'Run',
    icon: 'shield',
    run: () => run('Lint', diagnosticsLint),
  },
  {
    id: 'run:content-lint',
    label: 'Lint pack content',
    group: 'Run',
    icon: 'shield',
    run: () => run('Content lint', diagnosticsContentLint),
  },
  {
    id: 'run:preflight',
    label: 'Preflight release gate',
    group: 'Run',
    icon: 'check',
    run: () => run('Preflight', diagnosticsPreflight),
  },
  {
    id: 'run:dry-sync',
    label: 'Dry sync (preview base changes)',
    group: 'Run',
    icon: 'sync',
    run: () => run('Dry sync', workspaceSyncPreview, syncSummary),
  },
  {
    id: 'run:apply-sync',
    label: 'Apply workspace sync',
    group: 'Run',
    icon: 'sync',
    run: () => run('Apply sync', workspaceSync),
  },
  {
    id: 'run:health',
    label: 'Check API health',
    group: 'Run',
    icon: 'target',
    run: () => run('Status', () => apiInspect('/health')),
  },
  ...(workbench.selectedPack
    ? [
        {
          id: 'run:refresh-pack',
          label: `Refresh metadata for ${workbench.selectedPack.id}`,
          group: 'Run',
          icon: 'refresh',
          run: () => run('Refresh pack', () => modsRefresh(workbench.selectedPack!.id)),
        },
      ]
    : []),
  {
    id: 'ws:reindex',
    label: 'Reindex workspace',
    group: 'Workspace',
    icon: 'refresh',
    run: () => workbench.refresh(),
  },
  {
    id: 'ws:new-project',
    label: 'New project…',
    group: 'Workspace',
    icon: 'plus',
    run: () => {
      void router.push('/')
      shell.requestNewProject()
    },
  },
  ...workbench.projects.map((project) => ({
    id: `project:${project.manifest.id}`,
    label: project.manifest.name || project.manifest.id,
    group: 'Open project',
    icon: 'folder',
    hint: project.category,
    run: () => workbench.selectProject(project.manifest.id),
  })),
  ...workbench.projectPacks.map((pack) => ({
    id: `pack:${pack.id}`,
    label: pack.name,
    group: 'Open pack target',
    icon: 'target',
    hint: pack.minecraftVersion ?? undefined,
    run: () => workbench.selectPack(pack.id),
  })),
  {
    id: 'view:sidebar',
    label: 'Toggle sidebar',
    group: 'View',
    icon: 'sidebar',
    hint: 'Ctrl+B',
    run: () => shell.toggleSidebar(),
  },
  {
    id: 'view:panel',
    label: 'Toggle panel',
    group: 'View',
    icon: 'panel',
    hint: 'Ctrl+J',
    run: () => shell.toggleDock(),
  },
  {
    id: 'view:problems',
    label: 'Show problems',
    group: 'View',
    icon: 'error',
    run: () => shell.showDock('problems'),
  },
  { id: 'view:output', label: 'Show output', group: 'View', icon: 'logs', run: () => shell.showDock('output') },
  // Extension-contributed commands, grouped under each extension's name so it is
  // clear where a command came from.
  ...extensionsStore.commands.map((entry) => ({
    id: `ext:${entry.id}`,
    label: entry.command.title,
    group: entry.command.group ?? entry.extensionName,
    icon: entry.command.icon ?? 'package',
    run: () => extensionsStore.run(entry.id),
  })),
])

/* --- Drag-to-resize for the sidebar and the bottom panel ----------- */

const sidebarDragging = ref(false)
const dockDragging = ref(false)

function startSidebarDrag(event: PointerEvent) {
  sidebarDragging.value = true
  const startX = event.clientX
  const startWidth = shell.sidebarWidth
  const move = (moved: PointerEvent) => shell.setSidebarWidth(startWidth + (moved.clientX - startX))
  const stop = () => {
    sidebarDragging.value = false
    window.removeEventListener('pointermove', move)
    window.removeEventListener('pointerup', stop)
  }
  window.addEventListener('pointermove', move)
  window.addEventListener('pointerup', stop)
}

function startDockDrag(event: PointerEvent) {
  dockDragging.value = true
  const startY = event.clientY
  const startHeight = shell.dockHeight
  const move = (moved: PointerEvent) => shell.setDockHeight(startHeight - (moved.clientY - startY))
  const stop = () => {
    dockDragging.value = false
    window.removeEventListener('pointermove', move)
    window.removeEventListener('pointerup', stop)
  }
  window.addEventListener('pointermove', move)
  window.addEventListener('pointerup', stop)
}
</script>

<template>
  <div class="workbench" :class="{ 'workbench--side-hidden': !shell.sidebarVisible }" :data-view="currentName">
    <header class="titlebar">
      <div class="titlebar-brand">
        <img :src="logoUrl" alt="" />
        <span>Packwand</span>
      </div>
      <span class="titlebar-sep">—</span>
      <span class="titlebar-workspace" :title="workspace.path ?? ''">{{ workspace.path || 'No workspace' }}</span>
      <button class="command-centre" @click="shell.openPalette('')">
        <AppIcon name="search" :size="13" />
        <span>{{ workbench.title }}</span>
        <kbd>Ctrl+P</kbd>
      </button>
      <div class="titlebar-actions">
        <button class="icon-btn" title="Command palette (Ctrl+Shift+P)" @click="shell.openPalette('')">
          <AppIcon name="palette" :size="15" />
        </button>
      </div>
    </header>

    <ActivityRail
      :items="primaryNav"
      :end-items="endNav"
      :current-name="currentName"
      :current-sidebar="shell.sidebarMode"
      @select="go"
      @select-sidebar="shell.selectSidebar"
    />

    <SideBar
      v-show="shell.sidebarVisible"
      @new-project="shell.requestNewProject()"
      @open-file="openFileInEditor"
    />
    <div
      class="side-resize"
      :class="{ 'side-resize--active': sidebarDragging }"
      role="separator"
      aria-orientation="vertical"
      @pointerdown.prevent="startSidebarDrag"
    />

    <div class="editor-area">
      <TabStrip
        :tabs="shell.tabs"
        :current-name="currentName"
        @select="selectTab"
        @close="closeTab"
        @toggle-dock="shell.toggleDock()"
        @toggle-sidebar="shell.toggleSidebar()"
      />

      <div class="editor-scroll">
        <header v-if="showHeader" class="editor-header">
          <div class="project-heading">
            <h1>{{ workbench.title }}</h1>
            <p>{{ workbench.summary }}</p>
          </div>
          <div class="top-actions">
            <label class="search-wrap">
              <AppIcon name="search" :size="14" />
              <input v-model="workbench.search" type="search" placeholder="Search current project…" />
            </label>
            <Button variant="quiet" :busy="workbench.loading" @click="workbench.refresh()">
              <AppIcon name="refresh" :size="14" /> Refresh
            </Button>
          </div>
        </header>

        <section v-if="showCommandRow" class="toolbar command-toolbar">
          <Button variant="secondary" :busy="busy === 'Validate workspace'" @click="run('Validate workspace', diagnosticsValidate)">
            Validate
          </Button>
          <Button variant="quiet" :busy="busy === 'Preflight'" @click="run('Preflight', diagnosticsPreflight)">
            Preflight
          </Button>
          <Button variant="quiet" :busy="busy === 'Lint'" @click="run('Lint', diagnosticsLint)">Lint</Button>
          <Button variant="quiet" :busy="busy === 'Dry sync'" @click="run('Dry sync', workspaceSyncPreview, syncSummary)">
            Dry sync
          </Button>
        </section>

        <div v-if="workbench.error" class="error-banner">{{ workbench.error }}</div>

        <div class="view-scroll">
          <RouterView />
        </div>

        <div
          v-if="shell.dockVisible"
          class="dock-resize"
          :class="{ 'dock-resize--active': dockDragging }"
          role="separator"
          aria-orientation="horizontal"
          @pointerdown.prevent="startDockDrag"
        />
        <BottomPanel v-if="shell.dockVisible" />
      </div>
    </div>

    <StatusBar />

    <CommandPalette
      :open="shell.paletteOpen"
      :seed="shell.paletteSeed"
      :commands="commands"
      @close="shell.closePalette()"
    />
  </div>
</template>
