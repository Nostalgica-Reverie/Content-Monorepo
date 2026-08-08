<script setup lang="ts">
// The themes sub-menu: a gallery you click into, rather than a dropdown buried
// in the settings body.
//
// Each card renders the real chrome — rail, tab strip, editor line, status bar
// — in that theme's own resolved tokens, so the card is the thing itself rather
// than a row of colour swatches. Those tokens are inline on the card, so a card
// always paints its own theme regardless of which one is active.
//
// Hovering or focusing a card additionally applies that theme to the whole
// window through the store's `preview`, and leaving restores the real one.
// That is a deliberate live preview, not an accident: judging a theme from a
// thumbnail is much harder than seeing the editor wear it. Only clicking
// persists the choice.
import { computed, ref } from 'vue'
import Button from '@/components/ui/Button.vue'
import ThemeWorkshop from '@/components/settings/ThemeWorkshop.vue'
import { resolveTheme, validateTheme } from '@/themes/theme'
import { themeTokenNames, type ResolvedTheme } from '@/themes/types'
import { useThemeStore } from '@/stores/theme'
import { useToastsStore } from '@/stores/toasts'

const themeStore = useThemeStore()
const toasts = useToastsStore()

/** `gallery` lists themes; `customize` is the existing workshop. */
const pane = ref<'gallery' | 'customize'>('gallery')
const filter = ref('')

// `themes` already merges the bundled set with anything the user imported.
// Each entry keeps its declaration alongside the resolved tokens: the card
// paints from `resolved`, while `preview` takes the declaration.
const available = computed(() =>
  themeStore.themes.map(theme => ({ theme, resolved: resolveTheme(theme) })),
)

const shown = computed(() => {
  const needle = filter.value.trim().toLowerCase()
  if (!needle) return available.value
  return available.value.filter(
    ({ resolved }) =>
      resolved.name.toLowerCase().includes(needle) || resolved.id.toLowerCase().includes(needle),
  )
})

const activeId = computed(() => themeStore.currentId)

/** Resolved tokens as inline custom properties, scoped to one preview card. */
function previewStyle(theme: ResolvedTheme): Record<string, string> {
  const style: Record<string, string> = {}
  for (const token of themeTokenNames) style[`--${token}`] = theme.colors[token]
  return style
}

/** How many of this theme's contrast pairs still fall short. */
function warningCount(theme: ResolvedTheme): number {
  return validateTheme(theme).warnings.filter(
    warning => !warning.startsWith('Bundled theme IDs are reserved'),
  ).length
}

async function choose(theme: ResolvedTheme) {
  if (theme.id === activeId.value) return
  try {
    await themeStore.setTheme(theme.id)
    toasts.push('Theme applied', theme.name, 'success')
  } catch (error) {
    toasts.push('Could not apply theme', String(error), 'danger')
  }
}
</script>

