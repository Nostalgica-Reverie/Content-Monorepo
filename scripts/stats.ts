import { execFileSync } from 'node:child_process';

interface PackEntry {
    id: string;
    manifest: {
        name?: string;
        modrinth_id?: string;
    };
}

interface MrProject {
    id: string;
    slug: string;
    title: string;
    downloads: number;
    followers: number;
    updated: string;
}

function collectModrinthIds(): Map<string, string> {
    const out = new Map<string, string>();
    const packwand = process.env.PACKWAND_BIN ?? 'packwand';
    let entries: PackEntry[];
    try {
        entries = JSON.parse(execFileSync(packwand, ['packs', 'list', '--json'], { encoding: 'utf-8' })) as PackEntry[];
    } catch (error) {
        throw new Error(`failed to read manifests with ${packwand} packs list --json: ${String(error)}`);
    }
    for (const entry of entries) {
        const modrinthID = entry.manifest.modrinth_id?.trim();
        if (modrinthID) {
            out.set(modrinthID, entry.manifest.name ?? entry.id);
        }
    }
    return out;
}

async function fetchProjects(ids: string[]): Promise<MrProject[]> {
    const url = `https://api.modrinth.com/v2/projects?ids=${encodeURIComponent(JSON.stringify(ids))}`;
    const resp = await fetch(url, { headers: { 'User-Agent': 'lasting-legacy/packwand-stats' } });
    if (!resp.ok) {
        throw new Error(`Modrinth API ${resp.status}: ${await resp.text()}`);
    }
    return (await resp.json()) as MrProject[];
}

const ids = collectModrinthIds();
if (ids.size === 0) {
    console.error('no modrinth_id values found in any manifest — run from the repo root');
    process.exit(2);
}

const projects = await fetchProjects([...ids.keys()]);
projects.sort((a, b) => b.downloads - a.downloads);

const nameW = Math.max(...projects.map((p) => p.title.length), 4);
let totalDl = 0;
let totalFo = 0;
console.log(`${'pack'.padEnd(nameW)}  ${'downloads'.padStart(10)}  ${'followers'.padStart(9)}  updated`);
for (const p of projects) {
    totalDl += p.downloads;
    totalFo += p.followers;
    console.log(
        `${p.title.padEnd(nameW)}  ${p.downloads.toLocaleString('en-US').padStart(10)}  ${p.followers
            .toLocaleString('en-US')
            .padStart(9)}  ${p.updated.slice(0, 10)}`
    );
}
console.log('-'.repeat(nameW + 34));
console.log(`${'total'.padEnd(nameW)}  ${totalDl.toLocaleString('en-US').padStart(10)}  ${totalFo.toLocaleString('en-US').padStart(9)}`);

const missing = [...ids.keys()].filter((id) => !projects.some((p) => p.id === id || p.slug === id));
if (missing.length > 0) {
    console.warn(`\n::warning::${missing.length} id(s) not returned by Modrinth: ${missing.join(', ')}`);
}
