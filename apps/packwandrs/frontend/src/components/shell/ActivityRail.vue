<script setup lang="ts">
import AppIcon from '@/components/ui/AppIcon.vue'
import type { NavItem } from '@/helpers/navigation'
import type { SidebarMode } from '@/stores/shell'

defineProps<{
	items: NavItem[]
	endItems: NavItem[]
	currentName: string
	currentSidebar: SidebarMode
}>()

const emit = defineEmits<{ select: [item: NavItem]; selectSidebar: [mode: SidebarMode] }>()
</script>

<template>
	<nav class="rail" aria-label="Primary">
		<div class="rail-group">
			<button
				class="rail-btn"
				:class="{ active: currentSidebar === 'collab' }"
				title="Live Collaboration"
				aria-label="Live Collaboration"
				@click="emit('selectSidebar', 'collab')"
			>
				<AppIcon name="users" :size="21" />
			</button>
			<button
				class="rail-btn"
				:class="{ active: currentSidebar === 'explorer' }"
				title="Explorer"
				aria-label="Explorer"
				@click="emit('selectSidebar', 'explorer')"
			>
				<AppIcon name="files" :size="21" />
			</button>
			<button
				class="rail-btn"
				:class="{ active: currentSidebar === 'source-control' }"
				title="Source Control"
				aria-label="Source Control"
				@click="emit('selectSidebar', 'source-control')"
			>
				<AppIcon name="source-control" :size="21" />
			</button>
			<button
				class="rail-btn"
				:class="{ active: currentSidebar === 'extensions' }"
				title="Extensions"
				aria-label="Extensions"
				@click="emit('selectSidebar', 'extensions')"
			>
				<AppIcon name="extensions" :size="21" />
			</button>
			<button
				v-for="item in items"
				:key="item.name"
				class="rail-btn"
				:class="{ active: currentName === item.name }"
				:title="item.label"
				:aria-label="item.label"
				:aria-current="currentName === item.name ? 'page' : undefined"
				@click="emit('select', item)"
			>
				<AppIcon :name="item.icon" :size="21" />
			</button>
		</div>
		<div class="rail-group rail-group--end">
			<button
				v-for="item in endItems"
				:key="item.name"
				class="rail-btn"
				:class="{ active: currentName === item.name }"
				:title="item.label"
				:aria-label="item.label"
				@click="emit('select', item)"
			>
				<AppIcon :name="item.icon" :size="21" />
			</button>
		</div>
	</nav>
</template>
