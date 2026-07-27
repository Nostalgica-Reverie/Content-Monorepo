<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { jobsList } from '@/helpers/invoke/jobs'
import { useShellStore } from '@/stores/shell'
import type { DockTab } from '@/stores/shell'

const shell = useShellStore()

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
]

function countFor(tab: DockTab) {
  if (tab === 'problems') return shell.problems.length || undefined
  if (tab === 'output') return shell.output.length || undefined
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
