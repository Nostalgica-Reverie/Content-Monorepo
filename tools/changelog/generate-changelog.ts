import { execFileSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';

function runGit(args: string[], cwd?: string): string {
    return execFileSync('git', args, { cwd, encoding: 'utf-8' }).trim();
}

function generateChangelog(manifestPathStr: string): string {
    const manifestPath = path.resolve(manifestPathStr);
    const pDir = path.dirname(manifestPath);

    if (!fs.existsSync(manifestPath)) {
        throw new Error(`manifest not found: ${manifestPathStr}`);
    }

    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf-8'));
    const rawName: string = manifest.name ?? path.basename(pDir);

    const changelogFile = path.join(pDir, 'changelog.md');
    let notes = fs.existsSync(changelogFile)
        ? fs.readFileSync(changelogFile, 'utf-8')
        : `update for ${rawName}`;

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

    const commitLines: string[] = [];
    if (prevHash) {
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
                commitLines.push(`${hash} ${subject} - ${author}`);
            }
        } catch (e) {
            console.warn(`could not fetch git logs for ${pDir}: ${e}`);
        }
    } else {
        console.warn('no prior manifest bump found; skipping automated commit log');
    }

    if (commitLines.length > 0) {
        if (!notes.includes('# Meta-changes')) {
            notes += '\n\n# Meta-changes\n';
        }
        notes += '\n### Automated Commit Log\n';
        notes += commitLines.map(line => `- ${line}`).join('\n') + '\n';
    }

    return notes;
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
    console.log('\nCHANGELOG PREVIEW\n');
    console.log(finalNotes.trim());
    console.log('\nEND\n');
}