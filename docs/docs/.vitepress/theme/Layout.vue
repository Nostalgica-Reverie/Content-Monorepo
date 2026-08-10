<script setup>
import DefaultTheme from 'vitepress/theme'
import { onMounted, watch, nextTick } from 'vue'
import { useRoute } from 'vitepress'
import mediumZoom from 'medium-zoom'
import HomeSearch from './HomeSearch.vue'

const { Layout } = DefaultTheme
const route = useRoute()

const setupMediumZoom = () => {
	mediumZoom('[data-zoomable]', { background: 'transparent' })
}

onMounted(setupMediumZoom)
watch(
	() => route.path,
	() => nextTick(setupMediumZoom),
)
</script>

<template>
	<Layout>
		<template #home-features-after>
			<HomeSearch />
		</template>
	</Layout>
</template>
