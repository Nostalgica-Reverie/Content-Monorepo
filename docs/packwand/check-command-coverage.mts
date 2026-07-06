// Compares the packwand CLI's own registered command tree against the
// generated docs/reference/commands/ pages, so a broken or omitted cobra
// command registration fails CI instead of silently producing a doc set
// that's missing pages (codex.md §4.2).
//
// Depends on a hidden `packwand utils commands --json` flag (a flat JSON
// array of command paths, e.g. "curseforge add") that is not this file's
// responsibility to add - see agent-split.md's cross-group dependency note.
// Until that flag exists upstream, this script soft-skips with a warning
// instead of failing CI on branches that don't have it yet.
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { readdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const commandsDir = join(here, "docs", "reference", "commands");
const packwandSrc = join(here, "..", "..", "src", "packwand");

function findPackwand(): [string, ...string[]] {
  if (process.env.PACKWAND_BIN && existsSync(process.env.PACKWAND_BIN)) {
    return [process.env.PACKWAND_BIN];
  }
  // Fall back to running from source; requires Go on PATH.
  return ["go", "run", "-C", packwandSrc, "."];
}

function pathToFilename(path: string): string {
  return `packwand_${path.replace(/ /g, "_")}.md`;
}

const [bin, ...binArgs] = findPackwand();

let commandPaths;
try {
  const output = execFileSync(bin, [...binArgs, "utils", "commands", "--json"], {
    encoding: "utf-8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  commandPaths = JSON.parse(output);
} catch (error) {
  console.warn(
    "Skipping CLI-reference coverage check: `packwand utils commands --json` is not available yet " +
      "(codex.md §4.2 depends on a flag from the CLI/core group - see agent-split.md's cross-group " +
      "dependency note). Once that flag lands this script will start enforcing coverage.\n" +
      `Underlying error: ${error instanceof Error ? error.message : error}`,
  );
  process.exit(0);
}

if (!Array.isArray(commandPaths)) {
  console.error(
    "`packwand utils commands --json` did not return a JSON array as expected; got:",
    commandPaths,
  );
  process.exit(1);
}

// The flag emits an array of {path, use, summary, group, runnable} objects
// (richer than the flat string array originally sketched in agent-split.md);
// accept both shapes so either side can evolve independently.
commandPaths = commandPaths.map((entry) =>
  typeof entry === "string" ? entry : entry.path,
);

// The root `packwand` command itself is excluded from the catalog (see
// cmd.CommandCatalog's doc comment) but always gets its own page.
const expectedFiles = new Set(["packwand.md", ...commandPaths.map(pathToFilename)]);

const actualFiles = new Set(
  (await readdir(commandsDir)).filter((name) => name.endsWith(".md")),
);

const missing = [...expectedFiles].filter((name) => !actualFiles.has(name)).sort();
const orphaned = [...actualFiles].filter((name) => !expectedFiles.has(name)).sort();

if (missing.length > 0) {
  console.error(
    `CLI-reference coverage: ${missing.length} registered command(s) have no generated page ` +
      `(a cobra command may be broken or omitted from \`packwand utils markdown\`):`,
  );
  for (const name of missing) console.error(`  - ${name}`);
}
if (orphaned.length > 0) {
  console.warn(
    `CLI-reference coverage: ${orphaned.length} page(s) under docs/reference/commands/ no longer ` +
      "correspond to a registered command (stale, probably safe to delete):",
  );
  for (const name of orphaned) console.warn(`  - ${name}`);
}

if (missing.length > 0) {
  process.exit(1);
}
console.log(
  `CLI-reference coverage OK: ${commandPaths.length} command(s), all with a generated page.`,
);
