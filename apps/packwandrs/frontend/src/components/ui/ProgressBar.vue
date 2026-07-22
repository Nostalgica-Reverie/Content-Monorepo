<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{ value: number; label?: string; indeterminate?: boolean }>(), {
  label: '',
  indeterminate: false,
})
const width = computed(() => `${Math.round(Math.min(1, Math.max(0, props.value)) * 100)}%`)
</script>

<template>
  <div class="progress" :aria-label="label" role="progressbar" :aria-valuenow="Math.round(value * 100)">
    <div :class="['progress__fill', { 'progress__fill--indeterminate': indeterminate }]" :style="{ width }" />
  </div>
</template>
