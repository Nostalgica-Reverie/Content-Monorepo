/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import * as path from 'node:path';
import { createRequire } from 'node:module';
import './bootstrap-cli.js'; // this MUST come before other imports as it changes global state
import { configurePortable } from './bootstrap-node.js';
import { bootstrapESM } from './bootstrap-esm.js';
import { resolveNLSConfiguration } from './vs/base/node/nls.js';
import { product } from './bootstrap-meta.js';
import { getUserDataPath } from './vs/platform/environment/node/userDataPath.js';

const require = createRequire(import.meta.url);
// NLS
const nlsConfiguration = await resolveNLSConfiguration({ userLocale: 'en', osLocale: 'en', commit: product.commit, userDataPath: '', nlsMetadataPath: import.meta.dirname });
process.env['VSCODE_NLS_CONFIG'] = JSON.stringify(nlsConfiguration); // required for `bootstrap-esm` to pick up NLS messages

// Enable portable support
configurePortable(product);

// Signal processes that we got launched as CLI
process.env['VSCODE_CLI'] = '1';

resolveUserProduct();

// Bootstrap ESM
await bootstrapESM();

// Load Server
await import('./vs/code/node/cli.js');

function resolveUserProduct() {
	const userDataPath = getUserDataPath({_:[]}, product.nameShort ?? 'code-oss-dev');
	const userProductPath = path.join(userDataPath, 'product.json');

	try {
		// Assign the product configuration to the global scope
		const productJson = require(userProductPath);

		globalThis._VSCODE_USER_PRODUCT_JSON = productJson;
	} catch (ex) {
	}
}