#!/usr/bin/env bun
/**
 * Single entry point for the scripts in this directory.
 *
 *   bun run.ts <script-name> [args...]
 *   bun run.ts --list
 *
 * Callers name a script rather than a path, so moving or renaming a file does
 * not break every workflow and justfile recipe that invokes it. Exit codes are
 * forwarded unchanged so CI still fails when a script fails.
 */
import * as fs from 'node:fs';
import * as path from 'node:path';

const SCRIPT_DIR = import.meta.dir;

// Reference material vendored from other projects, not part of this toolchain.
const EXCLUDED_DIRS = new Set(['node_modules', 'mr-tooling']);

function availableScripts(): string[] {
    return fs
        .readdirSync(SCRIPT_DIR, { withFileTypes: true })
        .filter((entry) => entry.isFile())
        .map((entry) => entry.name)
        .filter((name) => name.endsWith('.ts'))
        .filter((name) => name !== 'run.ts' && !name.endsWith('.test.ts'))
        .map((name) => name.slice(0, -'.ts'.length))
        .sort();
}

function usage(problem?: string): never {
    if (problem) console.error(`error: ${problem}\n`);
    console.error('usage: bun run.ts <script-name> [args...]\n');
    console.error('available scripts:');
    for (const name of availableScripts()) console.error(`  ${name}`);
    process.exit(problem ? 1 : 0);
}

const [name, ...args] = process.argv.slice(2);
if (!name || name === '--help' || name === '-h') usage();
if (name === '--list') {
    for (const script of availableScripts()) console.log(script);
    process.exit(0);
}

// Reject anything that would escape the scripts directory, so a caller cannot
// turn `bun scripts ...` into "run an arbitrary file".
if (name.includes('/') || name.includes('\\') || name.includes('..')) {
    usage(`invalid script name ${JSON.stringify(name)}`);
}

const scriptPath = path.join(SCRIPT_DIR, `${name}.ts`);
if (!fs.existsSync(scriptPath)) usage(`unknown script ${JSON.stringify(name)}`);

const proc = Bun.spawnSync(['bun', scriptPath, ...args], {
    stdout: 'inherit',
    stderr: 'inherit',
    stdin: 'inherit',
});
process.exit(proc.exitCode ?? 1);
