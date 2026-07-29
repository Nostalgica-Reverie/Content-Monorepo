/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Lasting Legacy. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { mainWindow } from '../../base/browser/window.js';
import { Disposable } from '../../base/common/lifecycle.js';

export const PACKWAND_RAW_INPUT_EVENT = 'packwand:raw-input';

export interface PackwandRawInputEvent {
	kind: 'keyboard' | 'mouse';
	timestampMs: number;
	makeCode: number;
	flags: number;
	virtualKey: number;
	buttonFlags: number;
	deltaX: number;
	deltaY: number;
	wheelDelta: number;
}

interface RawInputBatch {
	channel: 'packwand:ide-raw-input';
	direction: 'batch';
	events: PackwandRawInputEvent[];
}

/**
 * Delivers app-scoped Windows Raw Input batches to IDE features without creating
 * a second native hook. Consumers listen for PACKWAND_RAW_INPUT_EVENT on the IDE
 * window and receive the native records in CustomEvent.detail.
 */
export class PackwandRawInputBridge extends Disposable {
	constructor() {
		super();
		const listener = (event: MessageEvent<RawInputBatch>) => {
			const batch = event.data;
			if (event.source !== mainWindow.parent || batch?.channel !== 'packwand:ide-raw-input' || batch.direction !== 'batch' || !Array.isArray(batch.events)) return;
			mainWindow.dispatchEvent(new CustomEvent<PackwandRawInputEvent[]>(PACKWAND_RAW_INPUT_EVENT, { detail: batch.events }));
		};
		mainWindow.addEventListener('message', listener);
		this._register({ dispose: () => mainWindow.removeEventListener('message', listener) });
	}
}
