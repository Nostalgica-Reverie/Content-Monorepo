<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import LogViewer from '@/components/ui/LogViewer.vue'
import Tabs from '@/components/ui/Tabs.vue'
import { useInstanceLaunch } from '@/composables/instances/useInstanceLaunch'
import { inheritedPlaceholder } from '@/core/packwand'
import {
	instancesContentList,
	instancesContentRemove,
	instancesContentToggle,
	instancesExport,
	instancesFileRead,
	instancesFilesList,
	instancesFileWrite,
	instancesImage,
	instancesManualPending,
	instancesManualProvide,
} from '@/helpers/invoke/instances'
import { providerOpenPage } from '@/helpers/invoke/providers'
import { jobsGet } from '@/helpers/invoke/jobs'
import { useInstancesStore } from '@/stores/instances'
import { useSettingsStore } from '@/stores/settings'
import { useToastsStore } from '@/stores/toasts'
import type { InstanceFileEntry, PendingManualDownload } from '@/helpers/types'

const route = useRoute()
const router = useRouter()
const store = useInstancesStore()
const defaults = useSettingsStore()
const toasts = useToastsStore()
const id = computed(() => String(route.params.id))
const active = ref('content')
const busy = ref(false)
const logs = ref<string[]>([])
const files = ref<InstanceFileEntry[]>([])
const pendingManual = ref<PendingManualDownload[]>([])
const providing = ref('')
const selectedFile = ref('')
const fileContent = ref('')
const iconUrl = ref('')
const backgroundUrl = ref('')
const settings = reactive({
	javaPath: '',
	memoryMinMb: '',
	memoryMaxMb: '',
	extraJvmArgs: '',
	extraGameArgs: '',
	env: '',
	windowWidth: '',
	windowHeight: '',
	fullscreen: false,
})
const launch = useInstanceLaunch(id.value)
const tabs = [
	{ id: 'content', label: 'Content' },
	{ id: 'settings', label: 'Settings' },
	{ id: 'logs', label: 'Logs' },
	{ id: 'files', label: 'Files' },
]

function populate() {
	const value = store.current?.settings
	if (!value) return
	settings.javaPath = value.javaPath ?? ''
	settings.memoryMinMb = value.memoryMinMb?.toString() ?? ''
	settings.memoryMaxMb = value.memoryMaxMb?.toString() ?? ''
	settings.extraJvmArgs = value.extraJvmArgs?.join('\n') ?? ''
	settings.extraGameArgs = value.extraGameArgs?.join('\n') ?? ''
	settings.env = value.env
		? Object.entries(value.env)
				.map(([key, item]) => `${key}=${item}`)
				.join('\n')
		: ''
	settings.windowWidth = value.windowWidth?.toString() ?? ''
	settings.windowHeight = value.windowHeight?.toString() ?? ''
	settings.fullscreen = value.fullscreen ?? false
}

function optionalNumber(value: string): number | null {
	return value.trim() ? Number(value) : null
}
function lines(value: string): string[] | null {
	const result = value
		.split('\n')
		.map((item) => item.trim())
		.filter(Boolean)
	return result.length ? result : null
}
function environment(value: string): Record<string, string> | null {
	const entries = value
		.split('\n')
		.map((line) => line.trim())
		.filter(Boolean)
		.map((line) => {
			const at = line.indexOf('=')
			return [line.slice(0, at), line.slice(at + 1)] as const
		})
		.filter(([key]) => key)
	return entries.length ? Object.fromEntries(entries) : null
}

async function saveSettings() {
	busy.value = true
	try {
		await store.edit(id.value, {
			settings: {
				javaPath: String(settings.javaPath).trim() || null,
				memoryMinMb: optionalNumber(String(settings.memoryMinMb)),
				memoryMaxMb: optionalNumber(String(settings.memoryMaxMb)),
				extraJvmArgs: lines(String(settings.extraJvmArgs)),
				extraGameArgs: lines(String(settings.extraGameArgs)),
				env: environment(String(settings.env)),
				windowWidth: optionalNumber(String(settings.windowWidth)),
				windowHeight: optionalNumber(String(settings.windowHeight)),
				fullscreen: Boolean(settings.fullscreen),
			},
		})
		toasts.push('Settings saved', 'The next launch will use these settings.', 'success')
	} catch (error) {
		toasts.push('Could not save settings', String(error), 'danger')
	} finally {
		busy.value = false
	}
}

