import type { LayoutServerLoad } from "./$types";

export const load = (async () => ({
  packFormat: 57,
  gameVersion: "1.21.1",
})) satisfies LayoutServerLoad;

