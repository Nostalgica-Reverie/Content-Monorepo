import { execFileSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';

function runGit(args: string[], cwd?: string): string {
    return execFileSync('git', args, { cwd, encoding: 'utf-8' }).trim();
}

function runModDiff(oldRef: string, newRef: string, pathPrefix: string): string {
    const binPath = process.env.MOD_DIFF_BIN;
    if (!binPath || !fs.existsSync(binPath)) {
        return '';
    }
    try {
        const out = execFileSync(binPath, [oldRef, newRef, pathPrefix], {
            encoding: 'utf-8',
        });
        return out.trim();
    } catch (e) {
        console.warn(`mod-diff failed for ${pathPrefix}: ${e instanceof Error ? e.message : e}`);
        return '';
    }
}

function generateChangelog(manifestPathStr: string): string {
    const manifestPath = path.resolve(manifestPathStr);
    const pDir = path.dirname(manifestPath);
    const filename = path.basename(manifestPath);
    const isExperimental = filename === 'manifest-experimental.json';

    if (!fs.existsSync(manifestPath)) {
        throw new Error(`manifest not found: ${manifestPathStr}`);
    }

    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf-8'));
    const rawName: string = manifest.name ?? path.basename(pDir);

    let prevHash: string | null = null;
    try {
        const prevBumpLog = runGit(['log', '-n', '2', '--format=%H', '--', manifestPathStr]);
        const hashes = prevBumpLog.split('\n').filter(Boolean);
        if (hashes.length > 1) {
            prevHash = hashes[1];
        }
    } catch (e) {
        console.warn(`could not read git log for manifest: ${e}`);
    }

    if (!isExperimental) {
        const changelogFile = path.join(pDir, 'changelog.md');
        let notes = fs.existsSync(changelogFile)
            ? fs.readFileSync(changelogFile, 'utf-8')
            : `update for ${rawName}`;

        const modUpdatesSections: string[] = [];
        if (prevHash && manifest.type === 'modpack' && manifest.mc_version) {
            for (const platform of ['mr', 'cf']) {
                const subdir = path.join(pDir, `${manifest.mc_version}-${platform}`);
                if (!fs.existsSync(subdir)) continue;
                const repoRel = path
                    .relative(process.cwd(), subdir)
                    .split(path.sep)
                    .join('/');
                const md = runModDiff(prevHash, 'HEAD', repoRel);
                if (md) {
                    const label = platform === 'mr' ? 'Modrinth' : 'CurseForge';
                    modUpdatesSections.push(`## ${label}\n\n${md}`);
                }
            }
        }

        if (modUpdatesSections.length > 0) {
            const modUpdatesBlock = `# Mod Updates\n\n${modUpdatesSections.join('\n\n')}\n`;
            if (notes.includes('# Meta-changes')) {
                notes = notes.replace('# Meta-changes', `${modUpdatesBlock}\n# Meta-changes`);
            } else {
                notes = `${notes.trim()}\n\n${modUpdatesBlock}`;
            }
        }

        const commitLines = collectCommitLines(prevHash, pDir);
        if (commitLines.length > 0) {
            if (!notes.includes('# Meta-changes')) {
                notes += '\n\n# Meta-changes\n';
            }
            notes += '\n### Automated Commit Log\n';
            notes += commitLines.map(line => `- ${line}`).join('\n') + '\n';
        }

        return notes;
    }

    let notes = `_Experimental nightly build. Unfinished work for technical users._\n`;

    const commitLines = collectCommitLines(prevHash, pDir);
    if (commitLines.length > 0) {
        notes += '\n# Meta-changes\n\n### Automated Commit Log\n';
        notes += commitLines.map(line => `- ${line}`).join('\n') + '\n';
    } else {
        notes += '\n_No commits to report since last experimental build._\n';
    }
    return notes;
}

function collectCommitLines(prevHash: string | null, pDir: string): string[] {
    if (!prevHash) {
        console.warn('no prior manifest bump found; skipping automated commit log');
        return [];
    }
    const out: string[] = [];
    try {
        const logs = runGit([
            'log',
            `${prevHash}..HEAD`,
            '--format=%h%x09%s%x09%an',
            '--',
            pDir,
        ]);
        for (const line of logs.split('\n')) {
            const parts = line.split('\t');
            if (parts.length !== 3) continue;
            const [hash, subject, author] = parts;
            if (!subject.includes(': ')) continue;
            out.push(`${hash} ${subject} - ${author}`);
        }
    } catch (e) {
        console.warn(`could not fetch git logs for ${pDir}: ${e}`);
    }
    return out;
}

const args = process.argv.slice(2);
if (args.length === 0) {
    console.error('usage: tsx changelog.ts <path/to/manifest.json>');
    process.exit(1);
}

let finalNotes: string;
try {
    finalNotes = generateChangelog(args[0]);
} catch (e) {
    console.error(`${e instanceof Error ? e.message : e}`);
    process.exit(1);
}

const outPath = process.env.GITHUB_OUTPUT;
if (outPath) {
    const delimiter = `EOF_${crypto.randomBytes(8).toString('hex')}`;
    fs.appendFileSync(outPath, `notes<<${delimiter}\n${finalNotes.trim()}\n${delimiter}\n`);
    console.log(`wrote changelog for ${args[0]} to GITHUB_OUTPUT`);
} else {
    console.log('\n--- CHANGELOG PREVIEW ---\n');
    console.log(finalNotes.trim());
    console.log('\n--- END ---\n');
}