async function toggle(path: string) {
	await instancesContentToggle(id.value, path)
	store.content = await instancesContentList(id.value)
}
async function remove(path: string) {
	if (!confirm(`Remove ${path} from this instance?`)) return
	await instancesContentRemove(id.value, path)
	store.content = await instancesContentList(id.value)
}
async function loadPendingManual() {
	try {
		pendingManual.value = await instancesManualPending(id.value)
	} catch {
		pendingManual.value = []
	}
}
async function install() {
	try {
		await store.install(id.value)
		toasts.push('Installation started', 'Progress is available in Jobs.', 'success')
		// The install job runs in the background; give it a moment before
		// checking what, if anything, still needs a human.
		setTimeout(() => void loadPendingManual(), 3000)
	} catch (error) {
		toasts.push('Install failed', String(error), 'danger')
	}
}
async function provideManual(item: PendingManualDownload) {
	providing.value = item.target
	try {
		const provided = await instancesManualProvide(id.value, item.target)
		if (provided) {
			toasts.push('File placed', item.name, 'success')
			await Promise.all([loadPendingManual(), instancesContentList(id.value).then((c) => (store.content = c))])
		}
	} catch (error) {
		toasts.push(`Could not place ${item.name}`, String(error), 'danger')
	} finally {
		providing.value = ''
	}
}
async function exportPack(format: 'modrinth' | 'curse_forge') {
	try {
		const result = await instancesExport(id.value, format)
		toasts.push(
			'Export complete',
			`${result.path}${result.excludedHandAdded ? ` — ${result.excludedHandAdded} hand-added files excluded` : ''}`,
			'success',
		)
	} catch (error) {
		toasts.push('Export failed', String(error), 'danger')
	}
}
async function deleteInstance() {
	if (!confirm('Remove this instance from Packwand? Its game files and saves will be kept.')) return
	await store.remove(id.value, false)
	await router.push('/instances')
}
async function deleteInstanceFiles() {
	if (
		!confirm(
			'Permanently delete this instance, including saves, configs, screenshots, and logs? This cannot be undone.',
		)
	)
		return
	if (!confirm(`Final confirmation: permanently delete all files for ${store.current?.name}?`))
		return
	await store.remove(id.value, true)
	await router.push('/instances')
}
async function loadLogs() {
	const jobId = store.statuses[id.value]?.jobId
	if (jobId) {
		try {
			logs.value = Array.from((await jobsGet(jobId)).logs)
		} catch {
			logs.value = []
		}
	}
}
async function loadFiles() {
	files.value = await instancesFilesList(id.value)
}
async function openFile(path: string) {
	selectedFile.value = path
	try {
		fileContent.value = await instancesFileRead(id.value, path)
	} catch (error) {
		fileContent.value = ''
		toasts.push('Could not open file', String(error), 'danger')
	}
}
async function saveFile() {
	try {
		await instancesFileWrite(id.value, selectedFile.value, fileContent.value)
		toasts.push('File saved', selectedFile.value, 'success')
	} catch (error) {
		toasts.push('Could not save file', String(error), 'danger')
	}
}

function bytesToDataUrl(bytes: number[] | null): string {
	if (!bytes?.length) return ''
	let binary = ''
	for (const byte of bytes) binary += String.fromCharCode(byte)
	return `data:image/png;base64,${btoa(binary)}`
}
async function loadArtwork() {
	const [icon, background] = await Promise.all([
		instancesImage(id.value, 'icon').catch(() => null),
		instancesImage(id.value, 'background').catch(() => null),
	])
	iconUrl.value = bytesToDataUrl(icon)
	backgroundUrl.value = bytesToDataUrl(background)
}

watch(active, (value) => {
	if (value === 'logs') void loadLogs()
	if (value === 'files') void loadFiles()
})
onMounted(async () => {
	await Promise.all([
		store.load(id.value),
		defaults.value ? Promise.resolve() : defaults.load(),
		store.hydrate(),
		loadArtwork(),
		loadPendingManual(),
	])
	populate()
})
</script>

