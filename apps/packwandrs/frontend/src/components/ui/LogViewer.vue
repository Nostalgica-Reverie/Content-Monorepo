<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'

const props = withDefaults(defineProps<{ lines: string[]; follow?: boolean }>(), { follow: true })
const rowHeight = 21
const viewportHeight = 210
const scrollTop = ref(0)
const viewport = ref<HTMLElement | null>(null)
const start = computed(() => Math.max(0, Math.floor(scrollTop.value / rowHeight) - 5))
const visibleCount = Math.ceil(viewportHeight / rowHeight) + 10
const visible = computed(() => props.lines.slice(start.value, start.value + visibleCount))

watch(
	() => props.lines.length,
	async () => {
		if (!props.follow) return
		await nextTick()
		if (viewport.value) viewport.value.scrollTop = viewport.value.scrollHeight
	},
)
</script>

<template>
	<div
		ref="viewport"
		class="log-viewer"
		:style="{ height: `${viewportHeight}px` }"
		@scroll="scrollTop = ($event.target as HTMLElement).scrollTop"
	>
		<div :style="{ height: `${lines.length * rowHeight}px`, position: 'relative' }">
			<div :style="{ transform: `translateY(${start * rowHeight}px)` }">
				<div v-for="(line, index) in visible" :key="start + index" class="log-viewer__line">
					{{ line }}
				</div>
			</div>
		</div>
	</div>
</template>
