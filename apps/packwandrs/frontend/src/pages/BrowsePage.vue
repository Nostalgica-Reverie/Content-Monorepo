<script setup lang="ts">
// Find mods without leaving the window.
//
// The provider websites cannot be embedded: modrinth.com sends
// `X-Frame-Options: DENY`, and both CurseForge hosts send `SAMEORIGIN` plus a
// Cloudflare challenge to anything that is not a real browser. So this renders
// search results natively instead — which is also faster, themeable, and one
// click from adding a mod to the open pack. "Open on the site" hands off to
// the system browser for the cases a listing cannot cover.
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import ProjectDetail from '@/components/browse/ProjectDetail.vue'
import { providerAdd, providerBrowse, providerOpenPage } from '@/helpers/invoke/providers'
import { normalizeBridgeError } from '@/helpers/errors'
import type { BrowseProject, ProviderKind } from '@/helpers/types'
import { useToastsStore } from '@/stores/toasts'
import { useWorkbenchStore } from '@/stores/workbench'

const toasts = useToastsStore()
const workbench = useWorkbenchStore()

const PROVIDERS: Array<{ id: ProviderKind; label: string }> = [
	{ id: 'modrinth', label: 'Modrinth' },
	{ id: 'curse_forge', label: 'CurseForge' },
]

const PAGE_SIZE = 20
/**
 * How long the search box waits after the last keystroke.
 *
 * Providers meter requests per minute — Modrinth allows 300 — and the shared
 * transport already paces them, so an un-debounced box would spend the whole
 * budget queueing behind itself and make every result feel slow.
 */
const DEBOUNCE_MS = 350

const provider = ref<ProviderKind>('modrinth')
const text = ref('')
const offset = ref(0)
const results = ref<BrowseProject[]>([])
const total = ref(0)
const loading = ref(false)
const failure = ref<string | null>(null)
const adding = ref<string | null>(null)
/** The result whose page is open beside the list, if any. */
const selected = ref<BrowseProject | null>(null)

/** Filters taken from whichever pack is open, so results are relevant. */
const packLoaders = computed(() => workbench.selectedPack?.loaders ?? [])
const packVersion = computed(() => workbench.selectedPack?.minecraftVersion ?? null)
const scopeLabel = computed(() =>
	packLoaders.value.length || packVersion.value
		? [packVersion.value, ...packLoaders.value].filter(Boolean).join(' · ')
		: 'no pack filters',
)

let timer: ReturnType<typeof setTimeout> | null = null

async function run() {
	loading.value = true
	failure.value = null
	try {
		const page = await providerBrowse(
			provider.value,
			{
				text: text.value.trim(),
				loaders: packLoaders.value,
				gameVersions: packVersion.value ? [packVersion.value] : [],
				offset: offset.value,
				limit: PAGE_SIZE,
			},
			null,
		)
		results.value = page.projects
		total.value = page.total
		// The open project belongs to the previous result set; keeping it beside
		// an unrelated list is more confusing than closing it.
		selected.value = null
	} catch (error) {
		// A failed search must not leave stale results looking current.
		results.value = []
		total.value = 0
		failure.value = normalizeBridgeError(error).message
	} finally {
		loading.value = false
	}
}

/**
 * Coalesces keystrokes into one request.
 *
 * Paging deliberately does *not* go through here — it calls `run()` directly.
 * An earlier version watched `offset` as well, which meant a keystroke while
 * paginated reset the offset, fired an immediate request from that watcher,
 * and then fired the debounced one too: two requests against a metered API for
 * one character typed.
 */
function schedule() {
	offset.value = 0
	if (timer) clearTimeout(timer)
	timer = setTimeout(() => {
		timer = null
		void run()
	}, DEBOUNCE_MS)
}

/** Moves the page and fetches it, with no debounce — a click is deliberate. */
function page(delta: number) {
	offset.value = Math.max(0, offset.value + delta)
	void run()
}

watch(text, () => schedule())
watch(provider, () => schedule())
watch(
	() => workbench.selectedPackId,
	() => schedule(),
)
onBeforeUnmount(() => {
	if (timer) clearTimeout(timer)
})
void run()

const canPageBack = computed(() => offset.value > 0)
const canPageForward = computed(() => offset.value + PAGE_SIZE < total.value)

