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

const props = defineProps<{ packId: string; reload: number }>()
const frame = ref<HTMLIFrameElement | null>(null)
const loaded = ref(false)
const source = computed(() => `/packwand-ide/index.html?pack=${encodeURIComponent(props.packId)}&reload=${props.reload}`)

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

async function onMessage(event: MessageEvent<BridgeRequest>) {
  const request = event.data
  const expectedOrigin = window.location.origin
  if (event.source !== frame.value?.contentWindow || (expectedOrigin !== 'null' && event.origin !== expectedOrigin) || request?.channel !== 'packwand:ide-fs' || request.direction !== 'request') return
  const target = event.source as Window
  const targetOrigin = expectedOrigin === 'null' ? '*' : expectedOrigin
  try {
    const result = await dispatch(request)
    target.postMessage({ channel: request.channel, direction: 'response', id: request.id, result }, targetOrigin)
  } catch (error) {
    target.postMessage({ channel: request.channel, direction: 'response', id: request.id, error: normalizeBridgeError(error) }, targetOrigin)
  }
}

watch(source, () => { loaded.value = false })
onMounted(() => window.addEventListener('message', onMessage))
onBeforeUnmount(() => window.removeEventListener('message', onMessage))
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
      @load="loaded = true"
    />
  </div>
</template>
