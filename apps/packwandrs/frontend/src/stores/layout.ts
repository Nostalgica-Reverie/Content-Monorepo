import { computed, ref, watch } from 'vue'
import { defineStore } from 'pinia'

import type { ShellLayout } from '@/helpers/types'
import { useSettingsStore } from '@/stores/settings'

/**
 * The shell arrangement, and whether the user may change it.
 *
 * **Scoped to what the shell is actually built from.** An earlier version of
 * this modelled seven independently-placeable panels, which does not match the
 * workbench: `SideBar` is a single component that switches on
 * `shell.sidebarMode` from the activity rail, and the dock is one bottom panel.
 * Modelling placement that the shell cannot honour produced settings that
 * wrote state nothing rendered. What the grid genuinely supports — it is
 * `grid-template-areas` with named `rail`/`side`/`editor` regions — is which
 * side the sidebar occupies, so that is what this offers.
 *
 * Customization is opt-in and unsupported, but always *recoverable*: `reset()`
 * never reads the stored layout, so a bad arrangement can be undone from
 * settings rather than by hand-editing `settings.json`.
 */

export type SidebarSide = 'left' | 'right'

export function defaultLayout(): ShellLayout {
  return { version: 2, sidebarSide: 'left' }
}

/**
 * Repairs a stored layout into something renderable.
 *
 * Never throws and never returns a partial arrangement: an unknown version or
 * an unknown side resolves to the default. Trusting what is on disk means one
 * stale settings file can leave the shell unrenderable with no way back
 * through the UI.
 */
export function reconcileLayout(stored: ShellLayout | null | undefined): ShellLayout {
  if (!stored || stored.version !== 2) return defaultLayout()
  const side: SidebarSide = stored.sidebarSide === 'right' ? 'right' : 'left'
  const sizes = stored.sizes && typeof stored.sizes === 'object' ? stored.sizes : undefined
  return { version: 2, sidebarSide: side, sizes }
}

export const useLayoutStore = defineStore('layout', () => {
  const settings = useSettingsStore()
  const layout = ref<ShellLayout>(defaultLayout())
  const editing = ref(false)

  // Settings load asynchronously, so adopt them whenever they arrive or change
  // rather than reading once at construction.
  watch(
    () => settings.value,
    value => {
      if (!value) return
      layout.value = reconcileLayout(value.layout)
      editing.value = value.layoutEditing === true
      applyReduceMotion(value.reduceMotion === true)
    },
    { immediate: true },
  )

  const sidebarSide = computed<SidebarSide>(() => layout.value.sidebarSide)

  async function persist() {
    if (!settings.value) return
    await settings.save({ ...settings.value, layout: layout.value, layoutEditing: editing.value })
  }

  async function setSidebarSide(side: SidebarSide) {
    layout.value = { ...layout.value, sidebarSide: side }
    await persist()
  }

  async function setEditing(value: boolean) {
    editing.value = value
    await persist()
  }

  /**
   * Restores the default arrangement.
   *
   * Deliberately does not consult the stored layout: this is the escape hatch
   * from an arrangement that cannot be interacted with, so it must not depend
   * on that arrangement being usable.
   */
  async function reset() {
    layout.value = defaultLayout()
    if (!settings.value) return
    await settings.save({ ...settings.value, layout: null })
  }

  async function setReduceMotion(value: boolean) {
    applyReduceMotion(value)
    if (!settings.value) return
    await settings.save({ ...settings.value, reduceMotion: value })
  }

  return { layout, editing, sidebarSide, setSidebarSide, setEditing, reset, setReduceMotion }
})

/**
 * Reflects the setting onto the document root, where the CSS in `base.css`
 * collapses the motion tokens. Kept out of the store's reactive state because
 * it is a side effect on the DOM, not part of the layout model.
 */
function applyReduceMotion(value: boolean) {
  if (typeof document === 'undefined') return
  document.documentElement.dataset.reduceMotion = value ? 'true' : 'false'
}