async function add(project: BrowseProject) {
	if (!workbench.selectedPackId) {
		toasts.push('No pack selected', 'Open a pack before adding mods to it.', 'danger')
		return
	}
	adding.value = project.id
	try {
		const path = await providerAdd(
			workbench.selectedPackId,
			provider.value,
			{
				project: project.id,
				game_versions: packVersion.value ? [packVersion.value] : [],
				loaders: packLoaders.value,
				channels: ['release', 'beta', 'alpha'],
				branch: null,
				asset_pattern: null,
			},
			null,
			null,
		)
		await workbench.refresh()
		toasts.push('Added to pack', `${project.title} → ${path}`, 'success')
	} catch (error) {
		toasts.push(`Could not add ${project.title}`, normalizeBridgeError(error).message, 'danger')
	} finally {
		adding.value = null
	}
}

/**
 * Hands a project page to the system browser.
 *
 * Not an `<a target="_blank">`: the app ships no opener plugin, so the webview
 * silently swallows external navigation and the link would look broken. The
 * Rust side validates the scheme and host before opening anything.
 */
async function openPage(url: string) {
	try {
		await providerOpenPage(url)
	} catch (error) {
		toasts.push('Could not open the page', normalizeBridgeError(error).message, 'danger')
	}
}

const numberFormat = new Intl.NumberFormat()
const downloads = (count: number) => numberFormat.format(count)
</script>

<template>
	<section class="grid view-grid browse-view">
		<article class="panel span-12">
			<div class="panel-head">
				<h2>Browse</h2>
				<span class="status-badge">{{ scopeLabel }}</span>
			</div>

			<div class="browse-controls">
				<div class="provider-switch" role="tablist" aria-label="Provider">
					<button
						v-for="option in PROVIDERS"
						:key="option.id"
						type="button"
						role="tab"
						:aria-selected="provider === option.id"
						:class="['provider-tab', { 'is-active': provider === option.id }]"
						@click="provider = option.id"
					>
						{{ option.label }}
					</button>
				</div>
				<label class="browse-search">
					<span class="sr-only"
						>Search {{ provider === 'modrinth' ? 'Modrinth' : 'CurseForge' }}</span
					>
					<input v-model="text" type="search" placeholder="Search for a mod…" />
				</label>
			</div>

			<p v-if="failure" class="notice danger-notice">{{ failure }}</p>

			<div class="browse-split">
				<ul v-if="results.length" class="result-list" role="list">
					<li
						v-for="project in results"
						:key="project.id"
						:class="['result', { 'is-open': selected?.id === project.id }]"
						@click="selected = project"
					>
						<img
							v-if="project.iconUrl"
							:src="project.iconUrl"
							alt=""
							class="result-icon"
							loading="lazy"
						/>
						<span v-else class="result-icon placeholder" aria-hidden="true" />
						<div class="result-body">
							<div class="result-head">
								<strong>{{ project.title }}</strong>
								<small>{{ downloads(project.downloads) }} downloads</small>
							</div>
							<p class="result-summary">{{ project.summary }}</p>
							<div class="result-meta">
								<span v-if="project.author">by {{ project.author }}</span>
								<span v-if="project.license">{{ project.license }}</span>
								<span v-for="loader in project.loaders.slice(0, 4)" :key="loader" class="chip">{{
									loader
								}}</span>
							</div>
						</div>
						<div class="result-actions">
							<Button :busy="adding === project.id" @click.stop="add(project)">Add to pack</Button>
							<button type="button" class="result-link" @click.stop="openPage(project.pageUrl)">
								Open on site
							</button>
							<!-- Legacy CurseForge is the same catalogue behind an older front
                 end, so it is a link on the result rather than a third tab. -->
							<button
								v-if="project.legacyPageUrl"
								type="button"
								class="result-link"
								@click.stop="openPage(project.legacyPageUrl)"
							>
								Legacy CurseForge
							</button>
						</div>
					</li>
				</ul>
				<ProjectDetail
					v-if="selected"
					:provider="provider"
					:summary="selected"
					:adding="adding === selected.id"
					@close="selected = null"
					@add="add"
				/>
			</div>

			<!-- Sibling of the split, not of the list inside it: the empty state
           replaces the whole results area, detail panel included. -->
			<EmptyState
				v-if="!results.length && !loading && !failure"
				title="Nothing found"
				:message="text.trim() ? `No results for “${text}”.` : 'Type to search this provider.'"
			/>

			<div v-if="results.length" class="action-row panel-bottom-actions">
				<Button variant="quiet" :disabled="!canPageBack" @click="page(-PAGE_SIZE)">
					Previous
				</Button>
				<span class="page-indicator">
					{{ offset + 1 }}–{{ offset + results.length
					}}<template v-if="total"> of {{ downloads(total) }}</template>
				</span>
				<Button variant="quiet" :disabled="!canPageForward" @click="page(PAGE_SIZE)"> Next </Button>
			</div>
		</article>
	</section>
