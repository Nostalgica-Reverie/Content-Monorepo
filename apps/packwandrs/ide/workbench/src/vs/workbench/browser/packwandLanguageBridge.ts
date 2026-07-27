/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Lasting Legacy. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { mainWindow } from '../../base/browser/window.js';
import { Disposable } from '../../base/common/lifecycle.js';
import { URI } from '../../base/common/uri.js';
import { Range } from '../../editor/common/core/range.js';
import * as languages from '../../editor/common/languages.js';
import { ILanguageFeaturesService } from '../../editor/common/services/languageFeatures.js';
import { IModelService } from '../../editor/common/services/model.js';
import { IMarkerService, MarkerSeverity } from '../../platform/markers/common/markers.js';
import { PACKWAND_FILE_SYSTEM_SCHEME } from './packwandFileSystemProvider.js';

const OWNER = 'packwand.extensions';

interface EditorDiagnostic {
	path: string;
	severity: 'error' | 'warning';
	message: string;
	startLine: number;
	startColumn: number;
	endLine: number;
	endColumn: number;
}

interface EditorSymbol {
	id: string;
	kind: string;
	registry: 'datapack' | 'config' | 'resourcepack' | 'kubejs';
	path: string;
	detail: string;
}

interface LanguageSnapshot {
	version: string;
	diagnostics: EditorDiagnostic[];
	symbols: EditorSymbol[];
}

interface LanguageUpdate {
	channel: 'packwand:ide-language';
	direction: 'update';
	snapshot: LanguageSnapshot;
}

function resource(path: string): URI {
	return URI.from({ scheme: PACKWAND_FILE_SYSTEM_SCHEME, path: `/${path.replace(/^\/+/, '')}` });
}

function tokenAt(model: import('../../editor/common/model.js').ITextModel, position: import('../../editor/common/core/position.js').Position): { value: string; range: Range } | undefined {
	const line = model.getLineContent(position.lineNumber);
	const allowed = /[A-Za-z0-9_.:/-]/;
	let start = Math.max(0, position.column - 1);
	let end = start;
	while (start > 0 && allowed.test(line[start - 1])) start--;
	while (end < line.length && allowed.test(line[end])) end++;
	if (start === end) return undefined;
	return { value: line.slice(start, end), range: new Range(position.lineNumber, start + 1, position.lineNumber, end + 1) };
}

function completionKind(symbol: EditorSymbol): languages.CompletionItemKind {
	if (symbol.kind === 'mod') return languages.CompletionItemKind.Module;
	if (symbol.kind.includes('function') || symbol.kind.includes('script')) return languages.CompletionItemKind.Function;
	if (symbol.kind.includes('texture') || symbol.kind.includes('model')) return languages.CompletionItemKind.File;
	return languages.CompletionItemKind.Reference;
}

export class PackwandLanguageBridge extends Disposable {
	private snapshot: LanguageSnapshot = { version: '', diagnostics: [], symbols: [] };

	constructor(
		@IMarkerService private readonly markerService: IMarkerService,
		@ILanguageFeaturesService languageFeatures: ILanguageFeaturesService,
		@IModelService private readonly modelService: IModelService,
	) {
		super();
		const listener = (event: MessageEvent<LanguageUpdate>) => {
			const update = event.data;
			if (event.source !== mainWindow.parent || update?.channel !== 'packwand:ide-language' || update.direction !== 'update') return;
			this.snapshot = update.snapshot;
			this.publishDiagnostics();
		};
		mainWindow.addEventListener('message', listener);
		this._register({ dispose: () => mainWindow.removeEventListener('message', listener) });

		const selector = { scheme: PACKWAND_FILE_SYSTEM_SCHEME };
		this._register(languageFeatures.completionProvider.register(selector, {
			_debugDisplayName: 'Packwand project index',
			provideCompletionItems: (model, position) => {
				const token = tokenAt(model, position);
				const range = token?.range ?? new Range(position.lineNumber, position.column, position.lineNumber, position.column);
				return {
					suggestions: this.symbolsFor(model.uri.path).map((symbol, index) => ({
						label: symbol.id,
						kind: completionKind(symbol),
						detail: `${symbol.registry} · ${symbol.detail}`,
						documentation: symbol.path ? `Defined in ${symbol.path}` : symbol.detail,
						insertText: symbol.id,
						range,
						sortText: String(index).padStart(6, '0'),
					})),
				};
			},
		}));
		this._register(languageFeatures.hoverProvider.register(selector, {
			provideHover: (model, position) => {
				const token = tokenAt(model, position);
				const symbol = token && this.findSymbol(token.value);
				if (!token || !symbol) return undefined;
				return {
					range: token.range,
					contents: [{ value: `**${symbol.id}**  \n${symbol.registry} · ${symbol.detail}${symbol.path ? `  \n${symbol.path}` : ''}` }],
				};
			},
		}));
		this._register(languageFeatures.definitionProvider.register(selector, {
			provideDefinition: (model, position) => {
				const token = tokenAt(model, position);
				const symbol = token && this.findSymbol(token.value);
				return symbol?.path ? { uri: resource(symbol.path), range: new Range(1, 1, 1, 1) } : undefined;
			},
		}));
		this._register(languageFeatures.referenceProvider.register(selector, {
			provideReferences: (model, position) => {
				const token = tokenAt(model, position);
				if (!token || !this.findSymbol(token.value)) return [];
				return this.references(token.value);
			},
		}));
	}

	private publishDiagnostics(): void {
		this.markerService.changeAll(OWNER, this.snapshot.diagnostics.map(diagnostic => ({
			resource: resource(diagnostic.path),
			marker: {
				severity: diagnostic.severity === 'error' ? MarkerSeverity.Error : MarkerSeverity.Warning,
				message: diagnostic.message,
				source: 'Packwand',
				startLineNumber: diagnostic.startLine,
				startColumn: diagnostic.startColumn,
				endLineNumber: diagnostic.endLine,
				endColumn: diagnostic.endColumn,
			},
		})));
	}

	private symbolsFor(path: string): EditorSymbol[] {
		if (path.includes('/kubejs/')) return this.snapshot.symbols.filter(symbol => symbol.registry === 'kubejs');
		if (path.includes('/data/')) return this.snapshot.symbols.filter(symbol => symbol.registry === 'datapack');
		if (path.includes('/assets/')) return this.snapshot.symbols.filter(symbol => symbol.registry === 'resourcepack');
		return this.snapshot.symbols;
	}

	private findSymbol(value: string): EditorSymbol | undefined {
		return this.snapshot.symbols.find(symbol => symbol.id === value);
	}

	private references(value: string): languages.Location[] {
		const locations: languages.Location[] = [];
		for (const model of this.modelService.getModels()) {
			if (model.uri.scheme !== PACKWAND_FILE_SYSTEM_SCHEME) continue;
			for (let lineNumber = 1; lineNumber <= model.getLineCount(); lineNumber++) {
				const line = model.getLineContent(lineNumber);
				let offset = line.indexOf(value);
				while (offset >= 0) {
					locations.push({ uri: model.uri, range: new Range(lineNumber, offset + 1, lineNumber, offset + value.length + 1) });
					offset = line.indexOf(value, offset + value.length);
				}
			}
		}
		return locations;
	}
}
