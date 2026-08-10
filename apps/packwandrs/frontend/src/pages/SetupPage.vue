<script setup lang="ts">
// First run: workspace, then git, then provider credentials.
//
// Only the first step is required — `router.ts` gates every other route on a
// workspace and nothing else — so the remaining two are skippable and can be
// re-entered from Settings afterwards. The step model and the reachability
// rules live in `packwand/setup.gleam` and are tested there; this file is the
// form around them.
import { computed, ref, shallowRef } from 'vue'
import { useRouter } from 'vue-router'

import PackwandMascot from '@/components/PackwandMascot.vue'
import Button from '@/components/ui/Button.vue'
import { setupCore, setupStepKey, type SetupMessage, type SetupModel } from '@/core/packwand'
import { normalizeBridgeError } from '@/helpers/errors'
import { gitClone, gitInit, gitRemoteAdd, gitRepository } from '@/helpers/invoke/git'
import type { GitRepository } from '@/helpers/invoke/git'
import { markWorkspaceConfigured } from '@/router'
import { useAccountsStore } from '@/stores/accounts'
import { useWorkspaceStore } from '@/stores/workspace'

const workspace = useWorkspaceStore()
const accounts = useAccountsStore()
const router = useRouter()

const model = shallowRef<SetupModel>(setupCore.init())
const busy = ref('')
const error = ref('')
const notice = ref('')

const repository = ref<GitRepository | null>(null)
const cloneUrl = ref('')
const remoteUrl = ref('')
const modrinthToken = ref('')
const curseforgeKey = ref('')

// From Gleam, not `constructor.name` — see `setupStepKey`.
const stepName = computed(() => setupStepKey(model.value.step))

function dispatch(message: SetupMessage) {
	model.value = setupCore.update(model.value, message)
}

/** Runs an action with one shared busy/error surface. */
async function run(key: string, work: () => Promise<void>) {
	busy.value = key
	error.value = ''
	try {
		await work()
	} catch (caught) {
		error.value = normalizeBridgeError(caught).message
	} finally {
		busy.value = ''
	}
}

async function selectWorkspace() {
	await run('workspace', async () => {
		const path = await workspace.select()
		if (!path) return
		markWorkspaceConfigured(path)
		dispatch(new setupCore.WorkspaceChosen(path))
		// Detecting up front is what lets the git step offer "link" rather than
		// asking the user to tell us something we can already see.
		repository.value = await gitRepository()
	})
}

async function linkExisting() {
	await run('link', async () => {
		repository.value = await gitRepository()
		if (!repository.value.isRepo) {
			throw new Error('No git repository was found at the workspace path.')
		}
		notice.value = `Linked ${repository.value.root}`
		dispatch(new setupCore.RepositoryResolved(new setupCore.LinkExisting()))
	})
}

async function initRepository() {
	await run('init', async () => {
		repository.value = await gitInit()
		if (remoteUrl.value.trim()) await gitRemoteAdd('origin', remoteUrl.value.trim())
		notice.value = 'Initialised a new repository in the workspace.'
		dispatch(new setupCore.RepositoryResolved(new setupCore.InitNew()))
	})
}

async function cloneRepository() {
	await run('clone', async () => {
		const target = workspace.path
		if (!target) throw new Error('Choose a workspace first.')
		const cloned = await gitClone(cloneUrl.value, target)
		repository.value = await gitRepository()
		notice.value = `Cloned into ${cloned}`
		dispatch(new setupCore.RepositoryResolved(new setupCore.CloneRemote()))
	})
}

function skipRepository() {
	notice.value = ''
	dispatch(new setupCore.RepositorySkipped())
}

async function linkModrinth() {
	await run('modrinth', async () => {
		await accounts.linkModrinth(modrinthToken.value)
		modrinthToken.value = ''
		dispatch(new setupCore.ModrinthLinked())
	})
}

async function linkCurseforge() {
	await run('curseforge', async () => {
		await accounts.linkCurseforge(curseforgeKey.value)
		curseforgeKey.value = ''
		dispatch(new setupCore.CurseforgeLinked())
	})
}

async function finish() {
	dispatch(new setupCore.CredentialsFinished())
	await router.replace({ name: 'overview' })
}

function goTo(step: 'Repository' | 'Credentials') {
	dispatch(new setupCore.StepRequested(new setupCore[step]()))
}
</script>

