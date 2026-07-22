/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Lasting Legacy. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { Event, Emitter } from '../../base/common/event.js';
import { Disposable, IDisposable } from '../../base/common/lifecycle.js';
import { URI } from '../../base/common/uri.js';
import {
	createFileSystemProviderError,
	FileChangeType,
	FileSystemProviderCapabilities,
	FileSystemProviderErrorCode,
	FileType,
	IFileChange,
	IFileDeleteOptions,
	IFileOverwriteOptions,
	IFileSystemProviderWithFileReadWriteCapability,
	IFileWriteOptions,
	IStat,
	IWatchOptions,
} from '../../platform/files/common/files.js';
import { mainWindow } from '../../base/browser/window.js';

export const PACKWAND_FILE_SYSTEM_SCHEME = 'packwand';
const PACKWAND_PARENT_ORIGIN = mainWindow.location.origin === 'null' ? '*' : mainWindow.location.origin;

interface BridgeError {
	kind?: string;
	message?: string;
}

interface BridgeResponse {
	channel: 'packwand:ide-fs';
	direction: 'response';
	id: number;
	result?: unknown;
	error?: BridgeError;
}

interface DirectoryEntry {
	name: string;
	fileType: FileType;
}

interface FileStat {
	fileType: FileType;
	size: number;
	ctime: number;
	mtime: number;
}

interface PendingRequest {
	resolve(value: unknown): void;
	reject(error: Error): void;
	timeout: number;
}

function relativePath(resource: URI): string {
	return resource.path.replace(/^\/+/, '');
}

function providerError(error: BridgeError | undefined): Error {
	const message = error?.message || 'Packwand did not return a filesystem response.';
	let code = FileSystemProviderErrorCode.Unknown;
	switch (error?.kind) {
		case 'not_found': code = FileSystemProviderErrorCode.FileNotFound; break;
		case 'already_exists': code = FileSystemProviderErrorCode.FileExists; break;
		case 'is_directory': code = FileSystemProviderErrorCode.FileIsADirectory; break;
		case 'not_directory': code = FileSystemProviderErrorCode.FileNotADirectory; break;
		case 'permission_denied':
		case 'unsafe_path': code = FileSystemProviderErrorCode.NoPermissions; break;
		case 'unavailable': code = FileSystemProviderErrorCode.Unavailable; break;
	}
	return createFileSystemProviderError(message, code);
}

export class PackwandFileSystemProvider extends Disposable implements IFileSystemProviderWithFileReadWriteCapability {

	readonly capabilities = FileSystemProviderCapabilities.FileReadWrite;
	readonly onDidChangeCapabilities = Event.None;

	private readonly changeEmitter = this._register(new Emitter<readonly IFileChange[]>());
	readonly onDidChangeFile = this.changeEmitter.event;

	private nextRequestId = 0;
	private readonly pending = new Map<number, PendingRequest>();

	constructor() {
		super();
		const listener = (event: MessageEvent<BridgeResponse>) => {
			const response = event.data;
			if (event.source !== mainWindow.parent || response?.channel !== 'packwand:ide-fs' || response.direction !== 'response') {
				return;
			}
			const pending = this.pending.get(response.id);
			if (!pending) {
				return;
			}
			mainWindow.clearTimeout(pending.timeout);
			this.pending.delete(response.id);
			if (response.error) {
				pending.reject(providerError(response.error));
			} else {
				pending.resolve(response.result);
			}
		};
		mainWindow.addEventListener('message', listener);
		this._register({ dispose: () => mainWindow.removeEventListener('message', listener) });
	}

	private request<T>(method: string, parameters: Record<string, unknown>): Promise<T> {
		if (mainWindow.parent === mainWindow) {
			return Promise.reject(providerError({ kind: 'unavailable', message: 'Packwand IDE must be opened inside the Packwand application.' }));
		}
		const id = ++this.nextRequestId;
		return new Promise<T>((resolve, reject) => {
			const timeout = mainWindow.setTimeout(() => {
				this.pending.delete(id);
				reject(providerError({ kind: 'unavailable', message: `Packwand filesystem request ${method} timed out.` }));
			}, 30_000);
			this.pending.set(id, { resolve: value => resolve(value as T), reject, timeout });
			mainWindow.parent.postMessage({
				channel: 'packwand:ide-fs',
				direction: 'request',
				id,
				method,
				parameters,
			}, PACKWAND_PARENT_ORIGIN);
		});
	}

	async stat(resource: URI): Promise<IStat> {
		const stat = await this.request<FileStat>('stat', { path: relativePath(resource) });
		return { type: stat.fileType, size: stat.size, ctime: stat.ctime, mtime: stat.mtime };
	}

	async readdir(resource: URI): Promise<[string, FileType][]> {
		const entries = await this.request<DirectoryEntry[]>('readDir', { path: relativePath(resource) });
		return entries.map(entry => [entry.name, entry.fileType]);
	}

	async readFile(resource: URI): Promise<Uint8Array> {
		const content = await this.request<number[]>('readFile', { path: relativePath(resource) });
		return Uint8Array.from(content);
	}

	async writeFile(resource: URI, content: Uint8Array, options: IFileWriteOptions): Promise<void> {
		await this.request('writeFile', {
			path: relativePath(resource),
			content: Array.from(content),
			create: options.create,
			overwrite: options.overwrite,
		});
		this.changeEmitter.fire([{ type: FileChangeType.UPDATED, resource }]);
	}

	async mkdir(resource: URI): Promise<void> {
		await this.request('createDir', { path: relativePath(resource) });
		this.changeEmitter.fire([{ type: FileChangeType.ADDED, resource }]);
	}

	async delete(resource: URI, options: IFileDeleteOptions): Promise<void> {
		await this.request('delete', { path: relativePath(resource), recursive: options.recursive });
		this.changeEmitter.fire([{ type: FileChangeType.DELETED, resource }]);
	}

	async rename(from: URI, to: URI, options: IFileOverwriteOptions): Promise<void> {
		await this.request('rename', {
			from: relativePath(from),
			to: relativePath(to),
			overwrite: options.overwrite,
		});
		this.changeEmitter.fire([
			{ type: FileChangeType.DELETED, resource: from },
			{ type: FileChangeType.ADDED, resource: to },
		]);
	}

	watch(_resource: URI, _options: IWatchOptions): IDisposable {
		return Disposable.None;
	}

	override dispose(): void {
		for (const [id, pending] of this.pending) {
			mainWindow.clearTimeout(pending.timeout);
			pending.reject(providerError({ kind: 'unavailable', message: `Packwand filesystem request ${id} was cancelled.` }));
		}
		this.pending.clear();
		super.dispose();
	}
}
