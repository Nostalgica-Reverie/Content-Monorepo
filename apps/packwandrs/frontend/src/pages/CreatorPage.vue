<script setup lang="ts">
// One creator, and the projects they have published.
//
// Reached from the author line on a browse result. What can be shown depends
// entirely on the provider: Modrinth has a real user resource, CurseForge has
// none at all, so `profile.partial` drives an explicit notice rather than a
// page that merely looks empty.
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import Button from '@/components/ui/Button.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import { normalizeBridgeError } from '@/helpers/errors'
import { providerCreator, providerOpenPage } from '@/helpers/invoke/providers'
import type { CreatorProfile, ProviderKind } from '@/helpers/types'

const route = useRoute()
const router = useRouter()

const profile = ref<CreatorProfile | null>(null)
const loading = ref(false)
const error = ref('')

const provider = computed(() => String(route.params.provider ?? 'modrinth') as ProviderKind)
const handle = computed(() => String(route.params.handle ?? ''))
const providerLabel = computed(() => (provider.value === 'curse_forge' ? 'CurseForge' : 'Modrinth'))

const numberFormat = new Intl.NumberFormat()

const joined = computed(() => {
	if (!profile.value?.joined) return ''
	const parsed = new Date(profile.value.joined)
	return Number.isNaN(parsed.getTime()) ? '' : parsed.toLocaleDateString()
})

async function load() {
	if (!handle.value) return
	loading.value = true
	error.value = ''
	profile.value = null
	try {
		profile.value = await providerCreator(provider.value, handle.value)
	} catch (caught) {
		error.value = normalizeBridgeError(caught).message
	} finally {
		loading.value = false
	}
}

function openProject(id: string) {
	void router.push({ name: 'browse', query: { provider: provider.value, project: id } })
}

watch([provider, handle], load, { immediate: true })
</script>

<template>
	<section class="creator-page">
		<header class="creator-head">
			<img
				v-if="profile?.avatarUrl"
				class="creator-avatar"
				:src="profile.avatarUrl"
				:alt="profile.name"
			/>
			<div class="creator-identity">
				<p class="eyebrow">{{ providerLabel }} creator</p>
				<h1>{{ profile?.name || handle }}</h1>
				<p v-if="profile && profile.name !== profile.handle" class="creator-handle">
					{{ profile.handle }}
				</p>
				<p v-if="joined" class="creator-meta">Joined {{ joined }}</p>
			</div>
			<Button v-if="profile?.pageUrl" variant="quiet" @click="providerOpenPage(profile.pageUrl!)">
				Open on {{ providerLabel }}
			</Button>
		</header>

		<Transition name="fade">
			<div v-if="error" class="error-banner">{{ error }}</div>
		</Transition>

		<p v-if="profile?.bio" class="creator-bio">{{ profile.bio }}</p>

		<!-- Stated, not implied. A CurseForge profile is genuinely incomplete and
		     the user should know that rather than assume the creator is inactive. -->
		<p v-if="profile?.partial" class="creator-partial">
			CurseForge has no public profile API, so this page is assembled from search results. Only
			projects matching this author name are listed, and there is no avatar or bio to show.
		</p>

		<p v-if="loading" class="creator-meta">Loading…</p>

		<TransitionGroup v-if="profile?.projects.length" tag="ul" name="tab" class="creator-projects">
			<li v-for="project in profile.projects" :key="project.id">
				<button type="button" class="creator-project" @click="openProject(project.id)">
					<img v-if="project.iconUrl" :src="project.iconUrl" :alt="project.title" />
					<span class="creator-project__text">
						<strong>{{ project.title }}</strong>
						<span>{{ project.summary }}</span>
					</span>
					<span class="creator-project__downloads">
						{{ numberFormat.format(project.downloads) }}
					</span>
				</button>
			</li>
		</TransitionGroup>

		<EmptyState
			v-else-if="!loading && !error"
			title="No projects found"
			:message="`Nothing published by ${handle} was returned by ${providerLabel}.`"
		/>
	</section>
</template>
