import { IConfigurationService } from '../../platform/configuration/common/configuration.js';

export const FONT = {
	activityBarSize: 16,
	activityBarSize16: 16,
	activityBarSize24: 24,
	activityBarSize32: 32,
	activityBarSize36: 36,
	activityBarSize48: 48,

	bottomPaneSize: 13,
	bottomPaneSize22: 22,

	defaultSize: 13,
	defaultActivityBarSize: 16,
	defaultBottomPaneSize: 13,
	defaultSidebarSize: 13,
	defaultStatusBarSize: 12,
	defaultTabsSize: 13,

	sidebarSize: 13,
	sidebarSize8: 8,
	sidebarSize10: 10,
	sidebarSize16: 16,
	sidebarSize17: 17,
	sidebarSize18: 18,
	sidebarSize20: 20,
	sidebarSize22: 22,
	sidebarSize23: 23,
	sidebarSize24: 24,
	sidebarSize26: 26,
	sidebarSize28: 28,
	sidebarSize34: 34,
	sidebarSize39: 39,
	sidebarSize44: 44,
	sidebarSize54: 54,
	sidebarSize62: 62,
	sidebarSize72: 72,

	statusBarSize: 12,
	statusBarSize22: 22,

	tabsSize: 13,
	tabsSize22: 22,
	tabsSize35: 35,
	tabsSize38: 38,
	tabsSize80: 80,
	tabsSize120: 120,

	workbenchCoefficient: 1,
};

// Activity bar coefficients (base 16)
const ACTIVITY_BAR_COEFF_24 = 24/16;
const ACTIVITY_BAR_COEFF_32 = 32/16;
const ACTIVITY_BAR_COEFF_36 = 36/16;
const ACTIVITY_BAR_COEFF_48 = 48/16;

// Panel coefficients (base 13)
const BOTTOM_PANEL_COEFF_22 = 22/13;

// Workbench coefficients (base 13)
const DEFAULT_COEFF_12 = 12/13;
const DEFAULT_COEFF_16 = 16/13;

// Sidebar coefficients (base 13)
const SIDE_BAR_COEFF_8 = 8/13;
const SIDE_BAR_COEFF_10 = 10/13;
const SIDE_BAR_COEFF_16 = 16/13;
const SIDE_BAR_COEFF_17 = 17/13;
const SIDE_BAR_COEFF_18 = 18/13;
const SIDE_BAR_COEFF_20 = 20/13;
const SIDE_BAR_COEFF_22 = 22/13;
const SIDE_BAR_COEFF_23 = 23/13;
const SIDE_BAR_COEFF_24 = 24/13;
const SIDE_BAR_COEFF_26 = 26/13;
const SIDE_BAR_COEFF_28 = 28/13;
const SIDE_BAR_COEFF_34 = 34/13;
const SIDE_BAR_COEFF_39 = 39/13;
const SIDE_BAR_COEFF_44 = 44/13;
const SIDE_BAR_COEFF_54 = 54/13;
const SIDE_BAR_COEFF_62 = 62/13;
const SIDE_BAR_COEFF_72 = 72/13;

// Status bar coefficients (base 12)
const STATUS_BAR_COEFF_22 = 22/12;

// Tabs coefficients (base 13)
const TABS_COEFF_22 = 22/13;
const TABS_COEFF_35 = 35/13;
const TABS_COEFF_38 = 38/13;
const TABS_COEFF_80 = 80/13;
const TABS_COEFF_120 = 120/13;

/**
 * Inspect a configuration value and return whether it was explicitly set by the user,
 * along with the clamped numeric value.
 */
export function inspectFontSize(
	configurationService:IConfigurationService,
	key: string,
	defaultSize: number,
	min: number = 6,
	max: number = 32
): { isUserSet: boolean; size: number } {
	const inspected = configurationService.inspect<number>(key);
	const isUserSet = inspected.userValue !== undefined
		|| inspected.userLocalValue !== undefined
		|| inspected.userRemoteValue !== undefined
		|| inspected.workspaceValue !== undefined
		|| inspected.workspaceFolderValue !== undefined;

	const raw = configurationService.getValue<number>(key);
	const size = Math.max(min, Math.min(max, typeof raw === 'number' ? raw : defaultSize));

	return { isUserSet, size };
}

