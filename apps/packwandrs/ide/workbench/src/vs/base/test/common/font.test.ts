/*---------------------------------------------------------------------------------------------
 *  Copyright (c) VSCodium. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import assert from 'assert';
import { ensureNoDisposablesAreLeakedInTestSuite } from './utils.js';
import { FONT, updateSidebarSize, updateStatusBarSize, updatePanelSize, updateActivityBarSize, updateTabsSize } from '../../common/font.js';

/**
 * Test file for src/vs/base/common/font.ts
 *
 * Tests all update*Size() functions for:
 *   - Default values match upstream VS Code hardcoded constants
 *   - Proportional scaling preserves ratios at non-default sizes
 *   - Boundary values (minimum=6, maximum=32) produce positive values
 *   - Reset to default restores original values
 *   - Non-divisible sizes produce consistent coefficient-based output
 *   - Cross-area updates do not mutate unrelated fields
 */

const EPSILON = 1e-9;

/** Assert two numbers are equal within floating-point tolerance */
function assertClose(actual: number, expected: number, message?: string): void {
	const diff = Math.abs(actual - expected);
	assert.ok(diff < EPSILON, `${message ?? ''} expected ${expected}, got ${actual} (diff: ${diff})`);
}

/**
 * Snapshot all FONT fields to detect unintended mutations.
 * Returns a plain object copy of every enumerable property.
 */
function snapshotFont(): Record<string, number> {
	const snap: Record<string, number> = {};
	for (const key of Object.keys(FONT)) {
		snap[key] = (FONT as Record<string, number>)[key];
	}
	return snap;
}

/** Assert that specific fields in FONT have not changed from a snapshot */
function assertFieldsUnchanged(snapshot: Record<string, number>, prefix: string, message: string): void {
	for (const key of Object.keys(snapshot)) {
		if (key.startsWith(prefix)) {
			assertClose((FONT as Record<string, number>)[key], snapshot[key], `${message}: ${key}`);
		}
	}
}

suite('FONT - Sidebar Size', () => {

	ensureNoDisposablesAreLeakedInTestSuite();

	// Capture defaults before any test mutates them
	const DEFAULTS = snapshotFont();

	teardown(() => {
		// Reset after each test
		updateSidebarSize(13);
	});

	test('defaults match upstream VS Code constants', () => {
		assert.strictEqual(FONT.sidebarSize, 13);
		assert.strictEqual(FONT.sidebarSize8, 8);
		assert.strictEqual(FONT.sidebarSize10, 10);
		assert.strictEqual(FONT.sidebarSize16, 16);
		assert.strictEqual(FONT.sidebarSize17, 17);
		assert.strictEqual(FONT.sidebarSize18, 18);
		assert.strictEqual(FONT.sidebarSize20, 20);
		assert.strictEqual(FONT.sidebarSize22, 22);
		assert.strictEqual(FONT.sidebarSize23, 23);
		assert.strictEqual(FONT.sidebarSize24, 24);
		assert.strictEqual(FONT.sidebarSize26, 26);
		assert.strictEqual(FONT.sidebarSize28, 28);
		assert.strictEqual(FONT.sidebarSize34, 34);
		assert.strictEqual(FONT.sidebarSize39, 39);
		assert.strictEqual(FONT.sidebarSize44, 44);
		assert.strictEqual(FONT.sidebarSize62, 62);
		assert.strictEqual(FONT.sidebarSize72, 72);
	});

	test('proportional scaling preserves ratios at 2x', () => {
		updateSidebarSize(26); // 2x default
		assert.strictEqual(FONT.sidebarSize, 26);
		assert.strictEqual(FONT.sidebarSize8, 16);
		assert.strictEqual(FONT.sidebarSize22, 44);
		assert.strictEqual(FONT.sidebarSize44, 88);
		assert.strictEqual(FONT.sidebarSize72, 144);
	});

	test('non-divisible size produces consistent coefficient-based values', () => {
		updateSidebarSize(7);
		assertClose(FONT.sidebarSize22, 7 * (22 / 13));
		assertClose(FONT.sidebarSize44, 7 * (44 / 13));
		assertClose(FONT.sidebarSize72, 7 * (72 / 13));
	});

	test('another non-divisible size (31)', () => {
		updateSidebarSize(31);
		assertClose(FONT.sidebarSize22, 31 * (22 / 13));
		assertClose(FONT.sidebarSize44, 31 * (44 / 13));
	});

	test('minimum value (6) produces positive values', () => {
		updateSidebarSize(6);
		assert.strictEqual(FONT.sidebarSize, 6);
		assert.ok(FONT.sidebarSize8 > 0, 'sidebarSize8 must be positive');
		assert.ok(FONT.sidebarSize10 > 0, 'sidebarSize10 must be positive');
		assert.ok(FONT.sidebarSize22 > 0, 'sidebarSize22 must be positive');
		assert.ok(FONT.sidebarSize72 > 0, 'sidebarSize72 must be positive');
	});

	test('maximum value (32) produces reasonable values', () => {
		updateSidebarSize(32);
		assert.strictEqual(FONT.sidebarSize, 32);
		assert.ok(FONT.sidebarSize22 > 40, 'row height should scale up');
		assert.ok(FONT.sidebarSize22 < 80, 'row height should not be extreme');
	});

	test('reset to default restores all values', () => {
		updateSidebarSize(20);
		assert.notStrictEqual(FONT.sidebarSize22, DEFAULTS.sidebarSize22);
		updateSidebarSize(13);
		assertFieldsUnchanged(DEFAULTS, 'sidebar', 'after reset');
	});

	test('multiple updates in sequence are idempotent at same value', () => {
		updateSidebarSize(18);
		const snap = snapshotFont();
		updateSidebarSize(18);
		assertFieldsUnchanged(snap, 'sidebar', 'idempotent');
	});
});

