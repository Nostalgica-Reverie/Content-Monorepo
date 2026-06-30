import { copyFile, cp, mkdir, rm, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const staticDir = join(here, "..", "static");
const buildDir = join(here, "build", "dev", "javascript", "packwand_gui");
const entry = join(buildDir, "packwand_gui.mjs");
const ffi = join(here, "src", "packwand_gui", "ffi.mjs");

const result = spawnSync("gleam", ["build"], {
  cwd: here,
  stdio: "inherit",
  shell: process.platform === "win32",
});

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

if (!existsSync(entry)) {
  console.error(`Gleam build output not found: ${entry}`);
  process.exit(1);
}

await mkdir(staticDir, { recursive: true });
await rm(join(staticDir, "packwand_gui"), { recursive: true, force: true });
await rm(join(staticDir, "gleam_stdlib"), { recursive: true, force: true });
await cp(buildDir, staticDir, { recursive: true });
await mkdir(join(staticDir, "packwand_gui"), { recursive: true });
await copyFile(ffi, join(staticDir, "packwand_gui", "ffi.mjs"));
await writeFile(
  join(staticDir, "app.js"),
  'import { main } from "./packwand_gui.mjs";\nmain();\n',
);

console.log("Copied Gleam frontend into gui/static.");
