import * as fs from 'node:fs';

function findSomnus(): string {
    const env = process.env.SOMNUS_BIN;
    if (env && fs.existsSync(env)) return env;
    if (fs.existsSync('./somnus-bin/somnus')) return './somnus-bin/somnus';
    return 'somnus';
}

const args = process.argv.slice(2);
if (args.length === 0) {
    console.error('usage: bun validate.ts <path/to/manifest.json> [more manifests...] | bun validate.ts --all');
    process.exit(1);
}

const proc = Bun.spawnSync([findSomnus(), 'validate', ...args], {
    stdout: 'inherit',
    stderr: 'inherit',
});
process.exit(proc.exitCode ?? 1);
