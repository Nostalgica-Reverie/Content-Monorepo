<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from 'vue'

import { core } from '@/core/packwand'
import {
	monaco,
	installMonacoTheme,
	languageForPath,
	registerPackwandLanguages,
} from '@/editor/monaco'
import { normalizeBridgeError } from '@/helpers/errors'
import {
	onCollabDocument,
	onCollabPresence,
	onPacksChanged,
	onWorkspaceFilesChanged,
} from '@/helpers/events'
import { transformMany } from '@/helpers/collab-ot'
import {
	collabDocumentClose,
	collabDocumentEdit,
	collabDocumentOpen,
	collabDocumentSave,
	collabDocumentSnapshot,
	collabPresence,
	type DocumentUpdate,
	type PresenceUpdate,
	type TextOp,
} from '@/helpers/invoke/collab'
import {
	editorDocumentRead,
	editorDocumentWrite,
	editorFsReadFile,
	editorSearch,
	type SearchMatch,
} from '@/helpers/invoke/editor'
import type { GitDiffDocument } from '@/helpers/invoke/git'
import {
	extensionLanguageSnapshot,
	type EditorLanguageSnapshot,
	type EditorSymbol,
} from '@/helpers/invoke/language'
import { useAppCoreStore } from '@/stores/appCore'
import { useCollabStore } from '@/stores/collab'
import { useExtensionsStore } from '@/stores/extensions'
import { useThemeStore } from '@/stores/theme'

const props = defineProps<{
	packId: string
	packRoot: string
	reload: number
	openPath?: string
	diffRequest?: GitDiffDocument | null
}>()
const appCore = useAppCoreStore()
const extensions = useExtensionsStore()
const themes = useThemeStore()
const collab = useCollabStore()
const host = ref<HTMLElement | null>(null)
const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null)
const diffHost = ref<HTMLElement | null>(null)
const diffEditor = shallowRef<monaco.editor.IStandaloneDiffEditor | null>(null)
const diffOpen = ref(false)
const diffTitle = ref('')
let diffModels:
	{ original: monaco.editor.ITextModel; modified: monaco.editor.ITextModel } | undefined
const tabs = ref<string[]>([])
const activePath = ref('')
const renderVersion = ref(0)
const busy = ref(false)
const error = ref('')
const searchQuery = ref('')
const searchResults = ref<SearchMatch[]>([])
const searchOpen = ref(false)
const searchCaseSensitive = ref(false)
const searchRegex = ref(false)
const snapshot = shallowRef<EditorLanguageSnapshot | null>(null)

type PreviewKind = 'text' | 'image' | 'audio' | 'video' | 'binary'
interface OpenDocument {
	path: string
	kind: PreviewKind
	model?: monaco.editor.ITextModel
	savedVersion?: number
	hash?: string
	conflicted?: boolean
	viewState?: monaco.editor.ICodeEditorViewState | null
	objectUrl?: string
	bytes?: number
	revision?: number
	pending?: Array<{ ops: TextOp[]; sent: boolean }>
}

const documents = new Map<string, OpenDocument>()
const remoteDecorations = new Map<number, monaco.editor.IEditorDecorationsCollection>()
let applyingRemote = false
let presenceTimer: ReturnType<typeof setTimeout> | null = null
const active = computed(() => {
	renderVersion.value
	return documents.get(activePath.value) ?? null
})

function dirty(document: OpenDocument) {
	return (
		document.kind === 'text' && document.model?.getAlternativeVersionId() !== document.savedVersion
	)
}

function name(path: string) {
	return path.split('/').pop() || path
}

function previewKind(path: string): PreviewKind {
	const extension = path.split('.').pop()?.toLowerCase()
	if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp', 'ico'].includes(extension ?? ''))
		return 'image'
	if (['mp3', 'ogg', 'wav', 'flac', 'm4a'].includes(extension ?? '')) return 'audio'
	if (['mp4', 'webm', 'ogv'].includes(extension ?? '')) return 'video'
	return 'text'
}

