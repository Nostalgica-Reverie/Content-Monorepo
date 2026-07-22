import { defineStore } from 'pinia'
import { reactive, ref } from 'vue'

import { instancesStatusList } from '@/helpers/invoke/instances'
import type { InstanceStatusPayload } from '@/helpers/types'

/**
 * Shared, live "what is this instance doing right now" truth, keyed by
 * instance (pack) id. Mirrors the role mrapp's `process_listener`/
 * `instance_listener` play for its `Instance.vue` cards, but centralized in
 * a Pinia store (Packwand's established pattern for cross-page reactive
 * state — see `stores/workbench.ts`, `stores/settings.ts`) instead of each
 * card owning its own `listen()` subscription.
 */
export const useInstancesStore = defineStore('instances', () => {
  const statuses = reactive<Record<string, InstanceStatusPayload>>({})
  const hydrated = ref(false)

  /** Hydrates from the backend's in-memory registry so a page that mounts
   * mid-launch (or after navigating away and back) doesn't show a blank
   * "idle" card until the next event arrives. Safe to call repeatedly —
   * only does work once per successful load. */
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
