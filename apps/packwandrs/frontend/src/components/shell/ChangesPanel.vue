<script setup lang="ts">
import { onMounted, ref } from 'vue'

import SideSection from '@/components/shell/SideSection.vue'
import type { StackEntry } from '@/helpers/invoke/changes'
import { useChangesStore } from '@/stores/changes'

const changes = useChangesStore()
const editing = ref('')
const message = ref('')

function startDescription(entry: StackEntry) {
	editing.value = entry.changeId
	message.value = entry.description
}

async function saveDescription(entry: StackEntry) {
	if (!message.value.trim()) return
	await changes.describe(entry.changeId, message.value)
	editing.value = ''
}

async function squash(entry: StackEntry) {
	if (!window.confirm(`Squash ${entry.changeId} into its first parent?`)) return
	await changes.squash(entry.changeId)
}

onMounted(() => void changes.refresh())
</script>

<template>
	<SideSection title="Jujutsu changes" :count="changes.entries.length">
		<div class="change-actions">
			<button v-if="changes.needsEnable" :disabled="changes.busy" @click="changes.enable">
				Enable for this repository
			</button>
			<button v-else :disabled="changes.busy" @click="changes.create()">New change</button>
			<button
				class="icon-btn"
				:disabled="changes.busy"
				title="Refresh changes"
				@click="changes.refresh"
			>
				↻
			</button>
		</div>
		<p v-if="changes.error && !changes.needsEnable" class="side-error">{{ changes.error }}</p>
		<p v-else-if="!changes.entries.length && !changes.needsEnable" class="side-empty">
			No local changes.
		</p>
		<div v-for="entry in changes.entries" :key="entry.commitId" class="change-row">
			<div class="change-row__summary">
				<span class="change-row__marker">{{ entry.isWorkingCopy ? '@' : '○' }}</span>
				<button class="change-row__description" @click="startDescription(entry)">
					{{ entry.description || '(no description set)' }}
				</button>
				<span v-if="entry.divergent" class="change-row__warning">divergent</span>
			</div>
			<code>{{ entry.changeId.slice(0, 12) }}</code>
			<form v-if="editing === entry.changeId" @submit.prevent="saveDescription(entry)">
				<input v-model="message" aria-label="Change description" />
				<button :disabled="changes.busy || !message.trim()">Save</button>
			</form>
			<button
				v-if="entry.parentChangeId"
				class="change-row__squash"
				:disabled="changes.busy || entry.divergent"
				@click="squash(entry)"
			>
				Squash into parent
			</button>
		</div>
	</SideSection>
</template>

<style scoped>
.change-actions,
.change-row__summary,
.change-row form {
	display: flex;
	align-items: center;
	gap: 6px;
}
.change-actions {
	padding: 4px 8px 8px;
}
.change-row {
	display: grid;
	gap: 4px;
	padding: 7px 9px;
	border-top: 1px solid var(--border);
}
.change-row__description {
	min-width: 0;
	flex: 1;
	overflow: hidden;
	padding: 0;
	background: transparent;
	text-align: left;
	text-overflow: ellipsis;
	white-space: nowrap;
}
.change-row__marker {
	color: var(--accent);
	font-weight: 700;
}
.change-row__warning,
.side-error {
	color: var(--danger);
}
.change-row code {
	color: var(--muted);
	font-size: 10px;
}
.change-row form input {
	min-width: 0;
	flex: 1;
}
.change-row__squash {
	justify-self: start;
	padding: 2px 0;
	background: transparent;
	color: var(--muted);
	font-size: 10px;
}
.side-error {
	padding: 5px 9px;
	font-size: 11px;
}
</style>
