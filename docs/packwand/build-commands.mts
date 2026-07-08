// Regenerates handbook-owned command pages from the packwand CLI itself
// (packwand utils markdown), then sanitizes machine-specific defaults
// that cobra bakes into generated flag help.
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const commandsDir = join(here, "..", "modpack-dev-handbook", "src", "routes", "wiki", "modpack-management", "packwand", "reference", "commands");
const packwandSrc = join(here, "..", "..", "src", "packwand");

function findPackwand(): [string, ...string[]] {
  if (process.env.PACKWAND_BIN && existsSync(process.env.PACKWAND_BIN)) {
    return [process.env.PACKWAND_BIN];
  }
  return ["go", "run", "-C", packwandSrc, "."];
}

const [bin, ...binArgs] = findPackwand();
await rm(commandsDir, { recursive: true, force: true });
await mkdir(commandsDir, { recursive: true });
execFileSync(bin, [...binArgs, "utils", "markdown", "--dir", commandsDir], {
  stdio: "inherit",
});

const sanitizers: [RegExp, string][] = [
  [/\(default "[^"]*[\/]+packwand[\/]+cache"\)/g, '(default: your platform cache directory)'],
  [/\(default "[^"]*\.packwand\.toml"\)/g, '(default: .packwand.toml in your platform config directory)'],
];

for (const file of await readdir(commandsDir)) {
  if (!file.endsWith(".md")) continue;
  const filePath = join(commandsDir, file);
  let text = await readFile(filePath, "utf-8");
  for (const [pattern, replacement] of sanitizers) {
    text = text.replace(pattern, replacement);
  }
  await writeFile(filePath, text);
}

console.log("Regenerated and sanitized CLI reference in the handbook route tree.");