export function getFontSize(configurationService: IConfigurationService, key: string, defaultSize: number, min: number = 6,	max: number = 32): number {
	const inspected = configurationService.inspect<number>(key);
	const isUserSet = inspected.userValue !== undefined	|| inspected.userLocalValue !== undefined || inspected.userRemoteValue !== undefined || inspected.workspaceValue !== undefined || inspected.workspaceFolderValue !== undefined;

	if(isUserSet) {
		const raw = configurationService.getValue<number>(key);
		const size = Math.max(min, Math.min(max, typeof raw === 'number' ? raw : defaultSize));

		return size;
	}
	else {
		return defaultSize
	}
}

export function updateActivityBarSize(size: number): void {
	FONT.activityBarSize = size;
	FONT.activityBarSize24 = size * ACTIVITY_BAR_COEFF_24;
	FONT.activityBarSize32 = size * ACTIVITY_BAR_COEFF_32;
	FONT.activityBarSize36 = size * ACTIVITY_BAR_COEFF_36;
	FONT.activityBarSize48 = size * ACTIVITY_BAR_COEFF_48;
}

export function updateDefaultSize(size: number): void {
	FONT.defaultSize = size;
	FONT.defaultActivityBarSize = size * DEFAULT_COEFF_16
	FONT.defaultBottomPaneSize = size
	FONT.defaultSidebarSize = size
	FONT.defaultStatusBarSize = size * DEFAULT_COEFF_12
	FONT.defaultTabsSize = size

	FONT.workbenchCoefficient = size / 13
}

export function updatePanelSize(size: number): void {
	FONT.bottomPaneSize = size;
	FONT.bottomPaneSize22 = size * BOTTOM_PANEL_COEFF_22;
}

export function updateSidebarSize(size: number): void {
	FONT.sidebarSize = size;
	FONT.sidebarSize8 = size * SIDE_BAR_COEFF_8;
	FONT.sidebarSize10 = size * SIDE_BAR_COEFF_10;
	FONT.sidebarSize16 = size * SIDE_BAR_COEFF_16;
	FONT.sidebarSize17 = size * SIDE_BAR_COEFF_17;
	FONT.sidebarSize18 = size * SIDE_BAR_COEFF_18;
	FONT.sidebarSize20 = size * SIDE_BAR_COEFF_20;
	FONT.sidebarSize22 = size * SIDE_BAR_COEFF_22;
	FONT.sidebarSize23 = size * SIDE_BAR_COEFF_23;
	FONT.sidebarSize24 = size * SIDE_BAR_COEFF_24;
	FONT.sidebarSize26 = size * SIDE_BAR_COEFF_26;
	FONT.sidebarSize28 = size * SIDE_BAR_COEFF_28;
	FONT.sidebarSize34 = size * SIDE_BAR_COEFF_34;
	FONT.sidebarSize39 = size * SIDE_BAR_COEFF_39;
	FONT.sidebarSize44 = size * SIDE_BAR_COEFF_44;
	FONT.sidebarSize54 = size * SIDE_BAR_COEFF_54;
	FONT.sidebarSize62 = size * SIDE_BAR_COEFF_62;
	FONT.sidebarSize72 = size * SIDE_BAR_COEFF_72;
}

export function updateStatusBarSize(size: number): void {
	FONT.statusBarSize = size;
	FONT.statusBarSize22 = size * STATUS_BAR_COEFF_22;
}

export function updateTabsSize(size: number): void {
	FONT.tabsSize = size;
	FONT.tabsSize22 = size * TABS_COEFF_22;
	FONT.tabsSize35 = size * TABS_COEFF_35;
	FONT.tabsSize38 = size * TABS_COEFF_38;
	FONT.tabsSize80 = size * TABS_COEFF_80;
	FONT.tabsSize120 = size * TABS_COEFF_120;
}