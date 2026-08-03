import { defineStore } from 'pinia'
import { reactive, ref } from 'vue'

import { instancesStatusList } from '@/helpers/invoke/instances'
import type { InstanceStatusPayload } from '@/helpers/types'

/** Shared live instance status, keyed by pack id. */
export const useInstancesStore = defineStore('instances', () => {
  const statuses = reactive<Record<string, InstanceStatusPayload>>({})
  const hydrated = ref(false)

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

  return { statuses, hydrated, hydrate, apply }
})
