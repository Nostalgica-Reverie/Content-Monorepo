<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { computed, nextTick, ref, watch } from 'vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { jobsList } from '@/helpers/invoke/jobs'
import { shellExec } from '@/helpers/invoke/shell'
import { useShellStore } from '@/stores/shell'
import { useWorkbenchStore } from '@/stores/workbench'
import type { DockTab } from '@/stores/shell'

const shell = useShellStore()
const workbench = useWorkbenchStore()
const terminalFolder = computed(
  () => workbench.selectedPack?.path ?? workbench.selectedProject?.root,
)

/* pw4shell console. */

const command = ref('')
const running = ref(false)
const scrollback = ref<HTMLElement | null>(null)
const prompt = ref<HTMLInputElement | null>(null)
/** -1 means "typing a new line"; 0+ indexes into history, most recent first. */
const historyIndex = ref(-1)

async function stickToBottom() {
  await nextTick()
  const el = scrollback.value
  if (el) el.scrollTop = el.scrollHeight
}

async function submit() {
  const line = command.value
  if (running.value) return

  shell.appendTerminal(`> ${line}`)
  shell.rememberCommand(line)
  command.value = ''
  historyIndex.value = -1
  await stickToBottom()

  // A blank line is a no-op in the kernel too, but round-tripping it just to
  // be told nothing happened is wasted IPC.
  if (!line.trim()) return

  running.value = true
  try {
    const result = await shellExec(line, terminalFolder.value)
    for (const output of result.lines) shell.appendTerminal(output.text, output.tone)
  } catch (error) {
    // A thrown error means the native core is unreachable, not that the
    // command was wrong -- the kernel reports bad commands as output.
    shell.appendTerminal(`console unavailable: ${String(error)}`, 'error')
  } finally {
    running.value = false
    await stickToBottom()
  }
}

/** Arrow-key recall. Walks the history without destroying a half-typed line. */
function recall(delta: number) {
  const next = historyIndex.value + delta
  if (next < -1 || next >= shell.history.length) return
  historyIndex.value = next
  command.value = next === -1 ? '' : (shell.history[next] ?? '')
}

watch(
  () => shell.dockTab,
  async (tab) => {
    if (tab !== 'terminal') return
    await nextTick()
    prompt.value?.focus()
    await stickToBottom()
  },
)

/** Jobs poll only while the dock is open on its tab. */
const jobs = useQuery({
  queryKey: ['dock-jobs'],
  queryFn: jobsList,
  refetchInterval: () => (shell.dockVisible && shell.dockTab === 'logs' ? 1000 : false),
})

const runningJobs = computed(() => jobs.data.value?.filter((job) => job.status === 'running') ?? [])
const recentJobs = computed(() => (jobs.data.value ?? []).slice(-40).reverse())

const tabs: { id: DockTab; label: string }[] = [
  { id: 'problems', label: 'Problems' },
  { id: 'output', label: 'Output' },
  { id: 'logs', label: 'Jobs' },
  { id: 'terminal', label: 'Terminal' },
]

function countFor(tab: DockTab) {
  if (tab === 'problems') return shell.problems.length || undefined
  if (tab === 'output') return shell.output.length || undefined
  if (tab === 'terminal') return undefined
  return runningJobs.value.length || undefined
}
</script>

