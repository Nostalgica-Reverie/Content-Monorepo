import { onMounted, onUnmounted, ref, type Ref } from 'vue'

/** A small Vue-native resource loader for desktop IPC data. */
export function usePolling<T>(
  load: () => Promise<T>,
  intervalMs = 0,
  enabled: () => boolean = () => true,
) {
  const data = ref<T | null>(null) as Ref<T | null>
  const pending = ref(true)
  const error = ref<unknown>(null)
  let timer: ReturnType<typeof setInterval> | undefined

  async function refresh() {
    if (!enabled()) return data.value
    pending.value = data.value === null
    try {
      data.value = await load()
      error.value = null
      return data.value
    } catch (caught) {
      error.value = caught
      throw caught
    } finally {
      pending.value = false
    }
  }

  onMounted(() => {
    void refresh().catch(() => undefined)
    if (intervalMs > 0) {
      timer = setInterval(() => void refresh().catch(() => undefined), intervalMs)
    }
  })
  onUnmounted(() => {
    if (timer) clearInterval(timer)
  })

  return { data, pending, error, refresh }
}
