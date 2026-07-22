<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import AppIcon from '@/components/ui/AppIcon.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import InstanceCard from '@/components/ui/InstanceCard.vue'
import Tabs from '@/components/ui/Tabs.vue'
import { instancesList } from '@/helpers/invoke/instances'
import type { InstanceSummary } from '@/helpers/types'
import { useToastsStore } from '@/stores/toasts'

// Mirrors mrapp's library tabs (`pages/library/Index.vue`'s `NavTabs`
// links): "All instances" / "Modpacks" / "Servers" / "Custom". Servers and
// Custom stay empty states — Packwand has no concept for either yet, so
// this filters real data rather than faking a populated tab.
const tabs = [
  { id: 'all', label: 'All instances' },
  { id: 'modpacks', label: 'Modpacks' },
  { id: 'servers', label: 'Servers' },
  { id: 'custom', label: 'Custom' },
]

const toasts = useToastsStore()
const activeTab = ref('all')
const search = ref('')
const loading = ref(false)
const instances = ref<InstanceSummary[]>([])

const filtered = computed(() => {
  if (activeTab.value === 'servers' || activeTab.value === 'custom') return []
  const term = search.value.trim().toLowerCase()
  return instances.value.filter((instance) => !term || (instance.name + ' ' + instance.id).toLowerCase().includes(term))
})

async function refresh() {
  loading.value = true
  try {
    instances.value = await instancesList()
  } catch (error) {
    toasts.push('Could not load instances', String(error), 'danger')
  } finally {
    loading.value = false
  }
}

onMounted(refresh)
</script>

<template>
  <section class="grid view-grid">
    <div class="panel span-12 instances-panel">
      <div class="panel-head">
        <div>
          <h2>Instances</h2>
          <p class="panel-copy">Launch a pack as a real Minecraft instance: installs its mods, resolves the Minecraft version and loader, then boots with an offline session.</p>
        </div>
        <span class="pill">{{ filtered.length }} shown</span>
      </div>

      <Tabs v-model="activeTab" :items="tabs" />
      <label class="instance-search"><AppIcon name="search" :size="15" /><input v-model="search" type="search" placeholder="Search instances…" /></label>

      <EmptyState v-if="activeTab === 'servers'" title="No servers yet" message="Packwand does not manage server instances yet — this tab will populate once that capability exists." />
      <EmptyState v-else-if="activeTab === 'custom'" title="No custom instances yet" message="Custom, non-pack instances aren't supported yet — this tab will populate once that capability exists." />
      <EmptyState v-else-if="!loading && !filtered.length" title="No instances found" message="No pack target matches this search, or this workspace has no discoverable pack.toml yet." />
      <div v-else class="instance-list">
        <InstanceCard v-for="instance in filtered" :key="instance.id" :instance="instance" />
      </div>
    </div>
  </section>
</template>