function mime(path: string, kind: PreviewKind) {
	const extension = path.split('.').pop()?.toLowerCase()
	const exact: Record<string, string> = {
		svg: 'image/svg+xml',
		png: 'image/png',
		jpg: 'image/jpeg',
		jpeg: 'image/jpeg',
		gif: 'image/gif',
		webp: 'image/webp',
		mp3: 'audio/mpeg',
		ogg: kind === 'video' ? 'video/ogg' : 'audio/ogg',
		wav: 'audio/wav',
		flac: 'audio/flac',
		mp4: 'video/mp4',
		webm: 'video/webm',
	}
	return exact[extension ?? ''] ?? 'application/octet-stream'
}

function modelUri(path: string) {
	return monaco.Uri.from({ scheme: 'packwand', authority: props.packId, path: `/${path}` })
}

function collabState(document: OpenDocument) {
	document.revision ??= 0
	document.pending ??= []
	return document as OpenDocument & {
		revision: number
		pending: Array<{ ops: TextOp[]; sent: boolean }>
	}
}

function changesToOps(event: monaco.editor.IModelContentChangedEvent): TextOp[] {
	return [...event.changes]
		.sort((left, right) => right.rangeOffset - left.rangeOffset)
		.flatMap((change) => {
			const operations: TextOp[] = []
			if (change.rangeLength) {
				operations.push({ kind: 'delete', offset: change.rangeOffset, length: change.rangeLength })
			}
			if (change.text) {
				operations.push({ kind: 'insert', offset: change.rangeOffset, text: change.text })
			}
			return operations
		})
}

function registerDocumentModel(document: OpenDocument) {
	const model = document.model
	if (!model) return
	model.onDidChangeContent((event) => {
		renderVersion.value++
		appCore.dispatch(core.Message$DocumentChanged(document.path))
		if (applyingRemote || !collab.role || collab.connection !== 'connected') return
		const operations = changesToOps(event)
		if (!operations.length) return
		const state = collabState(document)
		state.pending.push({ ops: operations, sent: false })
		void sendNextBatch(document)
	})
}

async function sendNextBatch(document: OpenDocument) {
	const state = collabState(document)
	if (state.pending.some((batch) => batch.sent)) return
	const next = state.pending[0]
	if (!next || collab.connection !== 'connected') return
	next.sent = true
	try {
		await collabDocumentEdit(document.path, state.revision, next.ops)
	} catch (caught) {
		next.sent = false
		error.value = normalizeBridgeError(caught).message
	}
}

async function announceDocument(document: OpenDocument) {
	if (!document.model || collab.connection !== 'connected') return
	if (collab.role === 'host') {
		await collabDocumentSnapshot(document.path, document.model.getValue())
	} else if (collab.role === 'guest') {
		await collabDocumentOpen(document.path)
	}
}

async function createDocument(path: string, activate = true) {
	let document = documents.get(path)
	if (!document) {
		const kind = previewKind(path)
		if (kind === 'text') {
			try {
				const loaded = await editorDocumentRead(props.packId, path)
				const model = monaco.editor.createModel(
					loaded.content,
					languageForPath(path),
					modelUri(path),
				)
				document = {
					path,
					kind,
					model,
					hash: loaded.hash,
					savedVersion: model.getAlternativeVersionId(),
				}
				registerDocumentModel(document)
			} catch (caught) {
				const normalized = normalizeBridgeError(caught)
				if (normalized.kind !== 'binary_file') throw caught
				const bytes = await editorFsReadFile(props.packId, path)
				document = { path, kind: 'binary', bytes: bytes.length }
			}
		} else {
			const bytes = await editorFsReadFile(props.packId, path)
			const objectUrl = URL.createObjectURL(
				new Blob([Uint8Array.from(bytes)], { type: mime(path, kind) }),
			)
			document = { path, kind, objectUrl, bytes: bytes.length }
		}
		documents.set(path, document)
		tabs.value.push(path)
		applyMarkers(document)
		await announceDocument(document)
	}
	if (activate) switchTo(path)
	return document
}

function switchTo(path: string) {
	const previous = documents.get(activePath.value)
	if (previous?.model && editor.value) previous.viewState = editor.value.saveViewState()
	activePath.value = path
	const document = documents.get(path)
	if (document?.model && editor.value) {
		editor.value.setModel(document.model)
		if (document.viewState) editor.value.restoreViewState(document.viewState)
		editor.value.focus()
	} else {
		editor.value?.setModel(null)
	}
	renderVersion.value++
	schedulePresence()
}

