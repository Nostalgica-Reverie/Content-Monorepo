import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import type { DiagnosticIssue } from '@/helpers/types'

export type DockTab = 'problems' | 'output' | 'logs' | 'terminal'
export type SidebarMode = 'explorer' | 'source-control' | 'extensions'
export type OutputTone = 'info' | 'error' | 'success'

export interface ShellTab {
  name: string
  path: string
  label: string
  icon: string
}

export interface OutputLine {
  id: number
  time: string
  text: string
  tone: OutputTone
}

const SIDEBAR_MIN = 190
const SIDEBAR_MAX = 460
const DOCK_MIN = 120
const DOCK_MAX = 620
const OUTPUT_LIMIT = 500
const HISTORY_LIMIT = 100

function stored(key: string, fallback: number) {
  const raw = Number(localStorage.getItem(key))
  return Number.isFinite(raw) && raw > 0 ? raw : fallback
}

function storedFlag(key: string, fallback: boolean) {
  const raw = localStorage.getItem(key)
  return raw === null ? fallback : raw === '1'
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value))
}

/**
 * Shell chrome state: which panels are open, which views are held as tabs, and
 * the diagnostics/output the dock renders. Kept separate from the workbench
 * store so the layout survives workspace reindexing.
 */
export const useShellStore = defineStore('shell', () => {
  const sidebarVisible = ref(storedFlag('packwand:sidebar', true))
  const sidebarWidth = ref(stored('packwand:sidebar-width', 268))
  const sidebarMode = ref<SidebarMode>((localStorage.getItem('packwand:sidebar-mode') as SidebarMode) || 'explorer')
  const dockVisible = ref(storedFlag('packwand:dock', false))
  const dockHeight = ref(stored('packwand:dock-height', 210))
  const dockTab = ref<DockTab>((localStorage.getItem('packwand:dock-tab') as DockTab) || 'problems')

  const tabs = ref<ShellTab[]>([])
  const paletteOpen = ref(false)
  const paletteSeed = ref('')
  /** Bumped to ask the Overview view to open its create-project dialog. */
  const newProjectRequest = ref(0)
  /**
   * The generator an extension asked the shell to open, and a counter so
   * asking twice for the same one still navigates. Extensions contribute rows
   * and commands rather than markup, so opening a form-based view has to go
   * through the shell rather than the extension rendering it itself.
   */
  const generatorRequest = ref<{ id: string; nonce: number }>({ id: '', nonce: 0 })

  /** pw4shell console transcript. Separate from `output`: the terminal is an
   * interactive session with its own scrollback and history, whereas `output`
   * is a log other views append to. Mixing them would make clearing one
   * destroy the other. */
  const terminal = ref<OutputLine[]>([])
  let terminalSeq = 0
  /** Most-recent-first command history, for arrow-key recall. */
  const history = ref<string[]>([])

  const problems = ref<DiagnosticIssue[]>([])
  const problemsSource = ref('')
  const output = ref<OutputLine[]>([])
  let outputSeq = 0

  const errorCount = computed(() => problems.value.filter((issue) => issue.severity === 'error').length)
  const warningCount = computed(() => problems.value.filter((issue) => issue.severity === 'warning').length)

  function toggleSidebar(force?: boolean) {
    sidebarVisible.value = force ?? !sidebarVisible.value
    localStorage.setItem('packwand:sidebar', sidebarVisible.value ? '1' : '0')
  }

  function selectSidebar(mode: SidebarMode) {
    if (sidebarVisible.value && sidebarMode.value === mode) {
      toggleSidebar(false)
      return
    }
    sidebarMode.value = mode
    localStorage.setItem('packwand:sidebar-mode', mode)
    toggleSidebar(true)
  }

  function setSidebarWidth(width: number) {
    sidebarWidth.value = clamp(Math.round(width), SIDEBAR_MIN, SIDEBAR_MAX)
    localStorage.setItem('packwand:sidebar-width', String(sidebarWidth.value))
  }

  function toggleDock(force?: boolean) {
    dockVisible.value = force ?? !dockVisible.value
    localStorage.setItem('packwand:dock', dockVisible.value ? '1' : '0')
  }

  function setDockHeight(height: number) {
    dockHeight.value = clamp(Math.round(height), DOCK_MIN, DOCK_MAX)
    localStorage.setItem('packwand:dock-height', String(dockHeight.value))
  }

  /** Opens the dock on a given tab, or toggles it shut if already showing it. */
  function showDock(tab: DockTab) {
    if (dockVisible.value && dockTab.value === tab) {
      toggleDock(false)
      return
    }
    dockTab.value = tab
    localStorage.setItem('packwand:dock-tab', tab)
    toggleDock(true)
  }

  function selectDockTab(tab: DockTab) {
    dockTab.value = tab
    localStorage.setItem('packwand:dock-tab', tab)
  }

  /** Records a view as an open tab. Re-visiting a view does not duplicate it. */
  function openTab(tab: ShellTab) {
    if (!tabs.value.some((existing) => existing.name === tab.name)) tabs.value.push(tab)
  }

  /** Closes a tab and reports the neighbour to navigate to, if any. */
  function closeTab(name: string): ShellTab | null {
    const index = tabs.value.findIndex((tab) => tab.name === name)
    if (index === -1) return null
    tabs.value.splice(index, 1)
    return tabs.value[index] ?? tabs.value[index - 1] ?? null
  }

  function requestNewProject() {
    newProjectRequest.value += 1
  }

  function requestGenerator(id = '') {
    generatorRequest.value = { id, nonce: generatorRequest.value.nonce + 1 }
  }

  function openPalette(seed = '') {
    paletteSeed.value = seed
    paletteOpen.value = true
  }

  function closePalette() {
    paletteOpen.value = false
  }

  function appendOutput(text: string, tone: OutputTone = 'info') {
    const time = new Date().toLocaleTimeString(undefined, { hour12: false })
    for (const line of text.split('\n')) {
      output.value.push({ id: ++outputSeq, time, text: line, tone })
    }
    if (output.value.length > OUTPUT_LIMIT) output.value.splice(0, output.value.length - OUTPUT_LIMIT)
  }

  function clearOutput() {
    output.value = []
  }

  function appendTerminal(text: string, tone: OutputTone = 'info') {
    const time = new Date().toLocaleTimeString(undefined, { hour12: false })
    for (const line of text.split('\n')) {
      terminal.value.push({ id: ++terminalSeq, time, text: line, tone })
    }
    if (terminal.value.length > OUTPUT_LIMIT) {
      terminal.value.splice(0, terminal.value.length - OUTPUT_LIMIT)
    }
  }

  function clearTerminal() {
    terminal.value = []
  }

  /** Records a command for recall. Repeating the last one does not duplicate it. */
  function rememberCommand(line: string) {
    const trimmed = line.trim()
    if (!trimmed || history.value[0] === trimmed) return
    history.value.unshift(trimmed)
    if (history.value.length > HISTORY_LIMIT) history.value.length = HISTORY_LIMIT
  }

  /** Publishes a diagnostics run into the Problems tab. */
  function setProblems(source: string, issues: DiagnosticIssue[]) {
    problemsSource.value = source
    problems.value = issues
  }

  function clearProblems() {
    problems.value = []
    problemsSource.value = ''
  }

  return {
    sidebarVisible,
    sidebarWidth,
    sidebarMode,
    dockVisible,
    dockHeight,
    dockTab,
    tabs,
    paletteOpen,
    paletteSeed,
    newProjectRequest,
    generatorRequest,
    problems,
    problemsSource,
    output,
    errorCount,
    warningCount,
    toggleSidebar,
    selectSidebar,
    setSidebarWidth,
    toggleDock,
    setDockHeight,
    showDock,
    selectDockTab,
    openTab,
    closeTab,
    requestNewProject,
    requestGenerator,
    openPalette,
    closePalette,
    appendOutput,
    clearOutput,
    terminal,
    history,
    appendTerminal,
    clearTerminal,
    rememberCommand,
    setProblems,
    clearProblems,
  }
})