suite('FONT - Status Bar Size', () => {

	ensureNoDisposablesAreLeakedInTestSuite();

	teardown(() => {
		updateStatusBarSize(12);
	});

	test('defaults match upstream VS Code status bar constants', () => {
		assert.strictEqual(FONT.statusBarSize, 12);
		assert.strictEqual(FONT.statusBarSize22, 22); // StatusbarPart.HEIGHT
	});

	test('proportional scaling at 2x', () => {
		updateStatusBarSize(24);
		assert.strictEqual(FONT.statusBarSize, 24);
		assert.strictEqual(FONT.statusBarSize22, 44);
	});

	test('non-divisible size (7)', () => {
		updateStatusBarSize(7);
		assertClose(FONT.statusBarSize22, 7 * (22 / 12));
	});

	test('minimum value (6) produces positive height', () => {
		updateStatusBarSize(6);
		assert.strictEqual(FONT.statusBarSize, 6);
		assert.ok(FONT.statusBarSize22 > 0, 'height must be positive');
		assertClose(FONT.statusBarSize22, 6 * (22 / 12));
	});

	test('maximum value (32) produces reasonable height', () => {
		updateStatusBarSize(32);
		assertClose(FONT.statusBarSize22, 32 * (22 / 12));
		assert.ok(FONT.statusBarSize22 > 50, 'height should scale up');
		assert.ok(FONT.statusBarSize22 < 70, 'height should not be extreme');
	});

	test('reset to default restores values', () => {
		updateStatusBarSize(20);
		updateStatusBarSize(12);
		assert.strictEqual(FONT.statusBarSize, 12);
		assert.strictEqual(FONT.statusBarSize22, 22);
	});
});

