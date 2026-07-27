<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import SideSection from '@/components/shell/SideSection.vue'
import { gitCommit, gitDiff, gitStage, gitStatus, gitUnstage } from '@/helpers/invoke/git'
import type { GitChange, GitStatus } from '@/helpers/invoke/git'
import { useShellStore } from '@/stores/shell'

const shell = useShellStore()
const status = ref<GitStatus | null>(null)
const message = ref('')
const loading = ref(false)
const error = ref('')

const staged = computed(() => status.value?.changes.filter((change) => change.staged) ?? [])
const unstaged = computed(
  () => status.value?.changes.filter((change) => change.untracked || change.worktreeStatus !== ' ') ?? [],
)

async function refresh() {
  loading.value = true
  error.value = ''
  try {
    status.value = await gitStatus()
  } catch (caught) {
    error.value = String(caught)
  } finally {
    loading.value = false
  }
}

async function stage(change: GitChange) {
  await gitStage([change.path])
  await refresh()
}

async function unstage(change: GitChange) {
  await gitUnstage([change.path])
  await refresh()
}

async function showDiff(change: GitChange, staged: boolean) {
  try {
    const diff = await gitDiff(change.path, staged)
    shell.appendOutput(`git diff ${staged ? '--cached ' : ''}-- ${change.path}`)
    shell.appendOutput(diff || '(no textual diff)')
    shell.showDock('output')
  } catch (caught) {
    shell.appendOutput(`git diff failed: ${String(caught)}`, 'error')
  }
}

async function commit() {
  loading.value = true
  error.value = ''
  try {
    const result = await gitCommit(message.value)
    shell.appendOutput(result, 'success')
    message.value = ''
    await refresh()
  } catch (caught) {
    error.value = String(caught)
  } finally {
    loading.value = false
  }
}

function statusLabel(change: GitChange, staged: boolean) {
  if (change.untracked) return 'U'
  return (staged ? change.indexStatus : change.worktreeStatus).trim() || 'M'
}

onMounted(() => void refresh())
</script>

<template>
  <div class="source-control">
    <div v-if="status" class="source-control__branch">
      <AppIcon name="git-branch" :size="14" />
      <span>{{ status.branch }}</span>
      <span v-if="status.ahead || status.behind" class="tree-row__meta">↑{{ status.ahead }} ↓{{ status.behind }}</span>
      <button class="icon-btn" title="Refresh source control" aria-label="Refresh source control" :disabled="loading" @click="refresh">
        <AppIcon name="refresh" :size="14" />
      </button>
    </div>

    <form class="source-control__commit" @submit.prevent="commit">
      <textarea v-model="message" rows="3" placeholder="Message (Ctrl+Enter to commit)" @keydown.ctrl.enter.prevent="commit" />
      <button type="submit" :disabled="loading || !message.trim() || !staged.length">Commit</button>
    </form>

    <p v-if="error" class="side-error">{{ error }}</p>
    <p v-if="loading && !status" class="side-empty">Reading repository…</p>
    <p v-else-if="status && !status.changes.length" class="side-empty">No changes.</p>

    <SideSection v-if="staged.length" title="Staged changes" :count="staged.length">
      <div v-for="change in staged" :key="change.path" class="scm-row">
        <button class="tree-row scm-row__file" :title="change.path" @click="showDiff(change, true)">
          <span class="scm-status">{{ statusLabel(change, true) }}</span>
          <span class="tree-row__label">{{ change.path }}</span>
        </button>
        <button class="icon-btn" title="Unstage" aria-label="Unstage" @click="unstage(change)">−</button>
      </div>
    </SideSection>

    <SideSection v-if="unstaged.length" title="Changes" :count="unstaged.length">
      <div v-for="change in unstaged" :key="change.path" class="scm-row">
        <button class="tree-row scm-row__file" :title="change.path" @click="showDiff(change, false)">
          <span class="scm-status">{{ statusLabel(change, false) }}</span>
          <span class="tree-row__label">{{ change.path }}</span>
        </button>
        <button class="icon-btn" title="Stage" aria-label="Stage" @click="stage(change)">+</button>
      </div>
    </SideSection>
  </div>
</template>

<style scoped>
.source-control {
  display: grid;
  gap: 10px;
}
.source-control__branch {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 4px 9px;
  color: var(--muted);
  font-size: 11.5px;
}
.source-control__branch .tree-row__meta {
  margin-left: auto;
}
.source-control__commit {
  display: grid;
  gap: 6px;
  padding: 0 8px;
}
.source-control__commit textarea {
  min-height: 62px;
  padding: 7px 8px;
  resize: vertical;
  font: 11.5px/1.4 var(--font-family);
}
.source-control__commit button {
  min-height: 27px;
  background: var(--accent);
  color: white;
}
.scm-row {
  display: flex;
  align-items: center;
  min-width: 0;
}
.scm-row__file {
  flex: 1;
  min-width: 0;
}
.scm-status {
  width: 14px;
  flex: none;
  color: var(--warning);
  font-weight: 700;
}
.side-error {
  padding: 5px 9px;
  color: var(--danger);
  font-size: 11px;
}
</style>
