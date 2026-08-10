<script setup lang="ts">
defineProps<{ open: boolean; title: string }>()
defineEmits<{ close: [] }>()
</script>

<template>
	<Teleport to="body">
		<Transition name="fade">
			<div v-if="open" class="modal-backdrop" @click.self="$emit('close')">
				<Transition name="scale-fade" appear>
					<section class="modal" role="dialog" aria-modal="true" :aria-label="title">
						<header>
							<h2>{{ title }}</h2>
							<button aria-label="Close" @click="$emit('close')">×</button>
						</header>
						<div class="modal__body"><slot /></div>
					</section>
				</Transition>
			</div>
		</Transition>
	</Teleport>
</template>
