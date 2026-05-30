import { execFileSync } from 'child_process';
import * as fs from 'fs';
import { parse as parseToml } from 'smol-toml';

function fail(file: string, msg: string): void {
    console.error(`::error file=${file}::${msg}`);
}

function changedFiles(): string[] {
    try {
        const out = execFileSync(
            'git',
            ['diff-tree', '--no-commit-id', '--name-only', '-r', 'HEAD'],
            { encoding: 'utf-8' },
        ).trim();
        return out.split('\n').map((l) => l.trim()).filter(Boolean);
    } catch (e) {
        console.warn(`::warning::could not read git diff-tree: ${e instanceof Error ? e.message : e}`);
        return [];
    }
}

type Kind = 'json' | 'toml' | null;

function kindOf(file: string): Kind {
    if (file.endsWith('.json')) return 'json';
    if (file.endsWith('.toml')) return 'toml';
    return null;
}

function lintFile(file: string): boolean {
    if (!fs.existsSync(file)) {
        return true;
    }
    const kind = kindOf(file);
    if (!kind) return true;

    let content: string;
    try {
        content = fs.readFileSync(file, 'utf-8');
    } catch (e) {
        fail(file, `could not read file: ${e instanceof Error ? e.message : e}`);
        return false;
    }

    try {
        if (kind === 'json') {
            JSON.parse(content);
        } else {
            parseToml(content);
        }
        return true;
    } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        fail(file, `INVALID ${kind.toUpperCase()}: ${msg}`);
        return false;
    }
}

function main(): void {
    const args = process.argv.slice(2);
    const files = args.length > 0 ? args : changedFiles();

    const lintable = files.filter((f) => kindOf(f) !== null);

    if (lintable.length === 0) {
        console.log('no JSON/TOML files to lint.');
        return;
    }

    console.log(`linting ${lintable.length} file(s)...`);

    let checked = 0;
    let failed = 0;
    for (const file of lintable) {
        console.log(`::group::Linting ${file}`);
        checked++;
        if (!lintFile(file)) failed++;
        console.log('::endgroup::');
    }

    if (failed > 0) {
        console.error(`::error::${failed} of ${checked} file(s) failed syntax linting`);
        process.exit(1);
    }
    console.log(`✓ all ${checked} file(s) parsed OK`);
}

main();
