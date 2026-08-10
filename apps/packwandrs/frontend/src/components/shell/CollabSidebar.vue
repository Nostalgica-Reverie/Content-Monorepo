<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import AppIcon from '@/components/ui/AppIcon.vue'
import { collabSetIdentity } from '@/helpers/invoke/collab'
import { useCollabStore } from '@/stores/collab'
import { useIdentityStore } from '@/stores/identity'
import { useSocialStore } from '@/stores/social'
import { useWorkbenchStore } from '@/stores/workbench'

const collab = useCollabStore()
const identity = useIdentityStore()
const social = useSocialStore()
const workbench = useWorkbenchStore()
const joinCode = ref('')
const displayName = ref('')
const atprotoIdentifier = ref('')
const busy = ref(false)
const error = ref('')

const localId = computed(() => (collab.role === 'host' ? 1 : collab.role === 'guest' ? 2 : null))

async function run(work: () => Promise<unknown>) {
	busy.value = true
	error.value = ''
	try {
		await work()
	} catch (caught) {
		error.value = String(caught)
	} finally {
		busy.value = false
	}
}

function host() {
	const packId = workbench.selectedPack?.id
	if (!packId) {
		error.value = 'Select a pack target before hosting.'
		return
	}
	void run(() => collab.host(packId))
}

function join() {
	if (!joinCode.value.trim()) return
	void run(() => collab.join(joinCode.value.trim()))
}

function saveIdentity() {
	if (!displayName.value.trim()) return
	void run(() => collabSetIdentity(displayName.value.trim()))
}

async function copyInvite() {
	if (!collab.invite) return
	try {
		await navigator.clipboard.writeText(collab.invite)
	} catch (caught) {
		error.value = `Could not copy invite: ${String(caught)}`
	}
}

function loginAtproto() {
	if (!atprotoIdentifier.value.trim()) return
	void run(async () => {
		await identity.login(atprotoIdentifier.value.trim())
		await social.refresh()
	})
}

function logoutAtproto() {
	void run(async () => {
		await identity.logout()
		social.clear()
	})
}

function sendToFriend(did: string) {
	if (!collab.invite) return
	void run(() => social.sendInvite(did, collab.invite!))
}

function joinPending(uri: string, invite: string) {
	void run(async () => {
		await collab.join(invite)
		social.dismissInvite(uri)
	})
}

onMounted(() => {
	void identity
		.load()
		.then((account) => (account ? social.refresh() : undefined))
		.catch((caught) => {
			error.value = String(caught)
		})
})
</script>

<template>
	<div class="collab-sidebar">
		<p class="collab-summary">
			<span class="connection-dot" :class="`connection-dot--${collab.connection}`" />
			{{ collab.role ? `${collab.role} · ${collab.connection}` : 'No live session' }}
		</p>

		<form v-if="!collab.role" class="collab-form" @submit.prevent="saveIdentity">
			<label>
				<span>Display name</span>
				<input v-model="displayName" autocomplete="name" placeholder="Account or git name" />
			</label>
			<button type="submit" :disabled="busy || !displayName.trim()">Save identity</button>
		</form>

		<div v-if="!collab.role" class="collab-actions">
			<button :disabled="busy || !workbench.selectedPack" @click="host">
				<AppIcon name="users" :size="14" /> Start session
			</button>
			<form class="collab-form" @submit.prevent="join">
				<label>
					<span>Invite code</span>
					<textarea
						v-model="joinCode"
						rows="3"
						spellcheck="false"
						placeholder="pw://host:port#key"
					/>
				</label>
				<button type="submit" :disabled="busy || !joinCode.trim()">Join session</button>
			</form>
		</div>

		<Transition name="slide-fade">
			<div v-if="collab.invite" class="collab-invite">
				<span>Invite</span>
				<code>{{ collab.invite }}</code>
				<button @click="copyInvite">Copy invite</button>
			</div>
		</Transition>

		<label v-if="collab.role === 'host'" class="collab-toggle">
			<input
				type="checkbox"
				:checked="collab.allowGitWrite"
				@change="collab.setAllowGitWrite(($event.target as HTMLInputElement).checked)"
			/>
			Allow guest stage and commit
		</label>

		<section class="social-friends">
			<div class="social-heading">
				<h3>Friends</h3>
				<button
					v-if="identity.signedIn"
					class="icon-btn"
					title="Refresh ATProto friends"
					:disabled="social.busy"
					@click="run(social.refresh)"
				>
					<AppIcon name="refresh" :size="13" />
				</button>
			</div>

			<form v-if="!identity.signedIn" class="collab-form" @submit.prevent="loginAtproto">
				<label>
					<span>ATProto handle or DID</span>
					<input v-model="atprotoIdentifier" autocomplete="username" placeholder="you.example" />
				</label>
				<button type="submit" :disabled="busy || !atprotoIdentifier.trim()">
					Sign in to discover friends
				</button>
			</form>

			<template v-else>
				<p class="social-account">
					<span>{{ identity.identity?.handle || identity.identity?.did }}</span>
					<button :disabled="busy" @click="logoutAtproto">Sign out</button>
				</p>
				<p v-if="social.busy && !social.loaded" class="side-empty">Loading friends…</p>
				<p v-else-if="social.loaded && !social.friends.length" class="side-empty">
					No mutual follows or Packwand contacts.
				</p>
				<div v-for="friend in social.friends" :key="friend.did" class="friend-row">
					<img v-if="friend.avatar" :src="friend.avatar" alt="" class="participant-avatar" />
					<span v-else class="participant-avatar">{{
						(friend.displayName || friend.handle || friend.did).slice(0, 1).toUpperCase()
					}}</span>
					<span class="participant-name" :title="friend.did">
						{{ friend.displayName || friend.handle || friend.did }}
					</span>
					<button
						v-if="collab.invite"
						class="participant-follow"
						:disabled="busy"
						@click="sendToFriend(friend.did)"
					>
						Send invite
					</button>
				</div>

				<div v-if="social.pendingInvites.length" class="pending-invites">
					<h3>
						Pending invites <span>{{ social.pendingInvites.length }}</span>
					</h3>
					<div v-for="pending in social.pendingInvites" :key="pending.uri" class="friend-row">
						<span class="participant-name">{{ pending.fromHandle || pending.from }}</span>
						<button
							:disabled="busy || Boolean(collab.role)"
							@click="joinPending(pending.uri, pending.invite)"
						>
							Join
						</button>
					</div>
				</div>
			</template>
		</section>

		<section v-if="collab.role" class="collab-participants">
			<h3>
				Participants <span>{{ collab.participants.length }}</span>
			</h3>
			<div v-for="participant in collab.participants" :key="participant.id" class="participant-row">
				<span class="participant-avatar">{{
					participant.displayName.slice(0, 1).toUpperCase()
				}}</span>
				<span class="participant-name">{{ participant.displayName }}</span>
				<span v-if="participant.id === localId" class="participant-you">you</span>
				<button
					v-else
					class="participant-follow"
					:class="{ active: collab.followTarget === participant.id }"
					@click="collab.follow(collab.followTarget === participant.id ? null : participant.id)"
				>
					{{ collab.followTarget === participant.id ? 'Following' : 'Follow' }}
				</button>
			</div>
		</section>

		<button v-if="collab.role" class="collab-leave" :disabled="busy" @click="run(collab.leave)">
			Leave session
		</button>
		<p v-if="error" class="side-error">{{ error }}</p>
	</div>
