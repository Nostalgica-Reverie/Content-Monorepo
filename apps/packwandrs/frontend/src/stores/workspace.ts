import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { workspaceGet, workspaceSelect, workspaceSet } from '@/helpers/invoke/workspace'

export const useWorkspaceStore = defineStore('workspace', () => {
  const path = ref<string | null>(null)
  const loading = ref(false)
  const configured = computed(() => Boolean(path.value))

  async function load() {
    loading.value = true
    try {
      path.value = await workspaceGet()
      return path.value
    } finally {
      loading.value = false
    }
  }

  async function select() {
    const selected = await workspaceSelect()
    if (selected) path.value = selected
    return selected
  }

  async function set(nextPath: string) {
    path.value = await workspaceSet(nextPath)
    return path.value
  }

  return { path, loading, configured, load, select, set }
})
