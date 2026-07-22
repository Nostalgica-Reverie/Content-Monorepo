<script setup lang="ts">
defineProps<{ items: Array<{ id: string; label: string }>; modelValue: string }>()
defineEmits<{ 'update:modelValue': [value: string] }>()
</script>

<template>
  <div class="tabs" role="tablist">
    <button v-for="item in items" :key="item.id" role="tab" :aria-selected="item.id === modelValue" @click="$emit('update:modelValue', item.id)">
      {{ item.label }}
    </button>
  </div>
</template>
