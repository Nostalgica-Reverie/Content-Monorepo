<script setup lang="ts">
import type { ToastMessage } from '@/stores/toasts'

defineProps<{ toast: ToastMessage }>()
defineEmits<{ dismiss: [] }>()
</script>

<template>
	<article :class="['toast', `toast--${toast.tone}`]">
		<div>
			<strong>{{ toast.title }}</strong>
			<p v-if="toast.message">{{ toast.message }}</p>
		</div>
		<button aria-label="Dismiss notification" @click="$emit('dismiss')">×</button>
	</article>
</template>
