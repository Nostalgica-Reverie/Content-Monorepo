<script setup lang="ts">
defineProps<{ title: string; message: string }>()
</script>

<template>
  <div class="empty-state">
    <div class="empty-state__mark">◇</div>
    <h2>{{ title }}</h2>
    <p>{{ message }}</p>
    <div v-if="$slots.action || $slots.default" class="empty-state__action"><slot name="action"><slot /></slot></div>
  </div>
</template>
