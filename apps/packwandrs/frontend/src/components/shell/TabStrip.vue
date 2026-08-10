<script setup lang="ts">
import AppIcon from '@/components/ui/AppIcon.vue'
import type { ShellTab } from '@/stores/shell'

defineProps<{ tabs: ShellTab[]; currentName: string }>()

const emit = defineEmits<{
	select: [tab: ShellTab]
	close: [name: string]
	toggleDock: []
	toggleSidebar: []
}>()
</script>

<template>
	<div class="tabstrip" role="tablist">
		<TransitionGroup tag="div" name="tab" class="tabstrip__scroll">
			<div
				v-for="tab in tabs"
				:key="tab.name"
				class="tab"
				:class="{ active: currentName === tab.name }"
				role="tab"
				:aria-selected="currentName === tab.name"
				@click="emit('select', tab)"
				@mousedown.middle.prevent="emit('close', tab.name)"
			>
				<AppIcon :name="tab.icon" :size="14" class="tab__icon" />
				<span class="tab__label">{{ tab.label }}</span>
				<button
					class="tab__close"
					:title="`Close ${tab.label}`"
					:aria-label="`Close ${tab.label}`"
					@click.stop="emit('close', tab.name)"
				>
					<AppIcon name="close" :size="12" />
				</button>
			</div>
		</TransitionGroup>
		<div class="tabstrip__actions">
			<button
				class="icon-btn"
				title="Toggle sidebar (Ctrl+B)"
				aria-label="Toggle sidebar"
				@click="emit('toggleSidebar')"
			>
				<AppIcon name="sidebar" :size="15" />
			</button>
			<button
				class="icon-btn"
				title="Toggle panel (Ctrl+J)"
				aria-label="Toggle panel"
				@click="emit('toggleDock')"
			>
				<AppIcon name="panel" :size="15" />
			</button>
		</div>
	</div>
</template>
