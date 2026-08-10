<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import LogViewer from '@/components/ui/LogViewer.vue'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { usePolling } from '@/composables/usePolling'
import { jobCancel, jobsList } from '@/helpers/invoke/jobs'
import { somnusList, somnusRun } from '@/helpers/invoke/somnus'
import type { SomnusWorkflowEntry } from '@/helpers/invoke/somnus'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

const workbench = useWorkbenchStore()
const toasts = useToastsStore()
const query = usePolling(jobsList, 750)
const selectedId = ref<string | null>(null)
const somnusWorkflows = ref<SomnusWorkflowEntry[]>([])
const somnusAvailable = ref(true)
const somnusBusy = ref(false)

async function loadSomnus() {
	try {
		somnusWorkflows.value = await somnusList()
		somnusAvailable.value = true
	} catch {
		// Somnus is a separate, optional Go binary (apps/packwandrs/somnus) — an
		// unbuilt binary is a normal state, not an error worth a toast every
		// time this page loads.
		somnusWorkflows.value = []
		somnusAvailable.value = false
	}
}

async function runSomnus(workflow?: string) {
	somnusBusy.value = true
	try {
		await somnusRun(workflow)
		toasts.push('Somnus', workflow ? `Running ${workflow}` : 'Running every matching workflow', 'success')
		await query.refresh()
	} catch (error) {
		toasts.push('Somnus failed to start', String(error), 'danger')
	} finally {
		somnusBusy.value = false
	}
}

onMounted(loadSomnus)
const filtered = computed(() => {
	const term = workbench.search.trim().toLowerCase()
	return (
		query.data.value?.filter(
			(job) =>
				!term ||
				(job.label + ' ' + job.kind + ' ' + job.logs.join(' ')).toLowerCase().includes(term),
		) ?? []
	)
})
const selected = computed(
	() => filtered.value.find((job) => job.id === selectedId.value) ?? filtered.value[0] ?? null,
)
const running = computed(() => query.data.value?.filter((job) => job.status === 'running') ?? [])
async function cancel() {
	if (!selected.value) return
	try {
		await jobCancel(selected.value.id)
		await query.refresh()
	} catch (error) {
		toasts.push('Cancellation failed', String(error), 'danger')
	}
}
</script>

<template>
	<section class="grid view-grid logs-grid">
		<article class="panel span-5 compact-panel">
			<div class="panel-head">
				<h2>Progress</h2>
				<span :class="['status-badge', running.length ? 'integrated' : '']"
					>{{ running.length }} active</span
				>
			</div>
			<div v-if="running.length" class="list">
				<div v-for="job in running" :key="job.id" class="mini-row">
					<strong>{{ job.label }}</strong
					><ProgressBar :value="job.fraction" :label="job.message ?? job.status" />
				</div>
			</div>
			<p v-else class="empty-note">No background operations are currently running.</p>
		</article>
		<article class="panel span-7 compact-panel">
			<div class="panel-head">
				<h2>Activity</h2>
				<Button variant="quiet" @click="query.refresh()">Reload</Button>
			</div>
			<p class="panel-copy">
				Builds, metadata operations, diagnostics, synchronization, and installer tasks appear here.
			</p>
			<div class="details">
				<div class="detail">
					<span>Total</span><strong>{{ query.data.value?.length || 0 }}</strong>
				</div>
				<div class="detail">
					<span>Running</span><strong>{{ running.length }}</strong>
				</div>
				<div class="detail">
					<span>Failed</span
					><strong>{{
						query.data.value?.filter((job) => job.status === 'failed').length || 0
					}}</strong>
				</div>
			</div>
		</article>
		<article class="panel span-12 compact-panel">
			<div class="panel-head">
				<h2>Local CI (Somnus)</h2>
				<Button v-if="somnusAvailable" variant="quiet" :busy="somnusBusy" @click="runSomnus()">
					Run matching workflows
				</Button>
			</div>
			<p class="panel-copy">
				Runs the same <code>.tangled/workflows/*.yml</code> steps that would run upstream,
				streamed into the job log below.
			</p>
			<div v-if="somnusWorkflows.length" class="list">
				<div v-for="workflow in somnusWorkflows" :key="workflow.path" class="row">
					<strong>{{ workflow.name }}</strong>
					<span style="display: flex; align-items: center; gap: 8px">
						<span :class="['status-badge', workflow.trigger ? 'integrated' : '']">
							{{ workflow.trigger ? 'would trigger' : 'no match' }}
						</span>
						<Button variant="quiet" :busy="somnusBusy" @click="runSomnus(workflow.path)">
							Run
						</Button>
					</span>
				</div>
			</div>
			<p v-else class="empty-note">
				{{
					somnusAvailable
						? 'No workflows discovered under .tangled/workflows.'
						: 'Somnus binary not found — build apps/packwandrs/somnus or set PACKWAND_SOMNUS_BIN.'
				}}
			</p>
		</article>
		<article class="panel span-12 logs-panel">
			<div class="panel-head">
				<h2>Job logs</h2>
				<span v-if="selected" class="pill">{{ selected.status }}</span>
			</div>
			<EmptyState
				v-if="!query.pending.value && !filtered.length"
				title="No jobs yet"
				message="Pack changes, exports, diagnostics, and installs will appear here."
			/>
			<div v-else class="logs-workbench">
				<aside class="job-list">
					<button
						v-for="job in filtered"
						:key="job.id"
						:class="{ active: selected?.id === job.id }"
						@click="selectedId = job.id"
					>
						<strong>{{ job.label }}</strong
						><span>{{ job.kind }} · {{ Math.round(job.fraction * 100) }}%</span
						><i :class="'job-state ' + job.status" />
					</button>
				</aside>
				<div v-if="selected" class="job-detail">
					<div class="detail-heading">
						<div>
							<span class="status-badge">{{ selected.kind }}</span>
							<h2>{{ selected.label }}</h2>
						</div>
						<Button v-if="selected.status === 'running'" variant="danger" @click="cancel"
							>Cancel</Button
						>
					</div>
					<ProgressBar :value="selected.fraction" :label="selected.message ?? selected.status" />
					<p v-if="selected.error" class="error-banner">{{ selected.error.message }}</p>
					<LogViewer :lines="selected.logs" />
				</div>
			</div>
		</article>
	</section>
</template>