suite('FONT - Panel Size', () => {

	ensureNoDisposablesAreLeakedInTestSuite();

	teardown(() => {
		updatePanelSize(13);
	});

	test('defaults match sidebar defaults (same base size)', () => {
		assert.strictEqual(FONT.bottomPaneSize, 13);
		assert.strictEqual(FONT.bottomPaneSize22, 22);
	});

	test('panel and sidebar can have independent sizes', () => {
		updatePanelSize(18);
		assert.strictEqual(FONT.bottomPaneSize, 18);
		assert.notStrictEqual(FONT.bottomPaneSize22, 22);
		// Sidebar unchanged
		assert.strictEqual(FONT.sidebarSize, 13);
		assert.strictEqual(FONT.sidebarSize22, 22);
	});

	test('proportional scaling at 2x', () => {
		updatePanelSize(26);
		assert.strictEqual(FONT.bottomPaneSize22, 44);
	});

	test('non-divisible size (7)', () => {
		updatePanelSize(7);
		assertClose(FONT.bottomPaneSize22, 7 * (22 / 13));
	});

	test('minimum value (6)', () => {
		updatePanelSize(6);
		assert.ok(FONT.bottomPaneSize22 > 0, 'row height must be positive');
		assert.ok(FONT.bottomPaneSize22 >= 10, 'row height at minimum should be usable');
	});

	test('maximum value (32)', () => {
		updatePanelSize(32);
		assert.ok(FONT.bottomPaneSize22 > 50, 'row height should scale up');
	});

	test('reset to default restores values', () => {
		updatePanelSize(20);
		updatePanelSize(13);
		assert.strictEqual(FONT.bottomPaneSize, 13);
		assert.strictEqual(FONT.bottomPaneSize22, 22);
	});
});

suite('FONT - Activity Bar Size', () => {

	ensureNoDisposablesAreLeakedInTestSuite();

	teardown(() => {
		updateActivityBarSize(16);
	});

	test('defaults match upstream VS Code activity bar constants', () => {
		assert.strictEqual(FONT.activityBarSize, 16);
		assert.strictEqual(FONT.activityBarSize16, 16);  // COMPACT_ICON_SIZE
		assert.strictEqual(FONT.activityBarSize24, 24);  // ICON_SIZE
		assert.strictEqual(FONT.activityBarSize32, 32);  // COMPACT_ACTION_HEIGHT
		assert.strictEqual(FONT.activityBarSize36, 36);  // COMPACT_ACTIVITYBAR_WIDTH
		assert.strictEqual(FONT.activityBarSize48, 48);  // ACTION_HEIGHT / ACTIVITYBAR_WIDTH
	});

	test('proportional scaling at 2x', () => {
		updateActivityBarSize(32);
		assert.strictEqual(FONT.activityBarSize, 32);
		assert.strictEqual(FONT.activityBarSize16, 32);  // compact icon = base size
		assert.strictEqual(FONT.activityBarSize24, 48);
		assert.strictEqual(FONT.activityBarSize32, 64);
		assert.strictEqual(FONT.activityBarSize36, 72);
		assert.strictEqual(FONT.activityBarSize48, 96);
	});

	test('compact constants scale correctly', () => {
		updateActivityBarSize(20);
		// Compact icon size = base size * (16/16) = 20
		assertClose(FONT.activityBarSize16, 20 * (16 / 16));
		// Compact width = base * (36/16) = 45
		assertClose(FONT.activityBarSize36, 20 * (36 / 16));
		// Compact action height = base * (32/16) = 40
		assertClose(FONT.activityBarSize32, 20 * (32 / 16));
	});

	test('non-divisible size (7)', () => {
		updateActivityBarSize(7);
		assertClose(FONT.activityBarSize24, 7 * (24 / 16));
		assertClose(FONT.activityBarSize48, 7 * (48 / 16));
		assertClose(FONT.activityBarSize36, 7 * (36 / 16));
	});

	test('minimum value (6) produces positive values', () => {
		updateActivityBarSize(6);
		assert.strictEqual(FONT.activityBarSize, 6);
		assert.ok(FONT.activityBarSize16 > 0, 'compact icon size must be positive');
		assert.ok(FONT.activityBarSize24 > 0, 'icon size must be positive');
		assert.ok(FONT.activityBarSize36 > 0, 'compact width must be positive');
		assert.ok(FONT.activityBarSize48 > 0, 'action height must be positive');
	});

	test('maximum value (32) produces reasonable values', () => {
		updateActivityBarSize(32);
		assert.strictEqual(FONT.activityBarSize48, 96);
		assert.ok(FONT.activityBarSize48 <= 100, 'action height should be bounded');
	});

	test('reset to default restores values', () => {
		updateActivityBarSize(24);
		updateActivityBarSize(16);
		assert.strictEqual(FONT.activityBarSize, 16);
		assert.strictEqual(FONT.activityBarSize16, 16);
		assert.strictEqual(FONT.activityBarSize24, 24);
		assert.strictEqual(FONT.activityBarSize32, 32);
		assert.strictEqual(FONT.activityBarSize36, 36);
		assert.strictEqual(FONT.activityBarSize48, 48);
	});
});

