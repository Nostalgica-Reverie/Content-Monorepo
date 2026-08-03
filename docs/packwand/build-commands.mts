// Regenerates handbook-owned command pages from the packwand CLI itself
// (packwand utils markdown), then sanitizes machine-specific defaults
// that cobra bakes into generated flag help.
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const commandsDir = join(
  here,
  "docs",
  "reference",
  "commands",
);
const packwandSrc = join(here, "..", "..", "apps", "packwandrs");

function findPackwand(): [string, ...string[]] {
  if (process.env.PACKWAND_BIN && existsSync(process.env.PACKWAND_BIN)) {
    return [process.env.PACKWAND_BIN];
  }
  return ["cargo", "run", "--manifest-path", join(packwandSrc, "Cargo.toml"), "-p", "packwand-cli", "--"];
}

// The generated pages are a pure function of apps/packwandrs's source, so when
// that tree is committed-clean and unchanged since the last run, regeneration
// is a no-op and can be skipped. This matters because `just docs-build` runs
// this script twice (docs/packwand's docs:build and the handbook's build);
// the second invocation then skips. Only used for the `go run` path — an
// explicit PACKWAND_BIN always regenerates.
const markerPath = join(
  here,
  "..",
  "..",
  "node_modules",
  ".cache",
  "packwand-commands-tree",
);

function packwandTreeState(): string | null {
  try {
    const dirty = execFileSync(
      "git",
      ["status", "--porcelain", "--", packwandSrc],
      { encoding: "utf-8", cwd: here },
    ).trim();
    if (dirty !== "") return null;
    return execFileSync("git", ["rev-parse", "HEAD:apps/packwandrs"], {
      encoding: "utf-8",
      cwd: here,
    }).trim();
  } catch {
    return null;
  }
}

const treeState = process.env.PACKWAND_BIN ? null : packwandTreeState();
if (
  treeState &&
  existsSync(commandsDir) &&
  existsSync(markerPath) &&
  (await readFile(markerPath, "utf-8")).trim() === treeState
) {
  console.log(
    "CLI reference up to date (apps/packwandrs unchanged since last run); skipping regeneration.",
  );
  process.exit(0);
}

const [bin, ...binArgs] = findPackwand();
const commandsParent = dirname(commandsDir);
const temporaryCommandsDir = await mkdtemp(
  join(commandsParent, ".packwand-commands-"),
);
let committed = false;

try {
  execFileSync(
    bin,
    [...binArgs, "utils", "markdown", "--dir", temporaryCommandsDir],
    {
      stdio: "inherit",
    },
  );

  const sanitizers: [RegExp, string][] = [
    [
      /\(default "[^"]*[\\/]+packwand[\\/]+cache"\)/g,
      "(default: your platform cache directory)",
    ],
    [
      /\(default "[^"]*\.packwand\.toml"\)/g,
      "(default: .packwand.toml in your platform config directory)",
    ],
  ];

  for (const file of await readdir(temporaryCommandsDir)) {
    if (!file.endsWith(".md")) continue;
    const filePath = join(temporaryCommandsDir, file);
    let text = await readFile(filePath, "utf-8");
    for (const [pattern, replacement] of sanitizers) {
      text = text.replace(pattern, replacement);
    }
    await writeFile(filePath, text);
  }

  await rm(commandsDir, { recursive: true, force: true });
  await rename(temporaryCommandsDir, commandsDir);
  committed = true;
  if (treeState) {
    await mkdir(dirname(markerPath), { recursive: true });
    await writeFile(markerPath, treeState + "\n");
  }
  console.log(
    "Regenerated and sanitized the Packwand CLI reference.",
  );
} finally {
  if (!committed) {
    await rm(temporaryCommandsDir, { recursive: true, force: true });
  }
}