<template>
  <article class="panel span-12 theme-gallery-panel">
    <div class="panel-head">
      <h2>Appearance</h2>
      <div class="panel-actions">
        <Button :variant="pane === 'gallery' ? 'primary' : 'quiet'" @click="pane = 'gallery'">
          Themes
        </Button>
        <Button :variant="pane === 'customize' ? 'primary' : 'quiet'" @click="pane = 'customize'">
          Customize
        </Button>
      </div>
    </div>

    <template v-if="pane === 'gallery'">
      <label class="field-stack theme-filter">
        <span class="sr-only">Filter themes</span>
        <input v-model="filter" type="search" placeholder="Filter themes by name" />
      </label>

      <ul class="theme-grid" role="list">
        <li v-for="entry in shown" :key="entry.resolved.id">
          <button
            type="button"
            class="theme-card"
            :class="{ 'is-active': entry.resolved.id === activeId }"
            :aria-pressed="entry.resolved.id === activeId"
            @click="choose(entry.resolved)"
            @mouseenter="themeStore.preview(entry.theme)"
            @mouseleave="themeStore.cancelPreview()"
            @focus="themeStore.preview(entry.theme)"
            @blur="themeStore.cancelPreview()"
          >
            <!-- The preview is real chrome in this theme's tokens, so what you
                 see is what the shell will look like. -->
            <span class="theme-preview" :style="previewStyle(entry.resolved)" aria-hidden="true">
              <span class="preview-rail" />
              <span class="preview-body">
                <span class="preview-tabs">
                  <span class="preview-tab is-active" />
                  <span class="preview-tab" />
                </span>
                <span class="preview-editor">
                  <span class="preview-line w-70" />
                  <span class="preview-line w-45 accented" />
                  <span class="preview-line w-60" />
                </span>
                <span class="preview-status" />
              </span>
            </span>
            <span class="theme-meta">
              <strong>{{ entry.resolved.name }}</strong>
              <small>{{ entry.resolved.appearance }}</small>
            </span>
            <span v-if="entry.resolved.id === activeId" class="theme-flag">Active</span>
            <span v-else-if="warningCount(entry.resolved)" class="theme-flag warn">
              {{ warningCount(entry.resolved) }} contrast
              warning{{ warningCount(entry.resolved) === 1 ? '' : 's' }}
            </span>
          </button>
        </li>
      </ul>
      <p v-if="!shown.length" class="panel-copy">No theme matches “{{ filter }}”.</p>
    </template>

    <ThemeWorkshop v-else class="theme-workshop-embedded" />
  </article>
</template>

<style scoped>
.theme-filter { max-width: 22rem; }

.theme-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(15rem, 1fr));
  gap: 0.75rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

.theme-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem;
  text-align: left;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 0.5rem;
  cursor: pointer;
  transition: border-color var(--motion-fast) var(--ease-standard),
    transform var(--motion-fast) var(--ease-standard);
}
.theme-card:hover { border-color: var(--accent-line); }
/* Transform only — animating size or margin here would reflow the grid. */
.theme-card:active { transform: scale(0.99); }
.theme-card.is-active { border-color: var(--accent); }
.theme-card:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

/* The preview paints itself from the inline custom properties above, so it is
   deliberately independent of the app's current theme. */
.theme-preview {
  display: flex;
  height: 6.5rem;
  overflow: hidden;
  background: var(--bg);
  border: 1px solid var(--line-soft);
  border-radius: 0.375rem;
}
.preview-rail { width: 1.25rem; background: var(--rail); border-right: 1px solid var(--line-soft); }
.preview-body { display: flex; flex: 1; flex-direction: column; }
.preview-tabs { display: flex; gap: 1px; height: 1.1rem; background: var(--side); }
.preview-tab { width: 3rem; background: var(--surface-2); }
.preview-tab.is-active { background: var(--surface); border-top: 2px solid var(--accent); }
.preview-editor {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 0.35rem;
  padding: 0.5rem;
  background: var(--surface-3);
}
.preview-line { height: 0.35rem; background: var(--muted); border-radius: 999px; }
.preview-line.accented { background: var(--accent); }
.w-70 { width: 70%; }
.w-45 { width: 45%; }
.w-60 { width: 60%; }
.preview-status { height: 0.9rem; background: var(--accent-dim); }

.theme-meta { display: flex; align-items: baseline; justify-content: space-between; gap: 0.5rem; }
.theme-meta strong { color: var(--text-strong); font-size: 0.85rem; }
.theme-meta small { color: var(--faint); font-size: 0.7rem; text-transform: capitalize; }

.theme-flag {
  position: absolute;
  top: 0.75rem;
  right: 0.75rem;
  padding: 0.1rem 0.4rem;
  color: var(--bg);
  font-size: 0.65rem;
  background: var(--accent);
  border-radius: 999px;
}
.theme-flag.warn { color: var(--text-strong); background: var(--warning); }

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
}
</style>