</template>

<style scoped>
.browse-controls {
	display: flex;
	gap: 0.75rem;
	align-items: center;
	flex-wrap: wrap;
}
.browse-search {
	flex: 1;
	min-width: 14rem;
}
.browse-search input {
	width: 100%;
}

.provider-switch {
	display: inline-flex;
	gap: 2px;
	padding: 2px;
	background: var(--surface-2);
	border-radius: 0.4rem;
}
.provider-tab {
	padding: 0.3rem 0.7rem;
	color: var(--muted);
	font-size: 0.8rem;
	background: transparent;
	border: 0;
	border-radius: 0.3rem;
	cursor: pointer;
	transition:
		background var(--motion-fast) var(--ease-standard),
		color var(--motion-fast) var(--ease-standard);
}
.provider-tab.is-active {
	color: var(--text-strong);
	background: var(--surface);
}

.browse-split {
	display: grid;
	grid-template-columns: 1fr;
	gap: 0.75rem;
}
/* The detail panel only earns a column when there is room for both. */
@media (min-width: 62rem) {
	.browse-split:has(.project-detail) {
		grid-template-columns: minmax(0, 1fr) minmax(20rem, 26rem);
	}
}
.project-detail {
	max-height: 34rem;
	border-radius: 0.5rem;
	border: 1px solid var(--line);
}

.result-list {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	margin: 0.75rem 0 0;
	padding: 0;
	list-style: none;
}
.result {
	display: flex;
	gap: 0.75rem;
	padding: 0.6rem;
	background: var(--surface);
	border: 1px solid var(--line-soft);
	border-radius: 0.5rem;
	transition: border-color var(--motion-fast) var(--ease-standard);
}
.result:hover {
	border-color: var(--accent-line);
}
.result {
	cursor: pointer;
}
.result.is-open {
	border-color: var(--accent);
}

.result-icon {
	width: 3rem;
	height: 3rem;
	object-fit: cover;
	border-radius: 0.375rem;
}
.result-icon.placeholder {
	background: var(--surface-2);
}
.result-body {
	flex: 1;
	min-width: 0;
}
.result-head {
	display: flex;
	align-items: baseline;
	gap: 0.5rem;
	justify-content: space-between;
}
.result-head strong {
	color: var(--text-strong);
}
.result-head small {
	color: var(--faint);
	font-size: 0.7rem;
	white-space: nowrap;
}
.result-summary {
	margin: 0.2rem 0 0.35rem;
	color: var(--muted);
	font-size: 0.8rem;
	/* Two lines, so a long description cannot push the row out of the list. */
	display: -webkit-box;
	-webkit-box-orient: vertical;
	-webkit-line-clamp: 2;
	overflow: hidden;
}
.result-meta {
	display: flex;
	flex-wrap: wrap;
	gap: 0.4rem;
	color: var(--faint);
	font-size: 0.7rem;
}
.chip {
	padding: 0.05rem 0.35rem;
	background: var(--surface-2);
	border-radius: 999px;
}

.result-actions {
	display: flex;
	flex-direction: column;
	gap: 0.3rem;
	align-items: stretch;
}
.result-link {
	color: var(--accent-2);
	font-size: 0.7rem;
	text-align: center;
	background: none;
	border: 0;
	cursor: pointer;
}
.result-link:hover {
	text-decoration: underline;
}

.page-indicator {
	color: var(--faint);
	font-size: 0.75rem;
}

.sr-only {
	position: absolute;
	width: 1px;
	height: 1px;
	overflow: hidden;
	clip-path: inset(50%);
}
</style>
