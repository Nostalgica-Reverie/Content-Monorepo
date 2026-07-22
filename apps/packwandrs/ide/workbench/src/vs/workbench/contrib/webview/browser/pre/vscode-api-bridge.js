/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Lasting Legacy. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//@ts-check

// Packwand: this is the `acquireVsCodeApi()` bridge that `index.html` (the
// webview host page) injects into every extension webview document via
// `toContentHtml()`. Upstream builds that script's source as an inline-text
// `<script>` element assembled at runtime (see git history / `getVsCodeApiScript`
// in `index.html`), because in a real vscode.dev-style deployment the webview
// host page runs on its own isolated `{{uuid}}` subdomain, entirely outside
// this app's CSP.
//
// Under Tauri, every page (including this one) is served from the single
// shared app origin and gets Tauri's own injected CSP, which only allows
// inline scripts whose sha256 hash it precomputed from files on disk at
// build time -- it has no way to hash something assembled at runtime with
// per-webview-varying `state`/`allowMultipleAPIAcquire` values, so the
// inline version is always blocked (silently, since the Vite dev server
// enforces no CSP at all, which is why this didn't surface until tested in
// the packaged app).
//
// An external, unchanging script file like this one is exempt from
// hash/nonce matching entirely -- CSP only gates inline script bodies and
// `javascript:` URLs, not `src`-loaded scripts -- as long as 'self' is
// allowed, which it already is. The per-webview dynamic bits are threaded
// through via `data-*` attributes on the `<script>` tag itself instead of
// inline script text, since attribute values aren't subject to script-src
// at all.
(function () {
	const scriptElement = /** @type {HTMLScriptElement} */ (document.currentScript);
	const allowMultipleAPIAcquire = scriptElement?.dataset.allowMultipleApiAcquire === 'true';
	const encodedState = scriptElement?.dataset.state;

	globalThis.acquireVsCodeApi = (function () {
		const originalPostMessage = window.parent['__vscode_post_message__'].bind(window.parent);
		const doPostMessage = (channel, data, transfer) => {
			originalPostMessage(channel, data, transfer);
		};

		let acquired = false;

		let state = encodedState ? JSON.parse(decodeURIComponent(encodedState)) : undefined;

		return () => {
			if (acquired && !allowMultipleAPIAcquire) {
				throw new Error('An instance of the VS Code API has already been acquired');
			}
			acquired = true;
			return Object.freeze({
				postMessage: function (message, transfer) {
					doPostMessage('onmessage', { message, transfer }, transfer);
				},
				setState: function (newState) {
					state = newState;
					doPostMessage('do-update-state', JSON.stringify(newState));
					return newState;
				},
				getState: function () {
					return state;
				}
			});
		};
	})();
	window.parent = window;
	window.top = window;
	window.frameElement = null;
})();
