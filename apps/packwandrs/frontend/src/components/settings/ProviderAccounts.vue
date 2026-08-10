<script setup lang="ts">
// Connecting Modrinth and CurseForge, and the publishing credentials.
//
// Separate from the "Account" card, which is Minecraft/MSA launcher identity.
// They are independent: you can be connected to Modrinth and still launch the
// game offline.
//
// No secret is ever read back out of the backend — the store knows only
// whether a link exists and who it belongs to — so the inputs are always empty
// on load, and a stored key is reported rather than displayed.
import { onMounted, ref } from 'vue'

import Button from '@/components/ui/Button.vue'
import { normalizeBridgeError } from '@/helpers/errors'
import { useAccountsStore } from '@/stores/accounts'
import { useToastsStore } from '@/stores/toasts'

const accounts = useAccountsStore()
const toasts = useToastsStore()

const modrinthToken = ref('')
const curseforgeKey = ref('')
const uploadToken = ref('')
const busy = ref('')

async function run(key: string, message: string, work: () => Promise<unknown>) {
	busy.value = key
	try {
		await work()
		toasts.push(message, '', 'success')
	} catch (caught) {
		toasts.push('Could not update account', normalizeBridgeError(caught).message, 'danger')
	} finally {
		busy.value = ''
	}
}

onMounted(() => void accounts.load())
</script>

<template>
	<article class="panel span-6">
		<div class="panel-head">
			<h2>Publishing accounts</h2>
			<span class="status-badge">{{ accounts.canPublish ? 'ready' : 'incomplete' }}</span>
		</div>
		<p class="panel-copy">Needed only to publish. Browsing and pack editing work without them.</p>

		<div class="account-row">
			<div class="account-row__name">
				<strong>Modrinth</strong>
				<small v-if="accounts.modrinth?.linked">
					Connected as {{ accounts.modrinth.identity ?? 'unknown user' }}
				</small>
				<small v-else>Not connected</small>
			</div>
			<Button
				v-if="accounts.modrinth?.linked"
				variant="quiet"
				@click="run('unlink-mr', 'Disconnected Modrinth', () => accounts.unlink('modrinth'))"
			>
				Disconnect
			</Button>
		</div>
		<label class="setup-field">
			<span>Personal access token</span>
			<input v-model="modrinthToken" type="password" autocomplete="off" placeholder="mrp_…" />
		</label>
		<Button
			variant="secondary"
			:busy="busy === 'modrinth'"
			:disabled="!modrinthToken.trim()"
			@click="
				run('modrinth', 'Connected Modrinth', async () => {
					await accounts.linkModrinth(modrinthToken)
					modrinthToken = ''
				})
			"
		>
			Connect Modrinth
		</Button>

		<div class="account-row">
			<div class="account-row__name">
				<strong>CurseForge</strong>
				<!-- "Connected", not "Signed in". CurseForge has no third-party user
				     OAuth, so there is no account here to sign into — only a key. -->
				<small>{{ accounts.curseforge?.linked ? 'API key stored' : 'No API key' }}</small>
			</div>
			<Button
				v-if="accounts.curseforge?.linked"
				variant="quiet"
				@click="run('unlink-cf', 'Disconnected CurseForge', () => accounts.unlink('curse_forge'))"
			>
				Disconnect
			</Button>
		</div>
		<label class="setup-field">
			<span>API key — for browsing and resolving</span>
			<input v-model="curseforgeKey" type="password" autocomplete="off" placeholder="$2a$…" />
		</label>
		<Button
			variant="secondary"
			:busy="busy === 'curseforge'"
			:disabled="!curseforgeKey.trim()"
			@click="
				run('curseforge', 'Connected CurseForge', async () => {
					await accounts.linkCurseforge(curseforgeKey)
					curseforgeKey = ''
				})
			"
		>
			Connect CurseForge
		</Button>

		<label class="setup-field">
			<!-- A genuinely different credential from a different page. Conflating
			     the two produces a link that browses fine and fails at publish. -->
			<span>Upload token — for publishing, from the CurseForge author console</span>
			<input v-model="uploadToken" type="password" autocomplete="off" />
		</label>
		<Button
			variant="secondary"
			:busy="busy === 'upload'"
			:disabled="!uploadToken.trim()"
			@click="
				run('upload', 'Saved upload token', async () => {
					await accounts.setPublishToken(uploadToken)
					uploadToken = ''
				})
			"
		>
			Save upload token
		</Button>

		<div v-if="accounts.publishBlockers.length" class="notice account-notice">
			Publishing is limited: {{ accounts.publishBlockers.join('; ') }}.
		</div>
	</article>
</template>
