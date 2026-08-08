import { defineStore } from 'pinia'
import { computed, reactive, ref } from 'vue'

import { instancesContentList, instancesCreate, instancesDelete, instancesEdit, instancesGet, instancesInstall, instancesList, instancesStatusList } from '@/helpers/invoke/instances'
import type { CreateInstanceSpec, InstanceContent, InstanceStatusPayload, InstanceSummary } from '@/helpers/types'

/** Shared live instance status, keyed by pack id. */
export const useInstancesStore = defineStore('instances', () => {
  const statuses = reactive<Record<string, InstanceStatusPayload>>({})
  const items = ref<InstanceSummary[]>([])
  const current = ref<InstanceSummary | null>(null)
  const content = ref<InstanceContent[]>([])
  const loading = ref(false)
  const hydrated = ref(false)
  const byId = computed(() => new Map(items.value.map(instance => [instance.id, instance])))

  async function refresh() {
    loading.value = true
    try { items.value = await instancesList() } finally { loading.value = false }
  }

  async function load(id: string) {
    current.value = await instancesGet(id)
    content.value = await instancesContentList(id)
    return current.value
  }

  async function create(spec: CreateInstanceSpec) {
    const instance = await instancesCreate(spec)
    items.value.push(instance)
    return instance
  }

  async function edit(id: string, patch: Parameters<typeof instancesEdit>[1]) {
    const instance = await instancesEdit(id, patch)
    current.value = instance
    await refresh()
    return instance
  }

  async function remove(id: string, deleteFiles = false) {
    await instancesDelete(id, deleteFiles)
    items.value = items.value.filter(instance => instance.id !== id)
  }

  /** Load current statuses once so remounted pages do not start blank. */
  async function hydrate() {
    if (hydrated.value) return
    try {
      const list = await instancesStatusList()
      for (const entry of list) statuses[entry.id] = entry
      hydrated.value = true
    } catch {
      // Leave `hydrated` false so a later mount can retry.
    }
  }

  function apply(payload: InstanceStatusPayload) {
    statuses[payload.id] = payload
  }

  return { statuses, items, current, content, loading, hydrated, byId, hydrate, refresh, load, create, edit, remove, install: instancesInstall, apply }
})
