window.__packwandIdeMarkStarted?.()

const core = new URL('./core/', import.meta.url)
globalThis._VSCODE_FILE_ROOT = new URL('out/', core).href

/**
 * The editor to open at startup, from the host's `?open=` parameter.
 *
 * The path is pack-relative and arrives on the `packwand:` scheme, the same
 * virtual filesystem `folderUri` is rooted at, so it cannot address anything
 * outside the pack even if the parameter is tampered with -- the scheme has no
 * way to express one. Leading slashes are stripped so `/pack.toml` and
 * `pack.toml` mean the same file rather than one of them resolving oddly.
 */
function initialEditors() {
  const requested = new URL(window.location.href).searchParams.get('open')
  if (!requested) return []
  const path = requested.replace(/^\/+/, '')
  if (!path) return []
  return [{ uri: { scheme: 'packwand', path: `/${path}` } }]
}

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
    // The workbench is embedded as *the editor*, not as a whole IDE. Packwand
    // already has an activity rail and a sidebar around this iframe, so
    // Code-OSS showing its own produces two nested navigation surfaces with
    // two different file trees, two different search boxes, and no clear
    // answer to which one is authoritative.
    //
    // The activity bar is hidden through its supported setting. The sidebar
    // has no equivalent -- `workbench.sideBar.location` only chooses a side --
    // so it is closed by the startup layout below and kept closed by CSS in
    // bootstrap.css. See frontend/src/components/shell/FileTreeSection.vue for
    // where the explorer went.
    'workbench.activityBar.location': 'hidden',
    'workbench.statusBar.visible': false,
    'workbench.layoutControl.enabled': false,
    'workbench.editor.showTabs': 'multiple',
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
  // Start with the sidebar closed and nothing selected in it. `force` makes
  // this win over any layout state a previous session persisted, so the
  // sidebar cannot reappear because someone once opened it.
  //
  // `editors` is also how a file gets opened from outside: an embedded
  // workbench exposes no post-boot API to open one (the browser entry point
  // self-boots and never hands back the IWorkbench), so the host passes the
  // path as a query parameter and it is seeded here at load time. That is why
  // clicking a file in the Packwand sidebar reloads this iframe.
  defaultLayout: {
    force: true,
    views: [],
    editors: initialEditors(),
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
