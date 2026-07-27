<script setup lang="ts">
import AppIcon from '@/components/ui/AppIcon.vue'
import type { NavItem } from '@/helpers/navigation'

defineProps<{
  items: NavItem[]
  endItems: NavItem[]
  currentName: string
}>()

const emit = defineEmits<{ select: [item: NavItem] }>()
</script>

<template>
  <nav class="rail" aria-label="Primary">
    <div class="rail-group">
      <button
        v-for="item in items"
        :key="item.name"
        class="rail-btn"
        :class="{ active: currentName === item.name }"
        :title="item.label"
        :aria-label="item.label"
        :aria-current="currentName === item.name ? 'page' : undefined"
        @click="emit('select', item)"
      >
        <AppIcon :name="item.icon" :size="21" />
      </button>
    </div>
    <div class="rail-group rail-group--end">
      <button
        v-for="item in endItems"
        :key="item.name"
        class="rail-btn"
        :class="{ active: currentName === item.name }"
        :title="item.label"
        :aria-label="item.label"
        @click="emit('select', item)"
      >
        <AppIcon :name="item.icon" :size="21" />
      </button>
    </div>
  </nav>
</template>