<template>
	<section v-if="store.current" class="grid view-grid">
		<div class="panel span-12 instance-detail-panel">
			<div
				class="instance-detail-hero"
				:style="backgroundUrl ? { backgroundImage: `url(${backgroundUrl})` } : undefined"
			>
				<div class="instance-detail-hero__shade" />
				<div class="instance-detail-hero__identity">
					<img v-if="iconUrl" :src="iconUrl" class="instance-detail-hero__icon" alt="" />
					<div v-else class="instance-detail-hero__icon instance-detail-hero__placeholder">PW</div>
					<div>
						<h2>{{ store.current.name }}</h2>
						<p class="panel-copy">
							{{ store.current.loader }} {{ store.current.gameVersion }} ·
							{{
								store.current.source.kind === 'linked' ? store.current.source.packDir : 'Standalone'
							}}
						</p>
					</div>
				</div>
				<div class="panel-actions instance-detail-hero__actions">
					<Button variant="secondary" @click="install">Reinstall</Button
					><Button v-if="launch.playing || launch.starting" variant="danger" @click="launch.stop"
						>Stop</Button
					><Button v-else @click="launch.play">Play</Button>
				</div>
			</div>
			<Tabs v-model="active" :items="tabs" />

			<div v-if="pendingManual.length" class="instance-manual-banner">
				<p>
					<strong>{{ pendingManual.length }}</strong> mod{{ pendingManual.length === 1 ? '' : 's' }}
					{{ pendingManual.length === 1 ? "isn't" : "aren't" }} on CurseForge's API for third-party
					downloads. Grab {{ pendingManual.length === 1 ? 'it' : 'them' }} from the page and point
					Packwand at the file — same as Prism.
				</p>
				<article v-for="item in pendingManual" :key="item.target" class="instance-content-row">
					<div>
						<strong>{{ item.name }}</strong><small>{{ item.target }}</small>
					</div>
					<div class="panel-actions">
						<Button v-if="item.pageUrl" variant="secondary" @click="providerOpenPage(item.pageUrl!)"
							>Open page</Button
						><Button :busy="providing === item.target" @click="provideManual(item)"
							>Choose file…</Button
						>
					</div>
				</article>
			</div>

			<div v-if="active === 'content'" class="instance-detail-list">
				<EmptyState
					v-if="!store.content.length"
					title="No installed content"
					message="Install this instance to materialize its pack content."
				/>
				<article v-for="item in store.content" :key="item.path" class="instance-content-row">
					<div>
						<strong>{{ item.name }}</strong
						><small
							>{{ item.packSourced ? 'From backing pack' : 'Hand-added' }} · {{ item.path }}</small
						>
					</div>
					<div class="panel-actions">
						<Button variant="secondary" @click="toggle(item.path)">{{
							item.enabled ? 'Disable' : 'Enable'
						}}</Button
						><Button variant="danger" @click="remove(item.path)">Remove</Button>
					</div>
				</article>
			</div>

			<form
				v-else-if="active === 'settings'"
				class="form-grid instance-settings"
				@submit.prevent="saveSettings"
			>
				<label
					><span>Java path</span
					><input
						v-model="settings.javaPath"
						:placeholder="
							defaults.value?.javaDefaults[store.current.gameVersion] || 'Automatically discovered'
						"
				/></label>
				<label
					><span>Minimum memory (MiB)</span
					><input v-model="settings.memoryMinMb" type="number" min="256" placeholder="JVM default"
				/></label>
				<label
					><span>Maximum memory (MiB)</span
					><input
						v-model="settings.memoryMaxMb"
						type="number"
						min="256"
						:placeholder="inheritedPlaceholder('', String(defaults.value?.memoryMb ?? 4096))"
				/></label>
				<label
					><span>Window width</span
					><input v-model="settings.windowWidth" type="number" min="320" placeholder="Game default"
				/></label>
				<label
					><span>Window height</span
					><input
						v-model="settings.windowHeight"
						type="number"
						min="240"
						placeholder="Game default"
				/></label>
				<label
					><span><input v-model="settings.fullscreen" type="checkbox" /> Fullscreen</span></label
				>
				<label
					><span>Extra JVM arguments (one per line)</span
					><textarea v-model="settings.extraJvmArgs" rows="4" />
				</label>
				<label
					><span>Extra game arguments (one per line)</span
					><textarea v-model="settings.extraGameArgs" rows="4" />
				</label>
				<label
					><span>Environment (KEY=value)</span><textarea v-model="settings.env" rows="4" />
				</label>
				<div class="form-actions"><Button type="submit" :busy="busy">Save settings</Button></div>
			</form>

			<div v-else-if="active === 'logs'">
				<LogViewer :lines="logs" /><Button variant="secondary" @click="loadLogs"
					>Refresh logs</Button
				>
			</div>
			<div v-else class="instance-files">
				<div class="instance-file-tree">
					<button
						v-for="file in files"
						:key="file.path"
						:disabled="file.directory"
						@click="openFile(file.path)"
					>
						{{ file.directory ? '▸' : '·' }} {{ file.path }}
					</button>
				</div>
				<div class="instance-file-editor">
					<EmptyState
						v-if="!selectedFile"
						title="Select a text file"
						message="Game files are confined to this instance; instance metadata and its backing pack are protected."
					/><template v-else
						><strong>{{ selectedFile }}</strong
						><textarea v-model="fileContent" rows="20" spellcheck="false" /><Button
							@click="saveFile"
							>Save file</Button
						></template
					>
				</div>
			</div>

			<footer class="instance-detail-footer">
				<Button variant="secondary" @click="exportPack('modrinth')">Export .mrpack</Button
				><Button variant="secondary" @click="exportPack('curse_forge')">Export CurseForge</Button
				><Button variant="quiet" @click="deleteInstance">Remove, keep files</Button
				><Button variant="danger" @click="deleteInstanceFiles">Delete files…</Button>
			</footer>
		</div>
	</section>