suite('FONT - Tabs Size', () => {

	ensureNoDisposablesAreLeakedInTestSuite();

	teardown(() => {
		updateTabsSize(13);
	});

	test('defaults match upstream VS Code tab constants', () => {
		assert.strictEqual(FONT.tabsSize, 13);
		assert.strictEqual(FONT.tabsSize22, 22);   // EDITOR_TAB_HEIGHT compact
		assert.strictEqual(FONT.tabsSize35, 35);   // EDITOR_TAB_HEIGHT normal
		assert.strictEqual(FONT.tabsSize38, 38);   // TAB_WIDTH compact
		assert.strictEqual(FONT.tabsSize80, 80);   // TAB_WIDTH shrink
		assert.strictEqual(FONT.tabsSize120, 120); // TAB_WIDTH fit
	});

	test('proportional scaling preserves tab height/width ratios at 2x', () => {
		updateTabsSize(26);
		assert.strictEqual(FONT.tabsSize, 26);
		assert.strictEqual(FONT.tabsSize22, 44);
		assert.strictEqual(FONT.tabsSize35, 70);
		assert.strictEqual(FONT.tabsSize38, 76);
		assert.strictEqual(FONT.tabsSize80, 160);
		assert.strictEqual(FONT.tabsSize120, 240);
	});

	test('non-divisible size (7)', () => {
		updateTabsSize(7);
		assertClose(FONT.tabsSize22, 7 * (22 / 13));
		assertClose(FONT.tabsSize35, 7 * (35 / 13));
		assertClose(FONT.tabsSize80, 7 * (80 / 13));
		assertClose(FONT.tabsSize120, 7 * (120 / 13));
	});

	test('minimum value (6) produces usable tab dimensions', () => {
		updateTabsSize(6);
		assert.strictEqual(FONT.tabsSize, 6);
		assert.ok(FONT.tabsSize35 > 10, 'normal tab height must be clickable');
		assert.ok(FONT.tabsSize22 > 8, 'compact tab height must be usable');
		assert.ok(FONT.tabsSize38 > 0, 'compact tab width must be positive');
		assert.ok(FONT.tabsSize80 > 0, 'shrink tab width must be positive');
	});

	test('maximum value (32) produces reasonable tab dimensions', () => {
		updateTabsSize(32);
		assert.ok(FONT.tabsSize35 > 80, 'normal tab height should scale up');
		assert.ok(FONT.tabsSize35 < 100, 'normal tab height should be bounded');
	});

	test('reset to default restores all values', () => {
		updateTabsSize(20);
		updateTabsSize(13);
		assert.strictEqual(FONT.tabsSize, 13);
		assert.strictEqual(FONT.tabsSize22, 22);
		assert.strictEqual(FONT.tabsSize35, 35);
		assert.strictEqual(FONT.tabsSize38, 38);
		assert.strictEqual(FONT.tabsSize80, 80);
		assert.strictEqual(FONT.tabsSize120, 120);
	});
});