async function openDocument(path: string) {
	if (!path) return
	error.value = ''
	busy.value = true
	try {
		const effects = appCore.dispatch(core.Message$RequestDocument(path))
		if (effects.some(core.Effect$isLoadDocument)) {
			await createDocument(path)
			appCore.dispatch(core.Message$DocumentOpened(path))
		} else {
			switchTo(path)
		}
	} catch (caught) {
		error.value = normalizeBridgeError(caught).message
	} finally {
		busy.value = false
	}
}

async function save(document = active.value) {
	if (!document?.model || !dirty(document)) return
	busy.value = true
	error.value = ''
	try {
		const saved = await editorDocumentWrite(
			props.packId,
			document.path,
			document.model.getValue(),
			document.hash ?? '',
		)
		document.hash = saved.hash
		document.conflicted = false
		document.savedVersion = document.model.getAlternativeVersionId()
		appCore.dispatch(core.Message$DocumentSaved(document.path))
		renderVersion.value++
		await refreshLanguageSnapshot()
		if (collab.role) await collabDocumentSave(document.path)
	} catch (caught) {
		const normalized = normalizeBridgeError(caught)
		if (normalized.kind === 'file_conflict') document.conflicted = true
		error.value = normalized.message
	} finally {
		busy.value = false
	}
}

function closeDiff() {
	diffOpen.value = false
	diffEditor.value?.setModel(null)
	diffModels?.original.dispose()
	diffModels?.modified.dispose()
	diffModels = undefined
}

async function showDiff(request?: GitDiffDocument | null) {
	if (!request) return
	closeDiff()
	diffOpen.value = true
	diffTitle.value = request.path
	await nextTick()
	if (!diffEditor.value) {
		diffEditor.value = monaco.editor.createDiffEditor(diffHost.value!, {
			automaticLayout: true,
			readOnly: true,
			originalEditable: false,
			minimap: { enabled: false },
			renderSideBySide: true,
		})
	}
	const language = languageForPath(request.path)
	const stamp = Date.now()
	diffModels = {
		original: monaco.editor.createModel(
			request.original,
			language,
			monaco.Uri.parse(`packwand-diff://original/${stamp}/${request.path}`),
		),
		modified: monaco.editor.createModel(
			request.modified,
			language,
			monaco.Uri.parse(`packwand-diff://modified/${stamp}/${request.path}`),
		),
	}
	diffEditor.value.setModel(diffModels)
}

async function handleExternalChanges(paths: string[]) {
	const normalizedChanges = new Set(paths.map((path) => path.replaceAll('\\', '/').toLowerCase()))
	const root = props.packRoot.replaceAll('\\', '/').replace(/\/$/, '').toLowerCase()
	for (const document of documents.values()) {
		const fullPath = `${root}/${document.path}`.toLowerCase()
		if (!normalizedChanges.has(fullPath) || !document.model) continue
		if (dirty(document)) {
			document.conflicted = true
			renderVersion.value++
			continue
		}
		try {
			const loaded = await editorDocumentRead(props.packId, document.path)
			document.model.setValue(loaded.content)
			document.hash = loaded.hash
			document.savedVersion = document.model.getAlternativeVersionId()
			document.conflicted = false
			renderVersion.value++
		} catch (caught) {
			error.value = normalizeBridgeError(caught).message
		}
	}
}

async function runSearch() {
	searchOpen.value = true
	error.value = ''
	if (!searchQuery.value) {
		searchResults.value = []
		return
	}
	busy.value = true
	try {
		searchResults.value = await editorSearch(
			props.packId,
			searchQuery.value,
			searchCaseSensitive.value,
			searchRegex.value,
		)
	} catch (caught) {
		error.value = normalizeBridgeError(caught).message
	} finally {
		busy.value = false
	}
}

async function openSearchResult(result: SearchMatch) {
	await openDocument(result.path)
	editor.value?.setPosition({ lineNumber: result.line, column: result.column })
	editor.value?.revealLineInCenter(result.line)
	editor.value?.focus()
}