</template>

<style scoped>
.instance-detail-list {
	display: grid;
	gap: 8px;
	padding: 16px 0;
}
.instance-detail-panel {
	overflow: hidden;
}
.instance-detail-hero {
	position: relative;
	display: flex;
	min-height: 190px;
	align-items: flex-end;
	justify-content: space-between;
	gap: 20px;
	margin: -18px -18px 0;
	padding: 28px 28px 22px;
	overflow: hidden;
	background:
		radial-gradient(
			circle at 75% 20%,
			color-mix(in srgb, var(--accent) 38%, transparent),
			transparent 40%
		),
		linear-gradient(145deg, var(--surface-2), var(--surface-3));
	background-position: center;
	background-size: cover;
}
.instance-detail-hero__shade {
	position: absolute;
	inset: 0;
	background:
		linear-gradient(90deg, rgb(7 10 14 / 88%), rgb(7 10 14 / 42%) 62%, rgb(7 10 14 / 65%)),
		linear-gradient(0deg, rgb(7 10 14 / 78%), transparent 70%);
}
.instance-detail-hero__identity {
	position: relative;
	z-index: 1;
	display: flex;
	align-items: center;
	min-width: 0;
	gap: 17px;
}
.instance-detail-hero__icon {
	width: 88px;
	height: 88px;
	flex: none;
	border: 3px solid rgb(255 255 255 / 24%);
	border-radius: 16px;
	background: var(--surface-3);
	box-shadow: 0 12px 32px rgb(0 0 0 / 42%);
	object-fit: cover;
}
.instance-detail-hero__placeholder {
	display: grid;
	place-items: center;
	color: rgb(255 255 255 / 72%);
	font-size: 22px;
	font-weight: 750;
}
.instance-detail-hero h2 {
	margin: 2px 0 5px;
	color: #fff;
	font-size: 26px;
	text-shadow: 0 2px 8px #000;
}
.instance-detail-hero p {
	overflow: hidden;
	max-width: 720px;
	margin: 0;
	color: rgb(255 255 255 / 72%);
	font-size: 11.5px;
	text-overflow: ellipsis;
	white-space: nowrap;
}
.instance-detail-hero__actions {
	position: relative;
	z-index: 2;
	flex: none;
}
.instance-detail-panel > .tabs {
	margin-top: 14px;
}
.instance-content-row {
	display: flex;
	justify-content: space-between;
	gap: 16px;
	align-items: center;
	padding: 12px;
	border: 1px solid var(--border);
	border-radius: 8px;
}
.instance-content-row small {
	display: block;
	color: var(--muted);
	margin-top: 4px;
}
.instance-manual-banner {
	display: grid;
	gap: 8px;
	margin-top: 14px;
	padding: 12px;
	border: 1px solid color-mix(in srgb, var(--accent) 45%, var(--border));
	border-radius: 8px;
	background: color-mix(in srgb, var(--accent) 8%, transparent);
}
.instance-manual-banner > p {
	margin: 0 0 4px;
	color: var(--muted);
	font-size: 12.5px;
}
.instance-settings {
	padding-top: 16px;
}
.instance-detail-footer {
	display: flex;
	gap: 8px;
	justify-content: flex-end;
	margin-top: 18px;
	padding-top: 14px;
	border-top: 1px solid var(--border);
}
.instance-files {
	display: grid;
	grid-template-columns: minmax(220px, 32%) 1fr;
	gap: 12px;
	padding-top: 16px;
}
.instance-file-tree {
	display: grid;
	align-content: start;
	max-height: 520px;
	overflow: auto;
}
.instance-file-tree button {
	text-align: left;
	border: 0;
	background: transparent;
	color: inherit;
	padding: 5px 8px;
}
.instance-file-editor {
	display: grid;
	gap: 8px;
}
.instance-file-editor textarea {
	width: 100%;
	font-family: var(--font-mono);
}
@media (max-width: 760px) {
	.instance-detail-hero {
		align-items: flex-start;
		flex-direction: column;
	}
	.instance-detail-hero__icon {
		width: 70px;
		height: 70px;
	}
}
</style>
