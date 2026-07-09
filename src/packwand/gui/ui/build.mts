import { copyFile, cp, mkdir, readdir, rename, rm, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { basename, dirname, extname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const staticDir = join(here, "..", "static");
const javascriptDir = join(here, "build", "dev", "javascript");
const buildDir = join(javascriptDir, "packwand_gui");
const entry = join(buildDir, "packwand_gui.mjs");
const ffi = join(here, "src", "packwand_gui", "ffi.mjs");

const winGetGleam = join(
  process.env.LOCALAPPDATA || "",
  "Microsoft",
  "WinGet",
  "Packages",
  "Gleam.Gleam_Microsoft.Winget.Source_8wekyb3d8bbwe",
  "gleam.exe",
);
const gleam = process.env.GLEAM_BIN
  || (process.platform === "win32" && existsSync(winGetGleam) ? winGetGleam : "gleam");

// Gleam requires symlink privileges for package priv directories on Windows.
// Lustre's priv files are server-component bundles and are unused by this SPA.
const lustrePriv = join(here, "build", "packages", "lustre", "priv");
const parkedPriv = `${lustrePriv}.packwand-build`;
let privParked = false;
if (process.platform === "win32") {
  if (!existsSync(lustrePriv) && existsSync(parkedPriv)) {
    await rename(parkedPriv, lustrePriv);
  }
  if (existsSync(lustrePriv)) {
    await rm(parkedPriv, { recursive: true, force: true });
    await rename(lustrePriv, parkedPriv);
    privParked = true;
  }
}

let result: ReturnType<typeof spawnSync>;
try {
  result = spawnSync(gleam, ["build"], {
    cwd: here,
    stdio: "inherit",
    shell: false,
  });
} finally {
  if (privParked && existsSync(parkedPriv)) {
    await rename(parkedPriv, lustrePriv);
  }
}

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

if (!existsSync(entry)) {
  console.error(`Gleam build output not found: ${entry}`);
  process.exit(1);
}

await mkdir(staticDir, { recursive: true });
await rm(join(staticDir, "packwand_gui"), { recursive: true, force: true });
await rm(join(staticDir, "_gleam_artefacts"), { recursive: true, force: true });
await rm(join(staticDir, "gleam.mjs"), { force: true });
await rm(join(staticDir, "packwand_gui.mjs"), { force: true });
const packageDir = join(staticDir, "packwand_gui");
await cp(buildDir, packageDir, { recursive: true, filter: browserArtifact });

const packages = await readdir(javascriptDir, { withFileTypes: true });
for (const entry of packages) {
  if (!entry.isDirectory() || entry.name === "packwand_gui") continue;
  const destination = join(staticDir, entry.name);
  await rm(destination, { recursive: true, force: true });
  await cp(join(javascriptDir, entry.name), destination, {
    recursive: true,
    filter: browserArtifact,
  });
}
await copyFile(join(javascriptDir, "prelude.mjs"), join(staticDir, "prelude.mjs"));

await mkdir(join(packageDir, "packwand_gui"), { recursive: true });
await copyFile(ffi, join(packageDir, "packwand_gui", "ffi.mjs"));
await writeFile(
  join(staticDir, "app.js"),
  'import { main } from "./packwand_gui/packwand_gui.mjs";\nmain();\n',
);

console.log("Copied Gleam frontend into gui/static.");

function browserArtifact(source: string): boolean {
  const name = basename(source);
  if (name.startsWith("_") || name === "fingerprint") return false;
  const extension = extname(name);
  return extension === "" || extension === ".mjs";
}