</template>

<style scoped>
.collab-sidebar,
.collab-actions,
.collab-form,
.collab-participants {
	display: grid;
	gap: 10px;
}
.social-friends,
.pending-invites {
	display: grid;
	gap: 8px;
}
.social-friends {
	padding-top: 8px;
	border-top: 1px solid var(--border);
}
.social-heading,
.social-account,
.friend-row {
	display: flex;
	align-items: center;
	gap: 8px;
}
.social-heading {
	justify-content: space-between;
}
.social-heading h3,
.pending-invites h3 {
	margin: 0;
	color: var(--muted);
	font-size: 11px;
	font-weight: 600;
	text-transform: uppercase;
}
.social-account {
	justify-content: space-between;
	margin: 0;
	color: var(--muted);
	font-size: 10.5px;
}
.social-account span {
	overflow: hidden;
	text-overflow: ellipsis;
}
.friend-row {
	min-width: 0;
}
.friend-row img.participant-avatar {
	object-fit: cover;
}
.collab-sidebar {
	padding: 8px;
}
.collab-summary,
.participant-row,
.collab-toggle {
	display: flex;
	align-items: center;
	gap: 8px;
}
.collab-summary {
	margin: 0;
	color: var(--muted);
	font-size: 11.5px;
	text-transform: capitalize;
}
.collab-form label {
	display: grid;
	gap: 5px;
	color: var(--muted);
	font-size: 11px;
}
.collab-form input,
.collab-form textarea {
	width: 100%;
	padding: 7px;
	font: 11.5px/1.4 var(--font-family);
	resize: vertical;
}
.collab-form button,
.collab-actions > button,
.collab-invite button {
	min-height: 29px;
	background: var(--accent);
	color: white;
}
.collab-actions > button {
	display: flex;
	align-items: center;
	justify-content: center;
	gap: 6px;
}
.collab-invite {
	display: grid;
	gap: 6px;
	padding: 8px;
	border: 1px solid var(--border);
	background: var(--surface-2);
}
.collab-invite > span,
.collab-participants h3 {
	color: var(--muted);
	font-size: 11px;
	font-weight: 600;
	text-transform: uppercase;
}
.collab-invite code {
	max-height: 72px;
	overflow: auto;
	font: 10.5px/1.4 var(--mono);
	word-break: break-all;
}
.collab-toggle {
	font-size: 11.5px;
}
.collab-participants h3 {
	display: flex;
	justify-content: space-between;
	margin: 4px 0 0;
}
.participant-row {
	min-width: 0;
	padding: 5px 2px;
}
.participant-avatar {
	display: grid;
	width: 24px;
	height: 24px;
	place-items: center;
	border-radius: 50%;
	background: var(--accent-muted);
	color: var(--accent);
	font-size: 10px;
	font-weight: 700;
}
.participant-name {
	min-width: 0;
	flex: 1;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}
.participant-you {
	color: var(--muted);
	font-size: 10px;
}
.participant-follow {
	padding: 3px 6px;
	font-size: 10.5px;
}
.participant-follow.active {
	color: var(--accent);
}
.collab-leave {
	min-height: 29px;
	color: var(--danger);
}
.side-error {
	margin: 0;
	color: var(--danger);
	font-size: 11px;
}
.connection-dot--connecting {
	background: var(--warning);
}
</style>