function closeDocument(path: string) {
	const document = documents.get(path)
	if (!document) return
	if (dirty(document) && !window.confirm(`Discard unsaved changes to ${name(path)}?`)) return
	const index = tabs.value.indexOf(path)
	tabs.value = tabs.value.filter((candidate) => candidate !== path)
	document.model?.dispose()
	if (document.objectUrl) URL.revokeObjectURL(document.objectUrl)
	documents.delete(path)
	if (collab.role) void collabDocumentClose(path).catch(() => undefined)
	appCore.dispatch(core.Message$DocumentClosed(path))
	if (activePath.value === path) {
		const next = tabs.value[Math.min(index, tabs.value.length - 1)] ?? ''
		if (next) switchTo(next)
		else {
			activePath.value = ''
			editor.value?.setModel(null)
		}
	}
	renderVersion.value++
}

async function reloadActive() {
	const document = active.value
	if (!document) return
	if (
		dirty(document) &&
		!window.confirm(`Reload ${name(document.path)} and discard unsaved changes?`)
	)
		return
	const path = document.path
	const index = tabs.value.indexOf(path)
	document.model?.dispose()
	if (document.objectUrl) URL.revokeObjectURL(document.objectUrl)
	documents.delete(path)
	tabs.value.splice(index, 1)
	await openDocument(path)
}

function applyMarkers(document: OpenDocument) {
	if (!document.model) return
	const diagnostics =
		snapshot.value?.diagnostics.filter(
			(issue) => issue.path.replaceAll('\\', '/') === document.path,
		) ?? []
	monaco.editor.setModelMarkers(
		document.model,
		'packwand',
		diagnostics.map((issue) => ({
			severity:
				issue.severity === 'error' ? monaco.MarkerSeverity.Error : monaco.MarkerSeverity.Warning,
			message: issue.message,
			startLineNumber: Math.max(1, issue.startLine),
			startColumn: Math.max(1, issue.startColumn),
			endLineNumber: Math.max(1, issue.endLine),
			endColumn: Math.max(1, issue.endColumn),
		})),
	)
}

async function refreshLanguageSnapshot() {
	if (collab.isGuest) return
	try {
		snapshot.value = await extensionLanguageSnapshot(props.packId, extensions.installedIds)
		for (const document of documents.values()) applyMarkers(document)
	} catch (caught) {
		console.warn('Packwand language snapshot could not be refreshed.', caught)
	}
}

function symbolAt(
	model: monaco.editor.ITextModel,
	position: monaco.Position,
): EditorSymbol | undefined {
	const line = model.getLineContent(position.lineNumber)
	const left = line.slice(0, position.column - 1).match(/[a-z0-9_.-]*$/i)?.[0] ?? ''
	const right = line.slice(position.column - 1).match(/^[a-z0-9_:./-]*/i)?.[0] ?? ''
	const token = left + right
	return snapshot.value?.symbols.find((symbol) => symbol.id === token)
}

const languageDisposables: monaco.IDisposable[] = []
function registerLanguageProviders() {
	for (const language of ['json', 'toml', 'mcfunction', 'javascript', 'typescript']) {
		languageDisposables.push(
			monaco.languages.registerCompletionItemProvider(language, {
				triggerCharacters: [':'],
				provideCompletionItems(model, position) {
					const word = model.getWordUntilPosition(position)
					const range = new monaco.Range(
						position.lineNumber,
						word.startColumn,
						position.lineNumber,
						word.endColumn,
					)
					return {
						suggestions: (snapshot.value?.symbols ?? []).map((symbol) => ({
							label: symbol.id,
							detail: symbol.detail,
							insertText: symbol.id,
							range,
							kind: monaco.languages.CompletionItemKind.Reference,
						})),
					}
				},
			}),
		)
		languageDisposables.push(
			monaco.languages.registerHoverProvider(language, {
				provideHover(model, position) {
					const symbol = symbolAt(model, position)
					return symbol
						? {
								contents: [
									{ value: `**${symbol.id}**` },
									{ value: symbol.detail },
									{ value: symbol.path },
								],
							}
						: null
				},
			}),
		)
		languageDisposables.push(
			monaco.languages.registerDefinitionProvider(language, {
				async provideDefinition(model, position) {
					const symbol = symbolAt(model, position)
					if (!symbol?.path) return null
					const target = await createDocument(symbol.path, false)
					return target.model
						? { uri: target.model.uri, range: new monaco.Range(1, 1, 1, 1) }
						: null
				},
			}),
		)
	}
}

