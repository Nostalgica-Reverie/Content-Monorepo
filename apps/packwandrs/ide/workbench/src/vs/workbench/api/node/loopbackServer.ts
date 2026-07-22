/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { randomBytes } from 'crypto';
import type * as http from 'http';
import { URL } from 'url';
import { DeferredPromise } from '../../../base/common/async.js';
import { DEFAULT_AUTH_FLOW_PORT } from '../../../base/common/oauth.js';
import { URI } from '../../../base/common/uri.js';
import { ILogger } from '../../../platform/log/common/log.js';

export interface IOAuthResult {
	code: string;
	state: string;
}

export interface ILoopbackServer {
	/**
	 * The state parameter used in the OAuth flow.
	 */
	readonly state: string;

	/**
	 * Starts the server.
	 * @throws If the server fails to start.
	 * @throws If the server is already started.
	 */
	start(): Promise<void>;

	/**
	 * Stops the server.
	 * @throws If the server is not started.
	 * @throws If the server fails to stop.
	 */
	stop(): Promise<void>;

	/**
	 * Returns a promise that resolves to the result of the OAuth flow.
	 */
	waitForOAuthResponse(): Promise<IOAuthResult>;
}

export class LoopbackAuthServer implements ILoopbackServer {
	private readonly _server: Promise<http.Server>;
	private readonly _resultPromise: Promise<IOAuthResult>;

	private _state = randomBytes(16).toString('base64');
	private _port: number | undefined;

	constructor(
		private readonly _logger: ILogger,
		private readonly _appUri: URI,
		private readonly _appName: string
	) {
		const deferredPromise = new DeferredPromise<IOAuthResult>();
		this._resultPromise = deferredPromise.p;

		this._server = (async () => {
			const http = await import('http');

			return http.createServer((req, res) => {
				const reqUrl = new URL(req.url!, `http://${req.headers.host}`);
				switch (reqUrl.pathname) {
					case '/': {
						const code = reqUrl.searchParams.get('code') ?? undefined;
						const state = reqUrl.searchParams.get('state') ?? undefined;
						const error = reqUrl.searchParams.get('error') ?? undefined;
						if (error) {
							res.writeHead(302, { location: `/done?error=${reqUrl.searchParams.get('error_description') || error}` });
							res.end();
							deferredPromise.error(new Error(error));
							break;
						}
						if (!code || !state) {
							res.writeHead(400);
							res.end();
							break;
						}
						if (this.state !== state) {
							res.writeHead(302, { location: `/done?error=${encodeURIComponent('State does not match.')}` });
							res.end();
							deferredPromise.error(new Error('State does not match.'));
							break;
						}
						deferredPromise.complete({ code, state });
						res.writeHead(302, { location: '/done' });
						res.end();
						break;
					}
					// Serve the static files
					case '/done':
						this._sendPage(res);
						break;
					default:
						res.writeHead(404);
						res.end();
						break;
				}
			});
		})();
	}

	get state(): string { return this._state; }
	get redirectUri(): string {
		if (this._port === undefined) {
			throw new Error('Server is not started yet');
		}
		return `http://127.0.0.1:${this._port}/`;
	}

	private _sendPage(res: http.ServerResponse): void {
		const html = this.getHtml();
		res.writeHead(200, {
			'Content-Type': 'text/html',
			'Content-Length': Buffer.byteLength(html, 'utf8')
		});
		res.end(html);
	}

	async start(): Promise<void> {
		const server = await this._server;
		const deferredPromise = new DeferredPromise<void>();
		if (server.listening) {
			throw new Error('Server is already started');
		}
		const portTimeout = setTimeout(() => {
			deferredPromise.error(new Error('Timeout waiting for port'));
		}, 5000);
		server.on('listening', () => {
			const address = server.address();
			if (typeof address === 'string') {
				this._port = parseInt(address);
			} else if (address instanceof Object) {
				this._port = address.port;
			} else {
				throw new Error('Unable to determine port');
			}

			clearTimeout(portTimeout);
			deferredPromise.complete();
		});
		server.on('error', err => {
			if ('code' in err && err.code === 'EADDRINUSE') {
				this._logger.error('Address in use, retrying with a different port...');
				// Best effort to use a specific port, but fallback to a random one if it is in use
				server.listen(0, '127.0.0.1');
				return;
			}
			clearTimeout(portTimeout);
			deferredPromise.error(new Error(`Error listening to server: ${err}`));
		});
		server.on('close', () => {
			deferredPromise.error(new Error('Closed'));
		});
		// Best effort to use a specific port, but fallback to a random one if it is in use
		server.listen(DEFAULT_AUTH_FLOW_PORT, '127.0.0.1');
		return deferredPromise.p;
	}

