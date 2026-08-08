<script setup lang="ts">
// Reading a mod's page without leaving the app.
//
// The provider sites cannot be embedded (they refuse framing), so this renders
// the project's own description instead. The markup arrives already sanitized:
// `providers_project` runs it through `commands::richtext` in Rust, which
// strips scripts, event handlers, non-http schemes, iframes and off-CDN
// images before the webview sees a byte. That is why `v-html` is acceptable
// here and would not be anywhere else in this app.
import { computed, ref, watch } from 'vue'
import Button from '@/components/ui/Button.vue'
import { providerOpenPage, providerProject } from '@/helpers/invoke/providers'
import { normalizeBridgeError } from '@/helpers/errors'
import type { BrowseProject, ProjectPage, ProviderKind } from '@/helpers/types'
import { useToastsStore } from '@/stores/toasts'

const props = defineProps<{
  provider: ProviderKind
  /** The search result that was clicked, shown while the full page loads. */
  summary: BrowseProject
  adding: boolean
}>()
const emit = defineEmits<{ close: []; add: [BrowseProject] }>()

const toasts = useToastsStore()
const page = ref<ProjectPage | null>(null)
const loading = ref(false)
const failure = ref<string | null>(null)

/**
 * The header falls back to the search result while the full page is in
 * flight, so opening a project shows something immediately rather than an
 * empty panel — and so a failed fetch still leaves a usable card.
 */
const shown = computed(() => page.value?.project ?? props.summary)

const links = computed(() =>
  [
    { label: 'Project page', url: shown.value.pageUrl },
    { label: 'Legacy CurseForge', url: shown.value.legacyPageUrl },
    { label: 'Source', url: page.value?.sourceUrl },
    { label: 'Issues', url: page.value?.issuesUrl },
    { label: 'Wiki', url: page.value?.wikiUrl },
  ].filter((link): link is { label: string; url: string } => Boolean(link.url)),
)

async function load() {
  loading.value = true
  failure.value = null
  page.value = null
  try {
    page.value = await providerProject(props.provider, props.summary.id)
  } catch (error) {
    failure.value = normalizeBridgeError(error).message
  } finally {
    loading.value = false
  }
}

watch(() => [props.provider, props.summary.id], load, { immediate: true })

async function open(url: string) {
  try {
    await providerOpenPage(url)
  } catch (error) {
    toasts.push('Could not open the page', normalizeBridgeError(error).message, 'danger')
  }
}

const numberFormat = new Intl.NumberFormat()
</script>

<template>
  <aside class="project-detail" aria-label="Project details">
    <header class="detail-head">
      <img v-if="shown.iconUrl" :src="shown.iconUrl" alt="" class="detail-icon" />
      <span v-else class="detail-icon placeholder" aria-hidden="true" />
      <div class="detail-title">
        <strong>{{ shown.title }}</strong>
        <small>{{ numberFormat.format(shown.downloads) }} downloads<template v-if="shown.author"> · {{ shown.author }}</template></small>
      </div>
      <Button variant="quiet" aria-label="Close details" @click="emit('close')">✕</Button>
    </header>

    <p class="detail-summary">{{ shown.summary }}</p>

    <div class="detail-meta">
      <span v-if="shown.license" class="chip">{{ shown.license }}</span>
      <span v-for="loader in shown.loaders.slice(0, 6)" :key="loader" class="chip">{{ loader }}</span>
      <span v-for="version in shown.gameVersions.slice(0, 6)" :key="version" class="chip subtle">{{ version }}</span>
    </div>

    <div class="detail-actions">
      <Button :busy="adding" @click="emit('add', shown)">Add to pack</Button>
      <button v-for="link in links" :key="link.label" type="button" class="detail-link" @click="open(link.url)">
        {{ link.label }}
      </button>
    </div>

    <p v-if="loading" class="panel-copy">Loading description…</p>
    <p v-else-if="failure" class="notice danger-notice">{{ failure }}</p>

    <template v-else-if="page">
      <ul v-if="page.gallery.length" class="detail-gallery" role="list">
        <li v-for="image in page.gallery.slice(0, 6)" :key="image.url">
          <img :src="image.url" :alt="image.title" loading="lazy" />
        </li>
      </ul>

      <!-- Sanitized in Rust before it crossed the IPC boundary. -->
      <!-- eslint-disable-next-line vue/no-v-html -->
      <div v-if="page.bodyHtml" class="detail-body" v-html="page.bodyHtml" />
      <p v-else class="panel-copy">This project has no description.</p>
    </template>
  </aside>
</template>

<style scoped>
.project-detail {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  height: 100%;
  padding: 0.75rem;
  overflow-y: auto;
  background: var(--surface);
  border-left: 1px solid var(--line);
}

.detail-head { display: flex; gap: 0.6rem; align-items: flex-start; }
.detail-icon { width: 3rem; height: 3rem; object-fit: cover; border-radius: 0.375rem; }
.detail-icon.placeholder { background: var(--surface-2); }
.detail-title { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.detail-title strong { color: var(--text-strong); }
.detail-title small { color: var(--faint); font-size: 0.7rem; }

.detail-summary { margin: 0; color: var(--muted); font-size: 0.8rem; }
.detail-meta { display: flex; flex-wrap: wrap; gap: 0.3rem; }
.chip { padding: 0.05rem 0.35rem; color: var(--muted); font-size: 0.68rem; background: var(--surface-2); border-radius: 999px; }
.chip.subtle { color: var(--faint); }

.detail-actions { display: flex; flex-wrap: wrap; gap: 0.4rem; align-items: center; }
.detail-link {
  color: var(--accent-2);
  font-size: 0.72rem;
  background: none;
  border: 0;
  cursor: pointer;
}
.detail-link:hover { text-decoration: underline; }

.detail-gallery { display: flex; gap: 0.4rem; margin: 0; padding: 0; overflow-x: auto; list-style: none; }
.detail-gallery img { height: 5.5rem; border-radius: 0.3rem; }

/* Third-party markup: constrained so a description cannot break the panel out
   of its column or scroll the whole window sideways. */
.detail-body {
  color: var(--text);
  font-size: 0.8rem;
  line-height: 1.55;
  overflow-wrap: anywhere;
}
.detail-body :deep(img) { max-width: 100%; height: auto; border-radius: 0.3rem; }
.detail-body :deep(h1),
.detail-body :deep(h2),
.detail-body :deep(h3) { margin: 0.8rem 0 0.3rem; color: var(--text-strong); font-size: 0.95rem; }
.detail-body :deep(p) { margin: 0.4rem 0; }
.detail-body :deep(a) { color: var(--accent-2); }
.detail-body :deep(code) {
  padding: 0.05rem 0.25rem;
  font-size: 0.75rem;
  background: var(--surface-2);
  border-radius: 0.2rem;
}
.detail-body :deep(pre) { padding: 0.5rem; overflow-x: auto; background: var(--surface-3); border-radius: 0.3rem; }
.detail-body :deep(table) { display: block; overflow-x: auto; border-collapse: collapse; }
.detail-body :deep(td),
.detail-body :deep(th) { padding: 0.2rem 0.4rem; border: 1px solid var(--line-soft); }
.detail-body :deep(blockquote) {
  margin: 0.4rem 0;
  padding-left: 0.6rem;
  color: var(--muted);
  border-left: 2px solid var(--line-strong);
}
</style>
