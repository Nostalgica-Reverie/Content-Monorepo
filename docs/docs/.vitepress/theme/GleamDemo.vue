<script setup>
// A Vue component whose logic is implemented in Gleam: the functions below
// are compiled from docs/gleam-demo/src/docs_demo.gleam to JavaScript
// (rebuild with `node docs/gleam-demo/build.mjs`).
import { ref, computed } from 'vue'
import { compare_versions, slugify } from './gleam/docs_demo/docs_demo.mjs'

const versionA = ref('1.20.1')
const versionB = ref('1.9')
const packName = ref('My Cool Pack!')

const verdict = computed(() => compare_versions(versionA.value, versionB.value))
const slug = computed(() => slugify(packName.value))
</script>

<template>
	<div class="gleam-demo">
		<p class="gleam-demo-title">⭐ Powered by Gleam (compiled to JavaScript)</p>
		<div class="gleam-demo-row">
			<label
				>Compare Minecraft versions:
				<input v-model="versionA" placeholder="1.20.1" />
				vs
				<input v-model="versionB" placeholder="1.9" />
			</label>
			<p class="gleam-demo-out">{{ verdict }}</p>
		</div>
		<div class="gleam-demo-row">
			<label
				>Slugify a pack name:
				<input v-model="packName" placeholder="My Cool Pack!" />
			</label>
			<p class="gleam-demo-out">
				<code>{{ slug }}</code>
			</p>
		</div>
	</div>
</template>

<style scoped>
.gleam-demo {
	border: 1px solid var(--vp-c-divider);
	border-radius: 8px;
	padding: 1rem 1.25rem;
	margin: 1rem 0;
	background: var(--vp-c-bg-soft);
}
.gleam-demo-title {
	font-weight: 600;
	color: var(--vp-c-brand-1);
	margin: 0 0 0.5rem;
}
.gleam-demo-row {
	margin: 0.75rem 0;
}
.gleam-demo input {
	border: 1px solid var(--vp-c-divider);
	border-radius: 4px;
	padding: 0.15rem 0.5rem;
	margin: 0 0.4rem;
	background: var(--vp-c-bg);
	color: var(--vp-c-text-1);
	width: 7.5rem;
}
.gleam-demo-out {
	margin: 0.35rem 0 0;
	color: var(--vp-c-text-2);
}
</style>
