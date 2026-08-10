<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'

const props = withDefaults(defineProps<{ compact?: boolean }>(), {
	compact: false,
})

type Mood = 'sleeping' | 'awake' | 'waving'

const mood = ref<Mood>('sleeping')
let resetTimer: number | undefined

const artwork = computed(() => {
	if (props.compact) {
		return mood.value === 'sleeping'
			? ['  ▄▄▄▄▄  z', '▄█ ███ █▄', '▀  ▀▀▀  ▀'].join('\n')
			: ['  ▄▄▄▄▄  ✦', '▄█ █▀█ █▄', '▀  ▀▀▀  ▀'].join('\n')
	}

	const eyes = mood.value === 'sleeping' ? '──  ──' : '▀▀  ▀▀'
	const arms = mood.value === 'waving' ? '▀█▀ ████████████ ▀█▀' : '▄█▄ ████████████ ▄█▄'
	const message =
		mood.value === 'sleeping' ? 'mimimi...' : mood.value === 'waving' ? 'let’s build!' : 'oh, hey!'
	const bubbles = mood.value === 'sleeping' ? ['z', 'z z'] : ['✦', '  ✦']

	return [
		`                          ${bubbles[0]}`,
		`      ▄▄▄▄▄▄▄▄▄▄▄▄        ${bubbles[1]}`,
		`      ██ ${eyes} ██`,
		`  ${arms}`,
		'  ▀▀  ████████████  ▀▀',
		`      ▀█▀ ▀█▀▀█▀ ▀█▀    ${message}`,
	].join('\n')
})

function greet() {
	if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
		mood.value = mood.value === 'sleeping' ? 'awake' : 'sleeping'
		return
	}

	if (resetTimer) window.clearTimeout(resetTimer)
	mood.value = 'waving'
	resetTimer = window.setTimeout(() => {
		mood.value = 'awake'
	}, 720)
}

onBeforeUnmount(() => {
	if (resetTimer) window.clearTimeout(resetTimer)
})
</script>

<template>
	<button
		type="button"
		:class="[
			'packwand-mascot',
			{ 'packwand-mascot--compact': compact, 'packwand-mascot--waving': mood === 'waving' },
		]"
		:aria-label="mood === 'sleeping' ? 'Wake the Packwand crab' : 'Wave to the Packwand crab'"
		:title="mood === 'sleeping' ? 'Wake the crab' : 'Wave again'"
		@click="greet"
	>
		<pre aria-hidden="true">{{ artwork }}</pre>
	</button>
</template>
