<script setup lang="ts">
import { computed, ref } from 'vue'

import AppIcon from '@/components/ui/AppIcon.vue'
import type { ExtensionManifest } from '@/extensions/api'
import { useExtensionsStore } from '@/stores/extensions'
import { useWorkbenchStore } from '@/stores/workbench'

const extensions = useExtensionsStore()
const workbench = useWorkbenchStore()
const query = ref('')
const expanded = ref('')
const logoUrl = new URL('../../../../src-tauri/icons/icon.png', import.meta.url).href

function matching(manifests: ExtensionManifest[]) {
  const needle = query.value.trim().toLowerCase()
  if (!needle) return manifests
  return manifests.filter((manifest) =>
    [manifest.name, manifest.id, manifest.description, ...manifest.capabilities]
      .filter(Boolean)
      .some((value) => String(value).toLowerCase().includes(needle)),
  )
}

const sections = computed(() => [
  {
    id: 'installed',
    label: 'Installed',
    items: matching(extensions.manifests.filter((manifest) => extensions.isInstalled(manifest.id))),
  },
  {
    id: 'available',
    label: 'Available',
    items: matching(extensions.manifests.filter((manifest) => !extensions.isInstalled(manifest.id))),
  },
])

const resultCount = computed(() => sections.value.reduce((count, section) => count + section.items.length, 0))

function active(manifest: ExtensionManifest) {
  if (!extensions.isInstalled(manifest.id)) return false
  if (manifest.activation.includes('*')) return true
  const category = workbench.selectedProject?.category
  return !!category && manifest.activation.includes(`project:${category}`)
}

function commandsFor(manifest: ExtensionManifest) {
  return extensions.commands.filter((entry) => entry.extensionId === manifest.id)
}

function toggle(id: string) {
  expanded.value = expanded.value === id ? '' : id
}

async function install(manifest: ExtensionManifest) {
  await extensions.install(manifest.id)
  expanded.value = manifest.id
}
</script>

<template>
  <div class="extensions-browser">
    <label class="extensions-search">
      <AppIcon name="search" :size="14" />
      <input v-model="query" type="search" placeholder="Search extensions" aria-label="Search extensions" />
    </label>

    <p v-if="!resultCount" class="side-empty">No extensions match your search.</p>

    <template v-for="section in sections" :key="section.id">
      <div class="extensions-heading">
        <AppIcon name="chevron-down" :size="13" />
        <span>{{ section.label }}</span>
        <span class="extensions-count">{{ section.items.length }}</span>
      </div>
      <p v-if="!section.items.length && !query" class="side-empty">
        {{ section.id === 'installed' ? 'No extensions installed.' : 'Every bundled extension is installed.' }}
      </p>

      <article
        v-for="manifest in section.items"
        :key="manifest.id"
        class="extension-card"
        :class="{ 'extension-card--expanded': expanded === manifest.id }"
      >
        <button class="extension-card__summary" :aria-expanded="expanded === manifest.id" @click="toggle(manifest.id)">
          <img class="extension-card__icon" :src="logoUrl" alt="" />
          <span class="extension-card__copy">
            <strong>{{ manifest.name }}</strong>
            <span>{{ manifest.description }}</span>
            <small>Packwand · v{{ manifest.version }}</small>
          </span>
          <span
            class="extension-card__state"
            :class="{ 'extension-card__state--inactive': !active(manifest) }"
            :title="extensions.isInstalled(manifest.id) ? active(manifest) ? 'Active for this project' : 'Installed; inactive for this project' : 'Available to install'"
          >
            <AppIcon :name="extensions.isInstalled(manifest.id) ? active(manifest) ? 'check' : 'target' : 'plus'" :size="12" />
          </span>
        </button>

        <div v-if="expanded === manifest.id" class="extension-card__details">
          <div class="extension-facts">
            <span>{{ manifest.commands.length }} commands</span>
            <span>{{ manifest.views.length }} views</span>
            <span>{{ manifest.validators.length }} validators</span>
          </div>
          <div class="extension-capabilities" aria-label="Declared capabilities">
            <span v-for="capability in manifest.capabilities" :key="capability">{{ capability }}</span>
          </div>
          <div v-if="commandsFor(manifest).length" class="extension-actions">
            <button
              v-for="entry in commandsFor(manifest)"
              :key="entry.id"
              class="extension-action"
              @click="extensions.run(entry.id)"
            >
              <AppIcon :name="entry.command.icon ?? 'play'" :size="13" />
              <span>{{ entry.command.title }}</span>
            </button>
          </div>
          <button v-if="!extensions.isInstalled(manifest.id)" class="extension-install" @click="install(manifest)">
            Install
          </button>
          <button v-else class="extension-uninstall" @click="extensions.uninstall(manifest.id)">
            Uninstall
          </button>
          <p v-if="extensions.isInstalled(manifest.id) && !active(manifest)" class="extension-card__hint">
            Select a compatible project to activate this extension.
          </p>
        </div>
      </article>
    </template>

    <div v-if="extensions.errors.length" class="extensions-errors">
      <strong>Extension errors</strong>
      <p v-for="error in extensions.errors" :key="error">{{ error }}</p>
    </div>

    <p class="extensions-footnote">Extensions are installed manually from Packwand's bundled, first-party catalog.</p>
  </div>
