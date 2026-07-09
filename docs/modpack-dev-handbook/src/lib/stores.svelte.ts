// This file handles all of the stores in the app. Stores are used to share data between components.

/**
 * This stores global window information.
 */
export const windowInfo = $state({
  width: 1920,
  isNavOpen: true,
});

/**
 * This stores the latest Minecraft data. It contains default values. The data is updated automatically on the server.
 */
export let latestMCData = $state({
  packFormat: 57,
  gameVersion: "1.21.1",
  changed: false,
});

