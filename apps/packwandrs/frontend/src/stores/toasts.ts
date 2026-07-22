import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface ToastMessage {
  id: number
  title: string
  message?: string
  tone: 'neutral' | 'success' | 'danger'
}

let nextId = 0

export const useToastsStore = defineStore('toasts', () => {
  const items = ref<ToastMessage[]>([])

  function push(title: string, message?: string, tone: ToastMessage['tone'] = 'neutral') {
    const id = ++nextId
    items.value.push({ id, title, message, tone })
    window.setTimeout(() => dismiss(id), 4_500)
  }

  function dismiss(id: number) {
    items.value = items.value.filter((toast) => toast.id !== id)
  }

  return { items, push, dismiss }
})
