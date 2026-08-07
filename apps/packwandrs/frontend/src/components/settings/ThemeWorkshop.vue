<script setup lang="ts">
import { computed, ref } from 'vue'

import Button from '@/components/ui/Button.vue'
import { useThemeStore } from '@/stores/theme'
import { validateTheme } from '@/themes/theme'
import { themeFileSuffix, themeTokenNames, type PackwandTheme } from '@/themes/types'

const themes = useThemeStore()
const draft = ref<PackwandTheme | null>(null)
const source = ref('')
const parseError = ref('')
const saving = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)
const validation = computed(() => draft.value ? validateTheme(draft.value) : null)

function edit(theme: PackwandTheme) {
  draft.value = theme.id.startsWith('builtin.') ? themes.duplicate(theme) : structuredClone(theme)
  source.value = JSON.stringify(draft.value, null, 2)
  parseError.value = ''
  themes.preview(draft.value)
}

function updateSource(preview = true) {
  if (!draft.value) return
  source.value = JSON.stringify(draft.value, null, 2)
  parseError.value = ''
  if (preview) themes.preview(draft.value)
}

function updateFromSource() {
  try {
    draft.value = JSON.parse(source.value) as PackwandTheme
    parseError.value = ''
    themes.preview(draft.value)
  } catch (error) {
    parseError.value = String(error)
  }
}

function setColor(token: (typeof themeTokenNames)[number], event: Event) {
  if (!draft.value) return
  draft.value.colors[token] = (event.target as HTMLInputElement).value
  updateSource()
}

async function save() {
  if (!draft.value || parseError.value || !validation.value?.valid) return
  saving.value = true
  try {
    draft.value = await themes.saveCustom(draft.value)
    updateSource(false)
  } finally {
    saving.value = false
  }
}

function cancel() {
  draft.value = null
  source.value = ''
  parseError.value = ''
  themes.cancelPreview()
}

async function remove() {
  if (!draft.value?.id.startsWith('user.') || !window.confirm(`Delete ${draft.value.name}?`)) return
  await themes.removeCustom(draft.value.id)
  cancel()
}

async function importFile(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  try {
    draft.value = themes.importText(await file.text())
    updateSource()
  } catch (error) {
    parseError.value = String(error)
  } finally {
    input.value = ''
  }
}

function exportTheme() {
  const theme = draft.value ?? themes.currentTheme
  const blob = new Blob([themes.exportText(theme)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${theme.id}${themeFileSuffix}`
  link.click()
  URL.revokeObjectURL(url)
}
</script>

<template>
  <article class="panel span-12 theme-workshop">
    <div class="panel-head">
      <div><h2>Theme Workshop</h2><p class="panel-copy">Themes color the complete Packwand shell and editor. Custom themes are safe JSON stored in your app configuration.</p></div>
      <div class="panel-actions">
        <input ref="fileInput" class="visually-hidden" type="file" :accept="themeFileSuffix" @change="importFile" />
        <Button variant="quiet" @click="fileInput?.click()">Import JSON</Button>
        <Button variant="quiet" @click="exportTheme">Export JSON</Button>
      </div>
    </div>
    <div class="theme-gallery">
      <button v-for="theme in themes.themes" :key="theme.id" type="button" class="theme-card" :class="{ active: themes.currentId === theme.id }" @click="themes.setTheme(theme.id)" @dblclick="edit(theme)">
        <span class="theme-card__swatches"><i v-for="token in ['bg', 'surface', 'accent', 'text']" :key="token" :style="{ background: theme.colors[token as keyof typeof theme.colors] }" /></span>
        <strong>{{ theme.name }}</strong><small>{{ theme.appearance }} &middot; {{ theme.id.startsWith('builtin.') ? 'built in' : 'custom' }}</small>
        <span class="theme-card__actions" @click.stop><Button variant="quiet" @click="edit(theme)">{{ theme.id.startsWith('builtin.') ? 'Duplicate' : 'Edit' }}</Button></span>
      </button>
    </div>
    <p v-for="diagnostic in themes.diagnostics" :key="diagnostic" class="notice">{{ diagnostic }}</p>
    <div v-if="draft" class="theme-editor">
      <div class="theme-editor__form">
        <label class="field-stack"><span>Name</span><input v-model="draft.name" @input="updateSource()" /></label>
        <label class="field-stack"><span>ID</span><input v-model="draft.id" @input="updateSource(false)" /></label>
        <label class="field-stack"><span>Appearance</span><select v-model="draft.appearance" @change="updateSource()"><option value="dark">Dark</option><option value="light">Light</option><option value="high-contrast">High contrast</option></select></label>
        <div class="theme-colors">
          <label v-for="token in themeTokenNames" :key="token"><span>{{ token }}</span><input type="color" :value="draft.colors[token]?.slice(0, 7)" @input="setColor(token, $event)" /><code>{{ draft.colors[token] }}</code></label>
        </div>
      </div>
      <div class="theme-editor__json">
        <label class="field-stack"><span>Theme JSON</span><textarea v-model="source" spellcheck="false" @input="updateFromSource" /></label>
        <p v-if="parseError" class="error-banner">{{ parseError }}</p>
        <p v-for="message in validation?.errors" :key="message" class="error-banner">{{ message }}</p>
        <p v-for="message in validation?.warnings" :key="message" class="notice">{{ message }}</p>
      </div>
      <div class="panel-actions theme-editor__actions">
        <Button variant="quiet" @click="cancel">Cancel preview</Button>
        <Button v-if="draft.id.startsWith('user.') && themes.themes.some(theme => theme.id === draft?.id)" variant="danger" @click="remove">Delete</Button>
        <Button :busy="saving" :disabled="Boolean(parseError) || !validation?.valid" @click="save">Save theme</Button>
      </div>
    </div>
  </article>
</template>

<style scoped lang="scss">
.theme-gallery { display: grid; grid-template-columns: repeat(auto-fill, minmax(190px, 1fr)); gap: 8px; }
.theme-card { display: grid; gap: 6px; border: 1px solid var(--line); border-radius: var(--radius); background: var(--surface); padding: 12px; color: var(--text); text-align: left; }
.theme-card:hover, .theme-card.active { border-color: var(--accent); background: var(--surface-2); }
.theme-card small { color: var(--muted); }
.theme-card__swatches { display: flex; height: 22px; overflow: hidden; border-radius: 4px; }
.theme-card__swatches i { flex: 1; }
.theme-card__actions { justify-self: end; }
.theme-editor { display: grid; grid-template-columns: minmax(320px, 1fr) minmax(360px, 1fr); gap: 16px; margin-top: 18px; }
.theme-editor__form { display: grid; align-content: start; gap: 10px; }
.theme-colors { display: grid; max-height: 420px; overflow: auto; grid-template-columns: repeat(2, 1fr); gap: 5px; }
.theme-colors label { display: grid; grid-template-columns: minmax(0, 1fr) 28px; align-items: center; gap: 6px; background: var(--surface-2); padding: 6px; }
.theme-colors input { width: 28px; height: 24px; border: 0; padding: 0; }
.theme-colors code { grid-column: 1 / -1; color: var(--faint); font-size: 10px; }
.theme-editor__json textarea { min-height: 480px; resize: vertical; font: 11px/1.5 var(--mono); }
.theme-editor__actions { grid-column: 1 / -1; justify-content: flex-end; }
@media (max-width: 900px) { .theme-editor { grid-template-columns: 1fr; } .theme-editor__actions { grid-column: 1; } }
</style>
