import type { ApiContract } from '../types'
import { call } from './core'

export const apiContract = () => call<ApiContract>('api_contract')
export const apiInspect = (path: string) => call<unknown>('api_inspect', { path })