<template>
	<main class="setup-page">
		<div class="setup-card">
			<PackwandMascot />

			<ol class="setup-steps" aria-label="Setup progress">
				<li :class="{ active: stepName === 'workspace', done: stepName !== 'workspace' }">
					Workspace
				</li>
				<li :class="{ active: stepName === 'repository' }">Repository</li>
				<li :class="{ active: stepName === 'credentials' }">Publishing</li>
			</ol>

			<Transition name="fade">
				<div v-if="error" class="error-banner">{{ error }}</div>
			</Transition>

			<Transition name="slide-fade" mode="out-in">
				<section v-if="stepName === 'workspace'" key="workspace" class="setup-step">
					<p class="eyebrow">Step 1 of 3</p>
					<h1>Choose your workspace</h1>
					<p>
						Pick the repository root containing <code>mods</code>, <code>modpacks</code>,
						<code>datapacks</code>, or <code>resourcepacks</code>. Packwand discovers manifest
						projects and their <code>pack.toml</code> targets locally.
					</p>
					<Button :busy="busy === 'workspace'" @click="selectWorkspace">Select workspace</Button>
					<small>Your path is stored in the local app configuration. No server is started.</small>
				</section>

				<section v-else-if="stepName === 'repository'" key="repository" class="setup-step">
					<p class="eyebrow">Step 2 of 3 · optional</p>
					<h1>Connect a Git repository</h1>
					<p v-if="repository?.isRepo">
						Found a repository at <code>{{ repository.root }}</code
						><template v-if="repository.branch">
							on <code>{{ repository.branch }}</code></template
						>.
					</p>
					<p v-else>
						There is no git repository in this workspace yet. Start one, clone an existing project,
						or skip — Packwand works fine without version control.
					</p>

					<div v-if="notice" class="setup-notice">{{ notice }}</div>

					<div class="setup-actions">
						<Button v-if="repository?.isRepo" :busy="busy === 'link'" @click="linkExisting">
							Use this repository
						</Button>
						<Button variant="quiet" :busy="busy === 'init'" @click="initRepository">
							Start a new repository
						</Button>
					</div>

					<label class="setup-field">
						<span>Remote URL (optional, for a new repository)</span>
						<input v-model="remoteUrl" placeholder="https://git.nostalgica.net/you/pack.git" />
					</label>

					<label class="setup-field">
						<span>Or clone an existing repository into the workspace</span>
						<input v-model="cloneUrl" placeholder="https://github.com/you/pack.git" />
					</label>
					<Button
						variant="secondary"
						:busy="busy === 'clone'"
						:disabled="!cloneUrl.trim()"
						@click="cloneRepository"
					>
						Clone
					</Button>

					<button type="button" class="setup-skip" @click="skipRepository">
						Skip — I'll do this later
					</button>
				</section>

				<section v-else key="credentials" class="setup-step">
					<p class="eyebrow">Step 3 of 3 · optional</p>
					<h1>Connect publishing accounts</h1>
					<p>
						Needed only to publish. You can add these any time from Settings, and browsing works
						without them.
					</p>

					<label class="setup-field">
						<span>Modrinth personal access token</span>
						<input v-model="modrinthToken" type="password" autocomplete="off" placeholder="mrp_…" />
					</label>
					<Button
						variant="secondary"
						:busy="busy === 'modrinth'"
						:disabled="!modrinthToken.trim()"
						@click="linkModrinth"
					>
						{{ accounts.modrinth?.linked ? 'Reconnect Modrinth' : 'Connect Modrinth' }}
					</Button>

					<!-- "Connect", never "Sign in": CurseForge has no third-party user
					     OAuth, so this is an API key and calling it a sign-in would
					     promise something that does not exist. -->
					<label class="setup-field">
						<span>CurseForge API key</span>
						<input v-model="curseforgeKey" type="password" autocomplete="off" placeholder="$2a$…" />
					</label>
					<Button
						variant="secondary"
						:busy="busy === 'curseforge'"
						:disabled="!curseforgeKey.trim()"
						@click="linkCurseforge"
					>
						{{ accounts.curseforge?.linked ? 'Replace CurseForge key' : 'Connect CurseForge' }}
					</Button>

					<div class="setup-actions">
						<button type="button" class="setup-skip" @click="goTo('Repository')">Back</button>
						<Button @click="finish">Open Packwand</Button>
					</div>
				</section>
			</Transition>
		</div>
	</main>
</template>
