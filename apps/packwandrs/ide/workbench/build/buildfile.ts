/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import type { IEntryPoint } from './lib/bundle.ts';

function createModuleDescription(name: string): IEntryPoint {
	return {
		name
	};
}

export const workerEditor = createModuleDescription('vs/editor/common/services/editorWebWorkerMain');
export const workerExtensionHost = createModuleDescription('vs/workbench/api/worker/extensionHostWorkerMain');
export const workerNotebook = createModuleDescription('vs/workbench/contrib/notebook/common/services/notebookWebWorkerMain');
export const workerLanguageDetection = createModuleDescription('vs/workbench/services/languageDetection/browser/languageDetectionWebWorkerMain');
export const workerLocalFileSearch = createModuleDescription('vs/workbench/services/search/worker/localFileSearchMain');
export const workerOutputLinks = createModuleDescription('vs/workbench/contrib/output/common/outputLinkComputerMain');
export const workerBackgroundTokenization = createModuleDescription('vs/workbench/services/textMate/browser/backgroundTokenization/worker/textMateTokenizationWorker.workerMain');

export const workbenchWeb = createModuleDescription('vs/workbench/workbench.web.main.internal');

export const sessionsWeb = createModuleDescription('vs/sessions/sessions.web.main.internal');

export const keyboardMaps = [
	createModuleDescription('vs/workbench/services/keybinding/browser/keyboardLayouts/layout.contribution.linux'),
	createModuleDescription('vs/workbench/services/keybinding/browser/keyboardLayouts/layout.contribution.darwin'),
	createModuleDescription('vs/workbench/services/keybinding/browser/keyboardLayouts/layout.contribution.win')
];

export const codeWeb = createModuleDescription('vs/code/browser/workbench/workbench');

export const codeServer = [
	// 'vs/server/node/server.main' is not included here because it gets inlined via ./src/server-main.js
	// 'vs/server/node/server.cli' is not included here because it gets inlined via ./src/server-cli.js
	createModuleDescription('vs/workbench/api/node/extensionHostProcess'),
	createModuleDescription('vs/platform/files/node/watcher/watcherMain'),
	createModuleDescription('vs/platform/terminal/node/ptyHostMain'),
	createModuleDescription('vs/platform/agentHost/node/agentHostMain'),
	createModuleDescription('vs/platform/agentHost/node/diffWorkerMain'),
];

export const entrypoint = createModuleDescription;

const buildfile = {
	workerEditor,
	workerExtensionHost,
	workerNotebook,
	workerLanguageDetection,
	workerLocalFileSearch,
	workerOutputLinks,
	workerBackgroundTokenization,
	workbenchWeb,
	sessionsWeb,
	keyboardMaps,
	codeWeb,
	codeServer,
	entrypoint: createModuleDescription
};

export default buildfile;
