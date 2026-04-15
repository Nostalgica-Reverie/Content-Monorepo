import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

function runGit(args: string[], cwd?: string): string {
    return execSync(`git ${args.join(' ')}`, { cwd, encoding: 'utf-8' }).trim();
}

function generateChangelog(manifestPathStr: string): string {
    const manifestPath = path.resolve(manifestPathStr);
    const pDir = path.dirname(manifestPath);

    if (!fs.existsSync(manifestPath)) {
        console.error(`Manifest not found at: ${manifestPathStr}`);
        process.exit(1);
    }

    const manifestContent = fs.readFileSync(manifestPath, 'utf-8');
    const manifest = JSON.parse(manifestContent);
    const rawName = manifest.name;

    const changelogFile = path.join(pDir, 'changelog.md');
    let notes = fs.existsSync(changelogFile)
        ? fs.readFileSync(changelogFile, 'utf-8')
        : `update for ${rawName}`;

    let prevHash = 'HEAD~1';
    try {
        const prevBumpLog = runGit(['log', '-n', '2', '--format=%H', '--', manifestPathStr]);
        const lines = prevBumpLog.split('\n');
        if (lines.length > 1) {
            prevHash = lines[1];
        }
    } catch (error) {
        console.warn('could not find previous manifest bump, defaulting to HEAD~1');
    }

    let commitLines: string[] = [];
    try {
        const logs = runGit(['log', `${prevHash}..HEAD`, '--format=%h %s - %an', '--', pDir]);
        commitLines = logs.split('\n').filter(line => line.includes(': '));
    } catch (error) {
        console.warn(`could not fetch git logs for ${pDir}`);
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
    console.error('Usage: tsx generate-changelog.ts <path/to/manifest.json>');
    process.exit(1);
}

const targetManifest = args[0];
const finalNotes = generateChangelog(targetManifest);

const outPath = process.env.GITHUB_OUTPUT;
if (outPath) {
    const delimiter = 'EOF_NOTES_DELIMITER';
    fs.appendFileSync(outPath, `notes<<${delimiter}\n${finalNotes.trim()}\n${delimiter}\n`);
    console.log(`successfully wrote changelog for ${targetManifest} to GITHUB_OUTPUT`);
} else {
    console.log('\nCHANGELOG PREVIEW\n');
    console.log(finalNotes.trim());
    console.log('\n\n');
}