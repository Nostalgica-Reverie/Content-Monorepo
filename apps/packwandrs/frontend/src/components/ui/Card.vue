<script setup lang="ts">
defineProps<{ elevated?: boolean }>()
</script>

<template>
	<section :class="['card', { 'card--elevated': elevated }]">
		<slot />
	</section>
</template>