<template>
  <section class="dock" :style="{ height: shell.dockHeight + 'px' }" aria-label="Panel">
    <div class="dock__head">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="dock__tab"
        :class="{ active: shell.dockTab === tab.id }"
        @click="shell.selectDockTab(tab.id)"
      >
        {{ tab.label }}
        <span v-if="countFor(tab.id)" class="dock__tab-count">{{ countFor(tab.id) }}</span>
      </button>
      <div class="dock__actions">
        <button
          v-if="shell.dockTab === 'output'"
          class="icon-btn"
          title="Clear output"
          aria-label="Clear output"
          @click="shell.clearOutput()"
        >
          <AppIcon name="trash" :size="15" />
        </button>
        <button
          v-if="shell.dockTab === 'problems'"
          class="icon-btn"
          title="Clear problems"
          aria-label="Clear problems"
          @click="shell.clearProblems()"
        >
          <AppIcon name="trash" :size="15" />
        </button>
        <button
          v-if="shell.dockTab === 'terminal'"
          class="icon-btn"
          title="Clear terminal"
          aria-label="Clear terminal"
          @click="shell.clearTerminal()"
        >
          <AppIcon name="trash" :size="15" />
        </button>
        <button class="icon-btn" title="Close panel (Ctrl+J)" aria-label="Close panel" @click="shell.toggleDock(false)">
          <AppIcon name="close" :size="15" />
        </button>
      </div>
    </div>

    <div class="dock__body">
      <template v-if="shell.dockTab === 'problems'">
        <p v-if="!shell.problems.length" class="dock-empty">
          No problems reported. Run Validate, Lint, or Preflight to populate this list.
        </p>
        <div
          v-for="(issue, index) in shell.problems"
          :key="index"
          class="problem-row"
          :class="`problem-row--${issue.severity}`"
        >
          <AppIcon :name="issue.severity === 'error' ? 'error' : 'warning'" :size="14" class="problem-row__icon" />
          <span class="problem-row__message">{{ issue.message }}</span>
          <code class="problem-row__path">{{ issue.path }}</code>
        </div>
      </template>

      <template v-else-if="shell.dockTab === 'output'">
        <p v-if="!shell.output.length" class="dock-empty">Command output appears here.</p>
        <div v-else class="dock-output">
          <div
            v-for="line in shell.output"
            :key="line.id"
            class="dock-output__line"
            :class="line.tone !== 'info' ? `dock-output__line--${line.tone}` : undefined"
          >
            <span class="dock-output__time">{{ line.time }}</span>{{ line.text }}
          </div>
        </div>
      </template>

      <template v-else-if="shell.dockTab === 'terminal'">
        <div class="term">
          <div ref="scrollback" class="term__scroll">
            <p v-if="!shell.terminal.length" class="dock-empty">
              pw4shell in <code>{{ terminalFolder ?? 'workspace' }}</code> &mdash;
              type <code>help</code> for kernel commands or <code>packwand --help</code>
              for the full CLI.
            </p>
            <div
              v-for="line in shell.terminal"
              :key="line.id"
              class="dock-output__line"
              :class="line.tone !== 'info' ? `dock-output__line--${line.tone}` : undefined"
            >{{ line.text }}</div>
          </div>
          <form class="term__prompt" @submit.prevent="submit">
            <span class="term__sigil" aria-hidden="true">&gt;</span>
            <input
              ref="prompt"
              v-model="command"
              class="term__input"
              type="text"
              spellcheck="false"
              autocomplete="off"
              autocapitalize="off"
              :disabled="running"
              aria-label="pw4shell command"
              @keydown.up.prevent="recall(1)"
              @keydown.down.prevent="recall(-1)"
            />
          </form>
        </div>
      </template>

      <template v-else>
        <p v-if="!recentJobs.length" class="dock-empty">No background jobs have run in this session.</p>
        <div v-else class="dock-output">
          <div v-for="job in recentJobs" :key="job.id" class="problem-row">
            <AppIcon
              :name="job.status === 'failed' ? 'error' : job.status === 'running' ? 'sync' : 'check'"
              :size="14"
              class="problem-row__icon"
            />
            <span class="problem-row__message">{{ job.label }}</span>
            <ProgressBar
              v-if="job.status === 'running'"
              :value="job.fraction"
              style="width: 120px; margin-left: auto"
            />
            <code v-else class="problem-row__path">{{ job.status }}</code>
          </div>
        </div>
      </template>
    </div>
  </section>
</template>

<style scoped>
/* The terminal owns its own scroll region so the prompt stays pinned at the
   bottom while the transcript scrolls behind it. */
.term {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.term__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  font: 12px/1.6 var(--mono);
}

.term__prompt {
  display: flex;
  align-items: center;
  gap: 6px;
  border-top: 1px solid var(--border);
  padding-top: 6px;
  margin-top: 6px;
}

.term__sigil {
  color: var(--accent);
  font: 12px/1 var(--mono);
}

.term__input {
  flex: 1;
  border: 0;
  background: transparent;
  color: var(--text);
  font: 12px/1.6 var(--mono);
  outline: none;
}

.term__input:disabled {
  opacity: 0.6;
}
</style>