	async stop(): Promise<void> {
		const deferredPromise = new DeferredPromise<void>();
		const server = await this._server;
		if (!server.listening) {
			deferredPromise.complete();
			return deferredPromise.p;
		}
		server.close((err) => {
			if (err) {
				deferredPromise.error(err);
			} else {
				deferredPromise.complete();
			}
		});
		// If the server is not closed within 5 seconds, reject the promise
		setTimeout(() => {
			if (!deferredPromise.isResolved) {
				deferredPromise.error(new Error('Timeout waiting for server to close'));
			}
		}, 5000);
		return deferredPromise.p;
	}

	waitForOAuthResponse(): Promise<IOAuthResult> {
		return this._resultPromise;
	}

	getHtml(): string {
		// TODO: Bring this in via mixin. Skipping exploration for now.
		let backgroundImage = 'url(\'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAABAAAAAQABAMAAACNMzawAAAABGdBTUEAALGPC/xhBQAAACBjSFJNAAB6JgAAgIQAAPoAAACA6AAAdTAAAOpgAAA6mAAAF3CculE8AAAAJ1BMVEUAAAD///9Qm8+ozed8tNsWer+Lvd9trNfF3u9Ck8slgsMzi8eZxeM/Qa6mAAAAAXRSTlMAQObYZgAAAAFiS0dEAf8CLd4AAAAHdElNRQfiCwYULRt0g+ZLAAAJRUlEQVR42u3SUY0CQRREUSy0hSZtBA+wfOwv4wAPYwAJSMAfAthkB6YD79HnKqikzmYjSZIkSZIkSZIkSZIkSZIkSZIkSZIkSZIkSZIkSZIkSZIkSZIkSZIkSZIkSZIkSZIkSZIk6cmKhg4AAASAABAAAkAACAABIAAEgAAQAAJAAAgAAaBvBVAjtV2wfFe1ogcA+0gdFgBoe60IAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAnAjAP/1MkWoAvBvAsUTqBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJAAgAASAABIAAEAACQAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKA7gDlbFwC6AijZagAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQDM2boA0BXAqAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIATARAAAkAAvF7N1hWArgBKthoAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAfBrAlK0bAF0BjBoAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4EQABIAAEwOtN2boB0BVAyVYDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgE8DqNm6AtAVwKgBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAEwEQAAJAAPzd7xypMwDvBnAskToBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAIAAkAACAABIAAEgAAQAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAkAACAABIAAEgAAQAAJAAAgAASAABIAAEAACQAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIBKBGarsAwK5qRQ8ANGYAACAABIAAEAACQAAIAAEgAASAABAAAkDfCUCSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJEmSJC3uDtO80OSql+i8AAAAJXRFWHRkYXRlOmNyZWF0ZQAyMDE4LTExLTA2VDIwOjQ1OjI3KzAwOjAwEjLurQAAACV0RVh0ZGF0ZTptb2RpZnkAMjAxOC0xMS0wNlQyMDo0NToyNyswMDowMGNvVhEAAAAZdEVYdFNvZnR3YXJlAEFkb2JlIEltYWdlUmVhZHlxyWU8AAAAAElFTkSuQmCC\')';
		return `<!DOCTYPE html>
<html lang="en">

<head>
	<meta charset="utf-8" />
	<title>GitHub Authentication - Sign In</title>
	<meta name="viewport" content="width=device-width, initial-scale=1">
	<style>
		html {
			height: 100%;
		}

		body {
			box-sizing: border-box;
			min-height: 100%;
			margin: 0;
			padding: 15px 30px;
			display: flex;
			flex-direction: column;
			color: white;
			font-family: "Segoe UI","Helvetica Neue","Helvetica",Arial,sans-serif;
			background-color: #2C2C32;
		}

		.branding {
			background-image: ${backgroundImage};
			background-size: 24px;
			background-repeat: no-repeat;
			background-position: left center;
			padding-left: 36px;
			font-size: 20px;
			letter-spacing: -0.04rem;
			font-weight: 400;
			color: white;
			text-decoration: none;
		}

		.message-container {
			flex-grow: 1;
			display: flex;
			align-items: center;
			justify-content: center;
			margin: 0 30px;
		}

		.message {
			font-weight: 300;
			font-size: 1.4rem;
		}

		body.error .message {
			display: none;
		}

		body.error .error-message {
			display: block;
		}

		.error-message {
			display: none;
			font-weight: 300;
			font-size: 1.3rem;
		}

		.error-text {
			color: red;
			font-size: 1rem;
		}

		@font-face {
			font-family: 'Segoe UI';
			src: url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/light/latest.eot"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/light/latest.eot?#iefix") format("embedded-opentype");
			src: local("Segoe UI Light"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/light/latest.woff2") format("woff2"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/light/latest.woff") format("woff"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/light/latest.ttf") format("truetype"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/light/latest.svg#web") format("svg");
			font-weight: 200
		}

		@font-face {
			font-family: 'Segoe UI';
			src: url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semilight/latest.eot"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semilight/latest.eot?#iefix") format("embedded-opentype");
			src: local("Segoe UI Semilight"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semilight/latest.woff2") format("woff2"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semilight/latest.woff") format("woff"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semilight/latest.ttf") format("truetype"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semilight/latest.svg#web") format("svg");
			font-weight: 300
		}

		@font-face {
			font-family: 'Segoe UI';
			src: url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/normal/latest.eot"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/normal/latest.eot?#iefix") format("embedded-opentype");
			src: local("Segoe UI"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/normal/latest.woff2") format("woff"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/normal/latest.woff") format("woff"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/normal/latest.ttf") format("truetype"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/normal/latest.svg#web") format("svg");
			font-weight: 400
		}

		@font-face {
			font-family: 'Segoe UI';
			src: url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semibold/latest.eot"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semibold/latest.eot?#iefix") format("embedded-opentype");
			src: local("Segoe UI Semibold"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semibold/latest.woff2") format("woff"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semibold/latest.woff") format("woff"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semibold/latest.ttf") format("truetype"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/semibold/latest.svg#web") format("svg");
			font-weight: 600
		}

		@font-face {
			font-family: 'Segoe UI';
			src: url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/bold/latest.eot"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/bold/latest.eot?#iefix") format("embedded-opentype");
			src: local("Segoe UI Bold"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/bold/latest.woff2") format("woff"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/bold/latest.woff") format("woff"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/bold/latest.ttf") format("truetype"),url("https://c.s-microsoft.com/static/fonts/segoe-ui/west-european/bold/latest.svg#web") format("svg");
			font-weight: 700
		}
	</style>
</head>

<body>
	<a class="branding" href="https://code.visualstudio.com/">
		${this._appName}
	</a>
	<div class="message-container">
		<div class="message">
			Sign-in successful! Returning to ${this._appName}...
			<br><br>
			If you're not redirected automatically, <a href="${this._appUri.toString(true)}" style="color: #85CEFF;">click here</a> or close this page.
		</div>
		<div class="error-message">
			An error occurred while signing in:
			<div class="error-text"></div>
		</div>
	</div>
	<script>
		const search = window.location.search;
		const error = (/[?&^]error=([^&]+)/.exec(search) || [])[1];
		if (error) {
			document.querySelector('.error-text')
				.textContent = decodeURIComponent(error);
			document.querySelector('body')
				.classList.add('error');
		} else {
			// Redirect to the app URI after a 1-second delay to allow page to load
			setTimeout(function() {
				window.location.href = '${this._appUri.toString(true)}';
			}, 1000);
		}
	</script>
</body>
</html>`;
	}
}