suite('FONT - Cross-area independence', () => {

	ensureNoDisposablesAreLeakedInTestSuite();

	teardown(() => {
		updateSidebarSize(13);
		updateStatusBarSize(12);
		updatePanelSize(13);
		updateActivityBarSize(16);
		updateTabsSize(13);
	});

	test('updating sidebar does not affect any other area (full derived field check)', () => {
		const before = snapshotFont();
		updateSidebarSize(20);

		// All non-sidebar fields must be unchanged
		assertFieldsUnchanged(before, 'statusBar', 'after sidebar update');
		assertFieldsUnchanged(before, 'panel', 'after sidebar update');
		assertFieldsUnchanged(before, 'activityBar', 'after sidebar update');
		assertFieldsUnchanged(before, 'tabs', 'after sidebar update');
	});

	test('updating statusBar does not affect any other area', () => {
		const before = snapshotFont();
		updateStatusBarSize(20);

		assertFieldsUnchanged(before, 'sidebar', 'after statusBar update');
		assertFieldsUnchanged(before, 'panel', 'after statusBar update');
		assertFieldsUnchanged(before, 'activityBar', 'after statusBar update');
		assertFieldsUnchanged(before, 'tabs', 'after statusBar update');
	});

	test('updating panel does not affect any other area', () => {
		const before = snapshotFont();
		updatePanelSize(20);

		assertFieldsUnchanged(before, 'sidebar', 'after panel update');
		assertFieldsUnchanged(before, 'statusBar', 'after panel update');
		assertFieldsUnchanged(before, 'activityBar', 'after panel update');
		assertFieldsUnchanged(before, 'tabs', 'after panel update');
	});

	test('updating activityBar does not affect any other area', () => {
		const before = snapshotFont();
		updateActivityBarSize(20);

		assertFieldsUnchanged(before, 'sidebar', 'after activityBar update');
		assertFieldsUnchanged(before, 'statusBar', 'after activityBar update');
		assertFieldsUnchanged(before, 'panel', 'after activityBar update');
		assertFieldsUnchanged(before, 'tabs', 'after activityBar update');
	});

	test('updating tabs does not affect any other area', () => {
		const before = snapshotFont();
		updateTabsSize(20);

		assertFieldsUnchanged(before, 'sidebar', 'after tabs update');
		assertFieldsUnchanged(before, 'statusBar', 'after tabs update');
		assertFieldsUnchanged(before, 'panel', 'after tabs update');
		assertFieldsUnchanged(before, 'activityBar', 'after tabs update');
	});

	test('all areas set to same value produce different derived values due to different coefficients', () => {
		const commonSize = 18;
		updateSidebarSize(commonSize);
		updateStatusBarSize(commonSize);
		updatePanelSize(commonSize);
		updateActivityBarSize(commonSize);
		updateTabsSize(commonSize);

		assert.strictEqual(FONT.sidebarSize, commonSize);
		assert.strictEqual(FONT.statusBarSize, commonSize);
		assert.strictEqual(FONT.bottomPaneSize, commonSize);
		assert.strictEqual(FONT.activityBarSize, commonSize);
		assert.strictEqual(FONT.tabsSize, commonSize);

		// sidebarSize22 = 18 * (22/13), statusBarSize22 = 18 * (22/12)
		// Different coefficients → different results
		assert.notStrictEqual(FONT.sidebarSize22, FONT.statusBarSize22,
			'same base size should produce different derived values due to different coefficients');
	});

	test('sequential updates across all areas and full reset', () => {
		updateSidebarSize(8);
		updateStatusBarSize(10);
		updatePanelSize(15);
		updateActivityBarSize(20);
		updateTabsSize(25);

		// Verify all set correctly
		assert.strictEqual(FONT.sidebarSize, 8);
		assert.strictEqual(FONT.statusBarSize, 10);
		assert.strictEqual(FONT.bottomPaneSize, 15);
		assert.strictEqual(FONT.activityBarSize, 20);
		assert.strictEqual(FONT.tabsSize, 25);

		// Reset all
		updateSidebarSize(13);
		updateStatusBarSize(12);
		updatePanelSize(13);
		updateActivityBarSize(16);
		updateTabsSize(13);

		// All defaults restored
		assert.strictEqual(FONT.sidebarSize, 13);
		assert.strictEqual(FONT.sidebarSize22, 22);
		assert.strictEqual(FONT.statusBarSize, 12);
		assert.strictEqual(FONT.statusBarSize22, 22);
		assert.strictEqual(FONT.bottomPaneSize, 13);
		assert.strictEqual(FONT.bottomPaneSize22, 22);
		assert.strictEqual(FONT.activityBarSize, 16);
		assert.strictEqual(FONT.activityBarSize48, 48);
		assert.strictEqual(FONT.tabsSize, 13);
		assert.strictEqual(FONT.tabsSize35, 35);
	});
});
