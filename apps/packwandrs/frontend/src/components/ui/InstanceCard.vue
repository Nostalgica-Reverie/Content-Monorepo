<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'

import { useInstanceLaunch } from '@/composables/instances/useInstanceLaunch'
import { instanceVersionLabel } from '@/core/packwand'
import { instancesImage } from '@/helpers/invoke/instances'
import type { InstanceSummary } from '@/helpers/types'

import AppIcon from './AppIcon.vue'
import Button from './Button.vue'
import ProgressBar from './ProgressBar.vue'

const props = defineProps<{ instance: InstanceSummary }>()
const { phase, message, playing, starting, modLoading, play, stop } = useInstanceLaunch(
	props.instance.id,
)
const iconUrl = ref('')
const backgroundUrl = ref('')

const subtitle = computed(() =>
	instanceVersionLabel(props.instance.loader, props.instance.gameVersion),
)
const sourcePath = computed(() =>
	props.instance.source.kind === 'linked'
		? props.instance.source.packDir
		: 'Private standalone pack',
)
const sourceName = computed(() => {
	if (props.instance.source.kind === 'owned') return 'Standalone'
	const parts = props.instance.source.packDir.replaceAll('\\', '/').split('/').filter(Boolean)
	return parts.slice(-2).join(' / ') || 'Linked pack'
})
const playedLabel = computed(() =>
	props.instance.lastPlayedMs
		? new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' }).format(
				Math.round((props.instance.lastPlayedMs - Date.now()) / 86_400_000),
				'day',
			)
		: 'Never played',
)

function bytesToDataUrl(bytes: number[] | null): string {
	if (!bytes?.length) return ''
	let binary = ''
	for (const byte of bytes) binary += String.fromCharCode(byte)
	return `data:image/png;base64,${btoa(binary)}`
}

onMounted(async () => {
	const [icon, background] = await Promise.all([
		instancesImage(props.instance.id, 'icon').catch(() => null),
		instancesImage(props.instance.id, 'background').catch(() => null),
	])
	iconUrl.value = bytesToDataUrl(icon)
	backgroundUrl.value = bytesToDataUrl(background)
})

function statusText(): string {
	if (message.value) return message.value
	if (playing.value) return 'Running'
	if (phase.value === 'stopped') return 'Ready'
	if (phase.value === 'error') return 'Failed'
	if (starting.value) return 'Starting'
	if (props.instance.stage.state === 'installing') return 'Installing'
	if (props.instance.stage.state === 'failed') return props.instance.stage.message
	if (props.instance.stage.state === 'ready') return 'Ready'
	return 'Not installed'
}
</script>

<template>
	<article
		class="instance-card"
		:class="[`instance-card--${phase}`, { 'instance-card--art': backgroundUrl }]"
	>
		<div
			class="instance-card__art"
			:style="backgroundUrl ? { backgroundImage: `url(${backgroundUrl})` } : undefined"
		>
			<div class="instance-card__shade" />
			<div class="instance-card__badges">
				<span class="instance-card__badge"
					><AppIcon name="mods" :size="11" />{{ instance.loader }}</span
				>
				<span v-if="playing || starting" class="instance-card__badge instance-card__badge--live"
					><i />{{ playing ? 'Running' : 'Starting' }}</span
				>
			</div>
		</div>

		<div class="instance-card__identity">
			<img v-if="iconUrl" :src="iconUrl" class="instance-card__icon" alt="" />
			<div v-else class="instance-card__icon instance-card__icon--placeholder">
				<AppIcon name="instances" :size="28" />
			</div>
			<div class="instance-card__title">
				<RouterLink :to="`/instances/${instance.id}`">{{ instance.name }}</RouterLink>
				<span>{{ subtitle }}</span>
			</div>
		</div>

		<div class="instance-card__body">
			<div class="instance-card__source" :title="sourcePath">
				<AppIcon :name="instance.source.kind === 'linked' ? 'target' : 'folder'" :size="12" />{{
					sourceName
				}}
			</div>
			<div class="instance-status" :class="'instance-status--' + phase">
				<i class="instance-status-dot" /><span>{{ statusText() }}</span>
			</div>
			<ProgressBar v-if="starting" :value="0" indeterminate label="Starting" />
		</div>

		<footer class="instance-card__footer">
			<span>{{ playedLabel }}</span>
			<Button v-if="playing || starting" variant="danger" @click="stop">Stop</Button>
			<Button v-else :busy="modLoading" @click="play"
				><AppIcon name="play" :size="13" /> Play</Button
			>
		</footer>
	</article>
</template>
