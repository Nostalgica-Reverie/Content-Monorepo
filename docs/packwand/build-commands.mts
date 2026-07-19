// Regenerates handbook-owned command pages from the packwand CLI itself
// (packwand utils markdown), then sanitizes machine-specific defaults
// that cobra bakes into generated flag help.
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import {
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
  "..",
  "modpack-dev-handbook",
  "src",
  "routes",
  "wiki",
  "modpack-management",
  "packwand",
  "reference",
  "commands",
);
const packwandSrc = join(here, "..", "..", "apps", "packwand");

function findPackwand(): [string, ...string[]] {
  if (process.env.PACKWAND_BIN && existsSync(process.env.PACKWAND_BIN)) {
    return [process.env.PACKWAND_BIN];
  }
  return ["go", "run", "-C", packwandSrc, "."];
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
  console.log(
    "Regenerated and sanitized CLI reference in the handbook route tree.",
  );
} finally {
  if (!committed) {
    await rm(temporaryCommandsDir, { recursive: true, force: true });
  }
}
