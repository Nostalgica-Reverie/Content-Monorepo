// Regenerates docs/reference/commands/ from the packwand CLI itself
// (packwand utils markdown), then sanitizes machine-specific defaults
// that cobra bakes into the generated flag help.
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
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

const [bin, ...binArgs] = findPackwand();
await rm(commandsDir, { recursive: true, force: true });
await mkdir(commandsDir, { recursive: true });
execFileSync(bin, [...binArgs, "utils", "markdown", "--dir", commandsDir], {
  stdio: "inherit",
});

// Cobra prints the generating machine's user directories as flag defaults.
const sanitizers: [RegExp, string][] = [
  [/\(default "[^"]*[\\/]+packwand[\\/]+cache"\)/g, '(default: your platform cache directory)'],
  [/\(default "[^"]*\.packwand\.toml"\)/g, '(default: .packwand.toml in your platform config directory)'],
];

for (const file of await readdir(commandsDir)) {
  if (!file.endsWith(".md")) continue;
  const path = join(commandsDir, file);
  let text = await readFile(path, "utf-8");
  for (const [pattern, replacement] of sanitizers) {
    text = text.replace(pattern, replacement);
  }
  await writeFile(path, text);
}

console.log("Regenerated and sanitized CLI reference in docs/reference/commands/.");
