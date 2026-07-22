import type { ProviderKind, ResolveRequest, ResolvedProject } from '../types'
import { call } from './core'

export const providerResolve = (provider: ProviderKind, request: ResolveRequest, token: string | null, instance: string | null) =>
  call<ResolvedProject>('providers_resolve', { provider, request, token, instance })
export const providerAdd = (id: string, provider: ProviderKind, request: ResolveRequest, token: string | null, instance: string | null, replace = false) =>
  call<string>('providers_add', { id, provider, request, token, instance, replace })
