import { call } from './core'

export const webviewOpen = (provider: string, items: unknown[]) => call<string>('webview_open', { provider, items })
export const webviewClose = (handle: string) => call<void>('webview_close', { handle })
