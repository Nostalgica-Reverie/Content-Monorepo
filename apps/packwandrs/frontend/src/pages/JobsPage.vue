<script setup lang="ts">
import { computed, ref } from 'vue'
import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import LogViewer from '@/components/ui/LogViewer.vue'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { usePolling } from '@/composables/usePolling'
import { jobCancel, jobsList } from '@/helpers/invoke/jobs'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

const workbench = useWorkbenchStore()
const toasts = useToastsStore()
const query = usePolling(jobsList, 750)
const selectedId = ref<string | null>(null)
const filtered = computed(() => {
  const term = workbench.search.trim().toLowerCase()
  return query.data.value?.filter((job) => !term || (job.label + ' ' + job.kind + ' ' + job.logs.join(' ')).toLowerCase().includes(term)) ?? []
})
const selected = computed(() => filtered.value.find((job) => job.id === selectedId.value) ?? filtered.value[0] ?? null)
const running = computed(() => query.data.value?.filter((job) => job.status === 'running') ?? [])
async function cancel() {
  if (!selected.value) return
  try { await jobCancel(selected.value.id); await query.refresh() }
  catch (error) { toasts.push('Cancellation failed', String(error), 'danger') }
}
</script>

<template>
  <section class="grid view-grid logs-grid">
    <article class="panel span-5 compact-panel">
      <div class="panel-head"><h2>Progress</h2><span :class="['status-badge', running.length ? 'integrated' : '']">{{ running.length }} active</span></div>
      <div v-if="running.length" class="list">
        <div v-for="job in running" :key="job.id" class="mini-row"><strong>{{ job.label }}</strong><ProgressBar :value="job.fraction" :label="job.message ?? job.status" /></div>
      </div>
      <p v-else class="empty-note">No background operations are currently running.</p>
    </article>
    <article class="panel span-7 compact-panel">
      <div class="panel-head"><h2>Activity</h2><Button variant="quiet" @click="query.refresh()">Reload</Button></div>
      <p class="panel-copy">Builds, metadata operations, diagnostics, synchronization, and installer tasks appear here.</p>
      <div class="details"><div class="detail"><span>Total</span><strong>{{ query.data.value?.length || 0 }}</strong></div><div class="detail"><span>Running</span><strong>{{ running.length }}</strong></div><div class="detail"><span>Failed</span><strong>{{ query.data.value?.filter((job) => job.status === 'failed').length || 0 }}</strong></div></div>
    </article>
    <article class="panel span-12 logs-panel">
      <div class="panel-head"><h2>Job logs</h2><span v-if="selected" class="pill">{{ selected.status }}</span></div>
      <EmptyState v-if="!query.pending.value && !filtered.length" title="No jobs yet" message="Pack changes, exports, diagnostics, and installs will appear here." />
      <div v-else class="logs-workbench">
        <aside class="job-list">
          <button v-for="job in filtered" :key="job.id" :class="{ active: selected?.id === job.id }" @click="selectedId = job.id"><strong>{{ job.label }}</strong><span>{{ job.kind }} · {{ Math.round(job.fraction * 100) }}%</span><i :class="'job-state ' + job.status" /></button>
        </aside>
        <div v-if="selected" class="job-detail">
          <div class="detail-heading"><div><span class="status-badge">{{ selected.kind }}</span><h2>{{ selected.label }}</h2></div><Button v-if="selected.status === 'running'" variant="danger" @click="cancel">Cancel</Button></div>
          <ProgressBar :value="selected.fraction" :label="selected.message ?? selected.status" />
          <p v-if="selected.error" class="error-banner">{{ selected.error.message }}</p>
          <LogViewer :lines="selected.logs" />
        </div>
      </div>
    </article>
  </section>
</template>
