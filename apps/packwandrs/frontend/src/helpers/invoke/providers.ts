import type {
	BrowsePage,
	BrowseQuery,
	CreatorProfile,
	ProjectPage,
	ProviderKind,
	ResolveRequest,
	ResolvedProject,
} from '../types'
import { call } from './core'

export const providerResolve = (
	provider: ProviderKind,
	request: ResolveRequest,
	token: string | null,
	instance: string | null,
) => call<ResolvedProject>('providers_resolve', { provider, request, token, instance })
export const providerAdd = (
	id: string,
	provider: ProviderKind,
	request: ResolveRequest,
	token: string | null,
	instance: string | null,
	replace = false,
) => call<string>('providers_add', { id, provider, request, token, instance, replace })

export const providerBrowse = (provider: ProviderKind, query: BrowseQuery, token: string | null) =>
	call<BrowsePage>('providers_browse', { provider, query, token })

/** Opens a provider project page in the system browser. */
export const providerOpenPage = (url: string) => call<void>('providers_open_page', { url })

/** Fetches one project, description included, for reading in-app. */
export const providerProject = (provider: ProviderKind, id: string, token: string | null = null) =>
	call<ProjectPage>('providers_project', { provider, id, token })

/** Fetches the person or team behind a project. */
export const providerCreator = (
	provider: ProviderKind,
	handle: string,
	token: string | null = null,
) => call<CreatorProfile>('providers_creator', { provider, handle, token })
