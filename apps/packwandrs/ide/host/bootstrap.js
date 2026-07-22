window.__packwandIdeMarkStarted?.()

const core = new URL('./core/', import.meta.url)
globalThis._VSCODE_FILE_ROOT = new URL('out/', core).href

const configuration = {
  folderUri: { scheme: 'packwand', path: '/' },
  callbackRoute: new URL('out/vs/code/browser/workbench/callback.html', core).pathname,
  enableWorkspaceTrust: false,
  _wrapWebWorkerExtHostInIframe: false,
  // Upstream's default webviewEndpoint points at https://{{uuid}}.vscode-cdn.net,
  // Microsoft's CDN for the real vscode.dev/insiders builds. That host doesn't
  // serve our fork's assets (and is unreachable offline/behind the app's CSP
  // anyway), so every webview-backed surface -- image/audio/video preview,
  // notebook renderers, walkthroughs, the settings UI, etc. -- silently fails
  // to load: the inner iframe never fires 'webview-ready' and the pane just
  // stays blank. Point it at our own same-origin copy of the webview host
  // page instead (served alongside the rest of vscode-web under `core/`).
  webviewEndpoint: new URL('out/vs/workbench/contrib/webview/browser/pre/', core).href,
  configurationDefaults: {
    'workbench.colorTheme': 'Packwand Dark',
    'workbench.startupEditor': 'none',
    'workbench.enableExperiments': false,
    'workbench.tips.enabled': false,
    'chat.disableAIFeatures': true,
    'window.commandCenter': false,
    'editor.minimap.enabled': false,
    'editor.bracketPairColorization.enabled': true,
    'editor.guides.bracketPairs': true,
    'files.trimTrailingWhitespace': true,
    'telemetry.telemetryLevel': 'off',
  },
  initialColorTheme: {
    themeType: 'dark',
    colors: {
      'activityBar.background': '#252832',
      'editor.background': '#1f222a',
      'editor.foreground': '#e8eaf2',
      'sideBar.background': '#292c36',
      'statusBar.background': '#20232e',
      'titleBar.activeBackground': '#252832',
    },
  },
  productConfiguration: {
    embedderIdentifier: 'packwand',
    defaultChatAgent: {
      extensionId: 'packwand.none',
      chatExtensionId: 'packwand.none',
      provider: {
        default: { id: 'none', name: 'None' },
        enterprise: { id: 'none', name: 'None' },
      },
      providerScopes: [],
    },
  },
}

document.querySelector('#vscode-workbench-web-configuration')
  ?.setAttribute('data-settings', JSON.stringify(configuration))

function reportBootstrapFailure(error) {
  const message = error instanceof Error ? error.message : String(error)
  console.error('Packwand IDE bootstrap failed.', error)
  document.body.innerHTML = ''
  const panel = document.createElement('div')
  panel.setAttribute('role', 'alert')
  panel.style.cssText = 'box-sizing:border-box;min-height:100%;padding:24px;background:#1f222a;color:#e8eaf2;font:14px/1.5 system-ui,sans-serif;'
  panel.innerHTML = `<h1 style="margin:0 0 8px;font-size:18px">Packwand IDE could not start</h1><p style="margin:0 0 12px;color:#bbc1d1">${message.replace(/[&<>]/g, character => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' })[character])}</p><p style="margin:0;color:#8f96aa">Reload the IDE after updating Packwand. This message is intentionally visible so startup failures cannot appear as a blank editor.</p>`
  document.body.append(panel)
  window.parent?.postMessage({ channel: 'packwand:ide', type: 'bootstrap-error', message }, window.location.origin === 'null' ? '*' : window.location.origin)
}

try {
  await import(new URL('out/nls.messages.js', core).href)
  await import(new URL('out/vs/code/browser/workbench/workbench.js', core).href)
} catch (error) {
  reportBootstrapFailure(error)
}
