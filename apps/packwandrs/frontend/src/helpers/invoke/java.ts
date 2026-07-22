import { call } from './core'

export const javaListSystem = () => call<unknown[]>('java_list_system')
export const javaRecommend = (minecraftVersion: string) => call<unknown>('java_recommend', { minecraftVersion })
export const javaTest = (path: string) => call<unknown>('java_test', { path })
export const javaInstallManaged = (major: number) => call<string>('java_install_managed', { major })
