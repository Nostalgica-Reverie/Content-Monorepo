<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { normalizeBridgeError } from '@/helpers/errors'
import {
  editorFsCreateDir,
  editorFsDelete,
  editorFsReadDir,
  editorFsReadFile,
  editorFsRename,
  editorFsStat,
  editorFsWriteFile,
} from '@/helpers/invoke/editor'
import { extensionLanguageSnapshot } from '@/helpers/invoke/language'
import { onPacksChanged, onRawInputBatch } from '@/helpers/events'
import { useExtensionsStore } from '@/stores/extensions'

const props = defineProps<{ packId: string; reload: number; openPath?: string; theme: string }>()
const extensions = useExtensionsStore()
const frame = ref<HTMLIFrameElement | null>(null)
const loaded = ref(false)
let languageRefresh: ReturnType<typeof setTimeout> | undefined
let stopWatching: (() => void) | undefined
let stopRawInput: (() => void) | undefined
/**
 * The `open` parameter seeds `defaultLayout.editors` in the workbench
 * bootstrap, which is the only supported way to open a file in an embedded
 * Code-OSS without an extension to receive a command. It is therefore a
 * *load-time* input: changing it reloads the iframe.
 *
 * That reload is the deliberate trade. The alternative -- applying it only on
 * first mount -- makes every subsequent click in the sidebar silently do
 * nothing, and a control that appears dead is worse than one that is slow.
 */
const source = computed(() => {
  const parameters = new URLSearchParams({ pack: props.packId, reload: String(props.reload), theme: props.theme })
  if (props.openPath) parameters.set('open', props.openPath)
  return `/packwand-ide/index.html?${parameters.toString()}`
})

interface BridgeRequest {
  channel: 'packwand:ide-fs'
  direction: 'request'
  id: number
  method: string
  parameters?: Record<string, unknown>
}

function text(parameters: Record<string, unknown>, key: string): string {
  const value = parameters[key]
  if (typeof value !== 'string') throw new Error(`Invalid Packwand IDE ${key} parameter.`)
  return value
}

function flag(parameters: Record<string, unknown>, key: string): boolean {
  return parameters[key] === true
}

async function dispatch(request: BridgeRequest): Promise<unknown> {
  const parameters = request.parameters ?? {}
  switch (request.method) {
    case 'stat': return editorFsStat(props.packId, text(parameters, 'path'))
    case 'readDir': return editorFsReadDir(props.packId, text(parameters, 'path'))
    case 'readFile': return editorFsReadFile(props.packId, text(parameters, 'path'))
    case 'writeFile': {
      const content = parameters.content
      if (!Array.isArray(content) || content.some(value => !Number.isInteger(value) || value < 0 || value > 255)) {
        throw new Error('Invalid Packwand IDE file content.')
      }
      return editorFsWriteFile(props.packId, text(parameters, 'path'), content as number[], flag(parameters, 'create'), flag(parameters, 'overwrite'))
    }
    case 'createDir': return editorFsCreateDir(props.packId, text(parameters, 'path'))
    case 'delete': return editorFsDelete(props.packId, text(parameters, 'path'), flag(parameters, 'recursive'))
    case 'rename': return editorFsRename(props.packId, text(parameters, 'from'), text(parameters, 'to'), flag(parameters, 'overwrite'))
    default: throw new Error(`Unsupported Packwand IDE filesystem method: ${request.method}`)
  }
}

async function refreshLanguageSnapshot() {
  const target = frame.value?.contentWindow
  if (!target) return
  try {
    const snapshot = await extensionLanguageSnapshot(props.packId, extensions.installedIds)
    target.postMessage(
      { channel: 'packwand:ide-language', direction: 'update', snapshot },
      window.location.origin === 'null' ? '*' : window.location.origin,
    )
  } catch (error) {
    console.warn('Packwand language snapshot could not be refreshed.', error)
  }
}

function scheduleLanguageRefresh() {
  if (languageRefresh) clearTimeout(languageRefresh)
  languageRefresh = setTimeout(() => void refreshLanguageSnapshot(), 150)
}

function forwardRawInput(events: import('@/helpers/events').RawInputEvent[]) {
  const target = frame.value?.contentWindow
  if (!target || !events.length) return
  target.postMessage(
    { channel: 'packwand:ide-raw-input', direction: 'batch', events },
    window.location.origin === 'null' ? '*' : window.location.origin,
  )
}

function onFrameLoad() {
  loaded.value = true
  scheduleLanguageRefresh()
}

async function onMessage(event: MessageEvent<BridgeRequest>) {
  const request = event.data
  const expectedOrigin = window.location.origin
  if (event.source !== frame.value?.contentWindow || (expectedOrigin !== 'null' && event.origin !== expectedOrigin) || request?.channel !== 'packwand:ide-fs' || request.direction !== 'request') return
  const target = event.source as Window
  const targetOrigin = expectedOrigin === 'null' ? '*' : expectedOrigin
  try {
    const result = await dispatch(request)
    target.postMessage({ channel: request.channel, direction: 'response', id: request.id, result }, targetOrigin)
    if (['writeFile', 'createDir', 'delete', 'rename'].includes(request.method)) scheduleLanguageRefresh()
  } catch (error) {
    target.postMessage({ channel: request.channel, direction: 'response', id: request.id, error: normalizeBridgeError(error) }, targetOrigin)
  }
}

watch(source, () => { loaded.value = false })
watch(() => extensions.installedIds.join(','), scheduleLanguageRefresh)
onMounted(async () => {
  window.addEventListener('message', onMessage)
  ;[stopWatching, stopRawInput] = await Promise.all([
    onPacksChanged(scheduleLanguageRefresh),
    onRawInputBatch(forwardRawInput),
  ])
})
onBeforeUnmount(() => {
  window.removeEventListener('message', onMessage)
  if (languageRefresh) clearTimeout(languageRefresh)
  stopWatching?.()
  stopRawInput?.()
})
</script>

<template>
  <div class="packwand-workbench-host">
    <div v-if="!loaded" class="packwand-workbench-loading">Loading Packwand IDE…</div>
    <iframe
      ref="frame"
      :key="source"
      :src="source"
      title="Packwand IDE"
      class="packwand-workbench-frame"
      @load="onFrameLoad"
    />
  </div>
</template>
