import { defineStore } from 'pinia'
import { shallowRef } from 'vue'

import { core, type CoreEffect, type CoreMessage } from '@/core/packwand'

export const useAppCoreStore = defineStore('app-core', () => {
  const model = shallowRef(core.init('builtin.packwand-dark'))

  function dispatch(message: CoreMessage): CoreEffect[] {
    const [next, effects] = core.update(model.value, message)
    model.value = next
    return [...effects]
  }

  return { model, dispatch }
})