function disposeDocuments() {
	for (const document of documents.values()) {
		document.model?.dispose()
		if (document.objectUrl) URL.revokeObjectURL(document.objectUrl)
	}
	documents.clear()
	tabs.value = []
	activePath.value = ''
}

function applyOperations(model: monaco.editor.ITextModel, operations: TextOp[]) {
	applyingRemote = true
	try {
		for (const operation of operations) {
			const start = model.getPositionAt(operation.offset)
			const end =
				operation.kind === 'delete'
					? model.getPositionAt(operation.offset + operation.length)
					: start
			model.applyEdits([
				{
					range: new monaco.Range(start.lineNumber, start.column, end.lineNumber, end.column),
					text: operation.kind === 'insert' ? operation.text : '',
					forceMoveMarkers: true,
				},
			])
		}
	} finally {
		applyingRemote = false
	}
}

function createSnapshotDocument(path: string, text: string) {
	const model = monaco.editor.createModel(text, languageForPath(path), modelUri(path))
	const document: OpenDocument = {
		path,
		kind: 'text',
		model,
		savedVersion: model.getAlternativeVersionId(),
		revision: 0,
		pending: [],
	}
	registerDocumentModel(document)
	documents.set(path, document)
	tabs.value.push(path)
	applyMarkers(document)
	return document
}

async function handleCollabDocument(update: DocumentUpdate) {
	if (update.type === 'fsChanged') {
		await handleExternalChanges([`${props.packRoot}/${update.path}`])
		return
	}
	if (update.type === 'open' && collab.role === 'host') {
		const document = await createDocument(update.path, false)
		if (document.model) await collabDocumentSnapshot(update.path, document.model.getValue())
		return
	}
	if (update.type === 'snapshot') {
		let document = documents.get(update.path)
		if (!document) document = createSnapshotDocument(update.path, update.text)
		if (!document.model) return
		const state = collabState(document)
		if (!state.pending.length && document.model.getValue() !== update.text) {
			applyingRemote = true
			try {
				document.model.setValue(update.text)
			} finally {
				applyingRemote = false
			}
		}
		state.revision = update.revision
		if (!activePath.value) switchTo(update.path)
		return
	}
	if (update.type !== 'applied') return
	const document = documents.get(update.path)
	if (!document?.model) return
	const state = collabState(document)
	const localId = collab.role === 'host' ? 1 : 2
	if (update.origin === localId) {
		const acknowledged = state.pending.shift()
		if (!acknowledged?.sent) {
			error.value = 'Collaboration acknowledgement arrived out of order.'
		}
	} else {
		const pending = state.pending.flatMap((batch) => batch.ops)
		const remoteForLocal = transformMany(update.ops, pending, true)
		applyOperations(document.model, remoteForLocal)
		for (const batch of state.pending) {
			batch.ops = transformMany(batch.ops, update.ops, false)
		}
	}
	state.revision = update.revision
	void sendNextBatch(document)
}

async function handleCollabPresence(update: PresenceUpdate) {
	const participant = collab.participants.find((candidate) => candidate.id === update.origin)
	const existing = remoteDecorations.get(update.origin)
	if (collab.followTarget === update.origin && update.path && update.selections[0]) {
		if (activePath.value !== update.path) await openDocument(update.path)
		const followedModel = documents.get(update.path)?.model
		if (followedModel) {
			const selection = update.selections[0]
			const start = followedModel.getPositionAt(selection.start)
			const end = followedModel.getPositionAt(selection.end)
			editor.value?.revealRangeInCenterIfOutsideViewport(
				new monaco.Range(start.lineNumber, start.column, end.lineNumber, end.column),
			)
		}
	}
	if (!update.path || update.path !== activePath.value || !active.value?.model) {
		existing?.clear()
		return
	}
	const model = active.value.model
	const colour = Math.abs(update.origin) % 6
	const decorations = update.selections.map((selection) => {
		const start = model.getPositionAt(selection.start)
		const end = model.getPositionAt(selection.end)
		return {
			range: new monaco.Range(start.lineNumber, start.column, end.lineNumber, end.column),
			options: {
				className: `collab-selection collab-selection-${colour}`,
				after:
					selection.start === selection.end
						? {
								content: participant?.displayName ?? 'Collaborator',
								inlineClassName: `collab-caret-label collab-caret-${colour}`,
							}
						: undefined,
			},
		}
	})
	if (existing) existing.set(decorations)
	else remoteDecorations.set(update.origin, editor.value!.createDecorationsCollection(decorations))
}

