<script setup lang="ts">
import { ref } from 'vue'
import AppIcon from '@/components/ui/AppIcon.vue'

const props = withDefaults(
	defineProps<{ title: string; count?: number | string; open?: boolean }>(),
	{
		open: true,
	},
)

const expanded = ref(props.open)
</script>

<template>
	<section class="side-section">
		<button class="side-section__head" :aria-expanded="expanded" @click="expanded = !expanded">
			<AppIcon
				name="chevron-down"
				:size="15"
				class="side-section__chevron"
				:class="{ 'side-section__chevron--collapsed': !expanded }"
			/>
			<span>{{ title }}</span>
			<span v-if="count !== undefined" class="side-section__count">{{ count }}</span>
		</button>
		<Transition name="slide-fade">
			<div v-if="expanded" class="side-section__body">
				<slot />
			</div>
		</Transition>
	</section>
</template>
