<script setup lang="ts">
withDefaults(defineProps<{
  variant?: 'primary' | 'secondary' | 'quiet' | 'danger'
  busy?: boolean
  disabled?: boolean
  type?: 'button' | 'submit'
}>(), { variant: 'primary', busy: false, disabled: false, type: 'button' })
</script>

<template>
  <button :class="['button', `button--${variant}`]" :type="type" :disabled="disabled || busy">
    <span v-if="busy" class="button__spinner" aria-hidden="true" />
    <slot />
  </button>
</template>