function schedulePresence() {
	if (!collab.role || presenceTimer) return
	presenceTimer = setTimeout(() => {
		presenceTimer = null
		const model = editor.value?.getModel()
		const selections =
			model && activePath.value
				? (editor.value?.getSelections() ?? []).map((selection) => ({
						start: model.getOffsetAt(selection.getStartPosition()),
						end: model.getOffsetAt(selection.getEndPosition()),
					}))
				: []
		void collabPresence(activePath.value || null, selections).catch(() => undefined)
	}, 50)
}

function beforeUnload(event: BeforeUnloadEvent) {
	if ([...documents.values()].some(dirty)) event.preventDefault()
}

let stopPacksChanged: (() => void) | undefined
let stopWorkspaceFilesChanged: (() => void) | undefined
let stopCollabDocument: (() => void) | undefined
let stopCollabPresence: (() => void) | undefined
function themeChanged(event: Event) {
	installMonacoTheme((event as CustomEvent).detail)
}

onMounted(async () => {
	registerPackwandLanguages()
	installMonacoTheme(themes.resolved)
	editor.value = monaco.editor.create(host.value!, {
		automaticLayout: true,
		model: null,
		fontFamily: 'JetBrains Mono',
		fontSize: 13,
		minimap: { enabled: false },
		scrollBeyondLastLine: false,
		renderWhitespace: 'selection',
		bracketPairColorization: { enabled: true },
		guides: { bracketPairs: true },
	})
	editor.value.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => void save())
	editor.value.onDidChangeCursorSelection(schedulePresence)
	editor.value.onDidScrollChange(schedulePresence)
	registerLanguageProviders()
	window.addEventListener('beforeunload', beforeUnload)
	window.addEventListener('packwand:theme-changed', themeChanged)
	stopPacksChanged = await onPacksChanged(() => void refreshLanguageSnapshot())
	stopWorkspaceFilesChanged = await onWorkspaceFilesChanged(
		(paths) => void handleExternalChanges(paths),
	)
	stopCollabDocument = await onCollabDocument((update) => void handleCollabDocument(update))
	stopCollabPresence = await onCollabPresence((update) => void handleCollabPresence(update))
	await refreshLanguageSnapshot()
	if (props.openPath) await openDocument(props.openPath)
})

watch(
	() => props.openPath,
	(path) => {
		if (path) void openDocument(path)
	},
)
watch(
	() => props.reload,
	() => void reloadActive(),
)
watch(
	() => props.diffRequest,
	(request) => void showDiff(request),
	{ immediate: true },
)
watch(
	() => props.packId,
	async () => {
		disposeDocuments()
		await nextTick()
		await refreshLanguageSnapshot()
		if (props.openPath) await openDocument(props.openPath)
	},
)
watch(
	() => extensions.installedIds.join(','),
	() => void refreshLanguageSnapshot(),
)
watch(
	() => collab.connection,
	(connection) => {
		if (connection !== 'connected') return
		for (const document of documents.values()) {
			void announceDocument(document).catch(() => undefined)
			void sendNextBatch(document)
		}
		schedulePresence()
	},
)

onBeforeUnmount(() => {
	stopPacksChanged?.()
	stopWorkspaceFilesChanged?.()
	stopCollabDocument?.()
	stopCollabPresence?.()
	if (presenceTimer) clearTimeout(presenceTimer)
	for (const collection of remoteDecorations.values()) collection.clear()
	remoteDecorations.clear()
	window.removeEventListener('beforeunload', beforeUnload)
	window.removeEventListener('packwand:theme-changed', themeChanged)
	languageDisposables.splice(0).forEach((disposable) => disposable.dispose())
	disposeDocuments()
	editor.value?.dispose()
	closeDiff()
	diffEditor.value?.dispose()
})
</script>

