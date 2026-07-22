<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { useInstanceLaunch } from '@/composables/instances/useInstanceLaunch'
import { packIcon } from '@/helpers/invoke/packs'
import type { InstanceSummary } from '@/helpers/types'

import AppIcon from './AppIcon.vue'
import Button from './Button.vue'
import ProgressBar from './ProgressBar.vue'

const props = defineProps<{ instance: InstanceSummary }>()
const { phase, message, playing, starting, modLoading, play, stop } = useInstanceLaunch(props.instance.id)

const iconUrl = ref<string | null>(null)

function bytesToDataUrl(bytes: number[]): string {
  let binary = ''
  for (const byte of bytes) binary += String.fromCharCode(byte)
  return `data:image/png;base64,${btoa(binary)}`
}

onMounted(async () => {
  try {
    const bytes = await packIcon(props.instance.id)
    if (bytes) iconUrl.value = bytesToDataUrl(bytes)
  } catch {
    // No icon on disk for this pack; the card falls back to the placeholder mark.
  }
})

function subtitle(): string {
  const loader = props.instance.loaders[0]
  const loaderLabel = loader ? loader.charAt(0).toUpperCase() + loader.slice(1) : 'Vanilla'
  return `${loaderLabel} ${props.instance.minecraftVersion ?? '(version unset)'}`
}

function statusText(): string {
  if (message.value) return message.value
  if (playing.value) return 'Running'
  if (phase.value === 'stopped') return 'Stopped'
  if (phase.value === 'error') return 'Failed'
  if (starting.value) return 'Starting'
  return 'Not launched yet'
}
</script>

<template>
  <article class="instance-row">
    <img v-if="iconUrl" :src="iconUrl" class="instance-icon" alt="" />
    <div v-else class="instance-icon instance-icon--placeholder"><AppIcon name="instances" /></div>
    <div class="instance-meta">
      <strong>{{ instance.name }}</strong>
      <span><AppIcon name="mods" :size="12" /> {{ subtitle() }}</span>
    </div>
    <div class="instance-status-block">
      <div class="instance-status" :class="'instance-status--' + phase">
        <i class="instance-status-dot" /><span>{{ statusText() }}</span>
      </div>
      <ProgressBar v-if="starting" :value="0" indeterminate label="Starting" />
    </div>
    <Button v-if="playing || starting" variant="danger" @click="stop">Stop</Button>
    <Button v-else :busy="modLoading" @click="play">Play</Button>
  </article>
</template>
