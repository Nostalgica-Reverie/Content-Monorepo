<script setup lang="ts">
import AppIcon from '@/components/ui/AppIcon.vue'
import { useAuthStore } from '@/stores/auth'
import { useCollabStore } from '@/stores/collab'
import { useShellStore } from '@/stores/shell'
import { useWorkbenchStore } from '@/stores/workbench'

const shell = useShellStore()
const workbench = useWorkbenchStore()
const auth = useAuthStore()
const collab = useCollabStore()
</script>

<template>
	<footer class="statusbar">
		<button class="statusbar__item" title="Reindex workspace" @click="workbench.refresh()">
			<AppIcon name="sync" :size="13" />
			<span>{{ workbench.loading ? 'Indexing…' : 'Native Rust workspace' }}</span>
		</button>
		<button
			class="statusbar__item"
			:class="{ 'statusbar__item--error': shell.errorCount }"
			title="Show problems"
			@click="shell.showDock('problems')"
		>
			<AppIcon name="error" :size="13" />
			<span>{{ shell.errorCount }}</span>
			<AppIcon name="warning" :size="13" />
			<span>{{ shell.warningCount }}</span>
		</button>
		<button class="statusbar__item" title="Show output" @click="shell.showDock('output')">
			<span>Output</span>
		</button>

		<div class="statusbar__spacer" />

		<div v-if="workbench.selectedPack" class="statusbar__item" :title="workbench.selectedPack.path">
			<AppIcon name="target" :size="13" />
			<span>{{ workbench.selectedPack.id }}</span>
		</div>
		<div class="statusbar__item">
			<span>{{ auth.label }}</span>
		</div>
		<div class="statusbar__item" title="Tauri IPC bridge">
			<i class="connection-dot" /><span>IPC</span>
		</div>
		<button
			class="statusbar__item"
			:title="collab.role ? `Live session: ${collab.role}` : 'Live collaboration'"
			@click="shell.showSidebar('collab')"
		>
			<i
				class="connection-dot"
				:class="collab.connection === 'connected' ? 'connection-dot--live' : 'connection-dot--idle'"
			/>
			<span>{{ collab.role ? `${collab.participants.length} live` : 'Offline' }}</span>
		</button>
		<div class="statusbar__item"><span>packwand 26.2.0</span></div>
	</footer>
</template>