<template>
	<div class="packwand-workbench-host">
		<form class="editor-search" @submit.prevent="runSearch">
			<input v-model="searchQuery" type="search" placeholder="Search this pack?" />
			<label title="Match case"><input v-model="searchCaseSensitive" type="checkbox" /> Aa</label>
			<label title="Regular expression"><input v-model="searchRegex" type="checkbox" /> .*</label>
			<button type="submit">Search</button>
			<button v-if="searchOpen" type="button" @click="searchOpen = false">Close</button>
		</form>
		<Transition name="slide-fade">
			<div v-if="searchOpen" class="editor-search-results">
				<button
					v-for="result in searchResults"
					:key="`${result.path}:${result.line}:${result.column}`"
					type="button"
					@click="openSearchResult(result)"
				>
					<strong>{{ result.path }}:{{ result.line }}</strong
					><span>{{ result.preview }}</span>
				</button>
				<p v-if="searchQuery && !searchResults.length && !busy">No matches.</p>
			</div>
		</Transition>
		<TransitionGroup v-if="tabs.length" tag="div" name="tab" class="document-tabs" role="tablist">
			<button
				v-for="path in tabs"
				:key="path"
				class="document-tab"
				:class="{ active: path === activePath }"
				type="button"
				@click="switchTo(path)"
			>
				<span>{{ name(path) }}</span>
				<span v-if="documents.get(path) && dirty(documents.get(path)!)" class="document-tab__dirty"
					>*</span
				>
				<span class="document-tab__close" title="Close" @click.stop="closeDocument(path)"
					>&times;</span
				>
			</button>
		</TransitionGroup>
		<Transition name="fade">
			<div v-if="error" class="packwand-editor-error">{{ error }}</div>
		</Transition>
		<Transition name="slide-fade">
			<div v-if="active?.conflicted" class="packwand-editor-conflict">
				<span>{{ name(active.path) }} changed on disk while you were editing it.</span>
				<button type="button" @click="reloadActive">Reload from disk</button>
				<button type="button" @click="active.conflicted = false">Keep my version</button>
			</div>
		</Transition>
		<div v-if="diffOpen" class="packwand-diff-heading">
			<strong>{{ diffTitle }}</strong
			><button type="button" @click="closeDiff">Close diff</button>
		</div>
		<div v-show="diffOpen" ref="diffHost" class="packwand-monaco packwand-diff" />
		<div
			v-show="!diffOpen && active?.kind === 'text' && !error"
			ref="host"
			class="packwand-monaco"
		/>
		<div v-if="active?.kind === 'image'" class="packwand-media-preview">
			<img :src="active.objectUrl" :alt="name(active.path)" />
		</div>
		<div v-else-if="active?.kind === 'audio'" class="packwand-media-preview">
			<audio :src="active.objectUrl" controls />
		</div>
		<div v-else-if="active?.kind === 'video'" class="packwand-media-preview">
			<video :src="active.objectUrl" controls />
		</div>
		<div v-else-if="active?.kind === 'binary'" class="packwand-editor-empty">
			Binary file - {{ active.bytes }} bytes
		</div>
		<div v-else-if="!active && !error" class="packwand-editor-empty">
			Choose a file from the explorer.
		</div>
		<div v-if="busy" class="packwand-workbench-loading">Working...</div>
	</div>
</template>

<style>
.collab-selection {
	border-radius: 2px;
}
.collab-selection-0 {
	background: color-mix(in srgb, var(--accent) 28%, transparent);
}
.collab-selection-1 {
	background: color-mix(in srgb, var(--success) 28%, transparent);
}
.collab-selection-2 {
	background: color-mix(in srgb, var(--warning) 28%, transparent);
}
.collab-selection-3 {
	background: color-mix(in srgb, var(--danger) 28%, transparent);
}
.collab-selection-4 {
	background: color-mix(in srgb, var(--text) 20%, transparent);
}
.collab-selection-5 {
	background: color-mix(in srgb, var(--muted) 32%, transparent);
}
.collab-caret-label {
	position: relative;
	z-index: 4;
	margin-left: 1px;
	border-radius: 2px;
	padding: 1px 3px;
	background: var(--accent);
	color: var(--surface-1);
	font: 9px/1.2 var(--font-family);
	white-space: nowrap;
}
.collab-caret-1 {
	background: var(--success);
}
.collab-caret-2 {
	background: var(--warning);
}
.collab-caret-3 {
	background: var(--danger);
}
.collab-caret-4,
.collab-caret-5 {
	background: var(--muted);
}
</style>