</template>

<style scoped>
.extensions-browser { min-height: 0; padding-bottom: 12px; }
.extensions-search { position: relative; display: flex; align-items: center; margin: 4px 8px 9px; color: var(--faint); }
.extensions-search > svg { position: absolute; left: 8px; pointer-events: none; }
.extensions-search input { height: 28px; min-height: 28px; padding: 0 8px 0 28px; border-radius: 2px; font-size: 11.5px; }
.extensions-heading { display: flex; height: 25px; align-items: center; gap: 4px; padding: 0 9px 0 6px; color: var(--text); font-size: 11px; font-weight: 700; letter-spacing: .04em; text-transform: uppercase; }
.extensions-count { display: grid; min-width: 17px; height: 17px; margin-left: auto; place-items: center; border-radius: 10px; background: var(--accent); padding: 0 5px; color: white; font-size: 9.5px; }
.extension-card { border-top: 1px solid transparent; border-bottom: 1px solid transparent; }
.extension-card:hover, .extension-card--expanded { background: var(--hover); }
.extension-card--expanded { border-color: var(--line); }
.extension-card__summary { display: grid; width: 100%; min-height: 72px; grid-template-columns: 39px minmax(0, 1fr) 18px; align-items: start; gap: 9px; border: 0; border-radius: 0; background: transparent; padding: 8px 9px; text-align: left; }
.extension-card__summary:hover { background: transparent; }
.extension-card__icon { width: 39px; height: 39px; border-radius: 9px; object-fit: cover; }
.extension-card__copy { display: flex; min-width: 0; flex-direction: column; gap: 2px; }
.extension-card__copy strong { overflow: hidden; color: var(--text-strong); font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
.extension-card__copy > span { display: -webkit-box; overflow: hidden; color: var(--muted); font-size: 10.5px; font-weight: 450; line-height: 1.25; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
.extension-card__copy small { color: var(--faint); font-size: 9.5px; font-weight: 550; }
.extension-card__state { display: grid; width: 17px; height: 17px; place-items: center; border-radius: 50%; background: var(--accent-soft); color: var(--accent-2); }
.extension-card__state--inactive { background: var(--surface-3); color: var(--faint); }
.extension-card__details { padding: 0 10px 10px 57px; }
.extension-facts { display: flex; flex-wrap: wrap; gap: 4px 9px; color: var(--faint); font-size: 9.5px; }
.extension-capabilities { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 7px; }
.extension-capabilities span { border: 1px solid var(--line-strong); border-radius: 3px; background: var(--surface-3); padding: 2px 4px; color: var(--muted); font: 8.5px var(--mono); }
.extension-actions { display: flex; flex-direction: column; gap: 2px; margin-top: 8px; }
.extension-action { display: flex; width: 100%; min-height: 25px; align-items: center; gap: 6px; justify-content: flex-start; background: transparent; padding: 0 5px; color: var(--text); font-size: 10.5px; font-weight: 550; text-align: left; }
.extension-install, .extension-uninstall { min-height: 25px; margin-top: 8px; padding: 0 10px; font-size: 10.5px; }
.extension-install { background: var(--accent); color: white; }
.extension-uninstall { border: 1px solid var(--line-strong); background: transparent; color: var(--muted); }
.extension-card__hint, .extensions-footnote { color: var(--faint); font-size: 10px; line-height: 1.4; }
.extension-card__hint { margin-top: 7px; }
.extensions-errors { margin: 10px 9px; border: 1px solid var(--danger-line); border-radius: 4px; background: var(--danger-bg); padding: 7px; color: #ff9aaa; font-size: 10px; }
.extensions-errors p { margin-top: 4px; }
.extensions-footnote { padding: 12px 10px 0; }
</style>
