import { execFileSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';

function runGit(args: string[], cwd?: string): string {
    return execFileSync('git', args, { cwd, encoding: 'utf-8' }).trim();
}
interface ModInfo {
    name: string;
    version: string;
}

function listModFilesAtRef(ref: string, repoRelDir: string): string[] {
    try {
        const out = runGit(['ls-tree', '-r', '--name-only', ref, '--', repoRelDir]);
        return out
            .split('\n')
            .filter((f) => f.endsWith('.pw.toml'))
            .map((f) => f.trim())
            .filter(Boolean);
    } catch {
        return [];
    }
}

function fileAtRef(ref: string, repoRelPath: string): string {
    try {
        return execFileSync('git', ['show', `${ref}:${repoRelPath}`], { encoding: 'utf-8' });
    } catch {
        return '';
    }
}

function changeSignal(content: string): string {
    const hashMatch = content.match(/hash\s*=\s*"([^"]+)"/);
    if (hashMatch) return hashMatch[1];
    const verMatch = content.match(/version\s*=\s*"([^"]+)"/);
    if (verMatch) return verMatch[1];
    return crypto.createHash('sha1').update(content).digest('hex');
}

function modNameFromPath(p: string): string {
    return path.basename(p).replace(/\.pw\.toml$/, '');
}

interface DiffResult {
    added: string[];
    updated: string[];
    removed: string[];
}

function diffMods(oldRef: string, newRef: string, repoRelDir: string): DiffResult {
    const oldFiles = listModFilesAtRef(oldRef, repoRelDir);
    const newFiles = listModFilesAtRef(newRef, repoRelDir);

    const oldMap = new Map<string, ModInfo>();
    for (const f of oldFiles) {
        oldMap.set(modNameFromPath(f), { name: modNameFromPath(f), version: changeSignal(fileAtRef(oldRef, f)) });
    }
    const newMap = new Map<string, ModInfo>();
    for (const f of newFiles) {
        newMap.set(modNameFromPath(f), { name: modNameFromPath(f), version: changeSignal(fileAtRef(newRef, f)) });
    }

    const added: string[] = [];
    const updated: string[] = [];
    const removed: string[] = [];

    for (const [name, info] of newMap) {
        const old = oldMap.get(name);
        if (!old) added.push(name);
        else if (old.version !== info.version) updated.push(name);
    }
    for (const name of oldMap.keys()) {
        if (!newMap.has(name)) removed.push(name);
    }

    added.sort();
    updated.sort();
    removed.sort();
    return { added, updated, removed };
}

function formatDiff(d: DiffResult): string {
    if (d.added.length === 0 && d.updated.length === 0 && d.removed.length === 0) return '';
    const lines: string[] = [];
    for (const m of d.added) lines.push(`🟢 Added \`${m}\``);
    for (const m of d.updated) lines.push(`🟠 Updated \`${m}\``);
    for (const m of d.removed) lines.push(`🔴 Removed \`${m}\``);
    const summary = `**${d.added.length} added, ${d.updated.length} updated, ${d.removed.length} removed**`;
    return `${summary}\n\n${lines.join('\n')}`;
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
        if (isExperimental) {
            const packLog = runGit(['log', '-n', '2', '--format=%H', '--', pDir]);
            const hashes = packLog.split('\n').filter(Boolean);
            if (hashes.length > 1) prevHash = hashes[1];
        } else {
            const prevBumpLog = runGit(['log', '-n', '2', '--format=%H', '--', manifestPathStr]);
            const hashes = prevBumpLog.split('\n').filter(Boolean);
            if (hashes.length > 1) prevHash = hashes[1];
        }
    } catch (e) {
        console.warn(`could not read git log for anchor: ${e}`);
    }

    function subdirKeys(): string[] {
        if (Array.isArray(manifest.variants)) {
            return manifest.variants.map((v: any) => v.id ?? v.mc_version);
        }
        if (manifest.mc_version) return [manifest.mc_version];
        return [];
    }

    function buildModUpdatesBlock(): string {
        if (!prevHash || manifest.type !== 'modpack') return '';
        const sections: string[] = [];
        for (const key of subdirKeys()) {
            for (const platform of ['mr', 'cf']) {
                const subdir = path.join(pDir, `${key}-${platform}`);
                if (!fs.existsSync(subdir)) continue;
                const repoRel = path.relative(process.cwd(), subdir).split(path.sep).join('/');
                const diff = diffMods(prevHash, 'HEAD', repoRel);
                const formatted = formatDiff(diff);
                if (formatted) {
                    const label = platform === 'mr' ? 'Modrinth' : 'CurseForge';
                    const variantLabel = subdirKeys().length > 1 ? ` (${key})` : '';
                    sections.push(`## ${label}${variantLabel}\n\n${formatted}`);
                }
            }
        }
        if (sections.length === 0) return '';
        return `# Mod Updates\n\n${sections.join('\n\n')}\n`;
    }

    if (!isExperimental) {
        const changelogFile = path.join(pDir, 'changelog.md');
        let notes = fs.existsSync(changelogFile)
            ? fs.readFileSync(changelogFile, 'utf-8')
            : `update for ${rawName}`;

        const modUpdatesBlock = buildModUpdatesBlock();
        if (modUpdatesBlock) {
            if (notes.includes('# Meta-changes')) {
                notes = notes.replace('# Meta-changes', `${modUpdatesBlock}\n# Meta-changes`);
            } else {
                notes = `${notes.trim()}\n\n${modUpdatesBlock}`;
            }
        }

        const commitLines = collectCommitLines(prevHash, pDir);
        if (commitLines.length > 0) {
            if (!notes.includes('# Meta-changes')) notes += '\n\n# Meta-changes\n';
            notes += '\n### Automated Commit Log\n';
            notes += commitLines.map((line) => `- ${line}`).join('\n') + '\n';
        }

        return notes;
    }

    let notes = `_Experimental commit build. Unfinished work for technical users. Here be dragons._\n`;
    const modUpdatesBlock = buildModUpdatesBlock();
    if (modUpdatesBlock) notes += `\n${modUpdatesBlock}`;

    const commitLines = collectCommitLines(prevHash, pDir);
    if (commitLines.length > 0) {
        notes += '\n# Meta-changes\n\n### Automated Commit Log\n';
        notes += commitLines.map((line) => `- ${line}`).join('\n') + '\n';
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
        const logs = runGit(['log', `${prevHash}..HEAD`, '--format=%h%x09%s%x09%an', '--', pDir]);
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
    console.error('usage: tsx generate-changelog.ts <path/to/manifest.json>');
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
