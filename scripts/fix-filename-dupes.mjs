// Fold a same-filename duplicate pair (one modrinth-only sidecar, one
// curseforge-only sidecar, same actual distributed file) into a single
// dual-provider sidecar, deleting the redundant file. Only acts on pairs
// where exactly one side is modrinth-only and the other curseforge-only —
// anything else is left alone and reported for manual review.
import { readdirSync, readFileSync, writeFileSync, unlinkSync } from "fs";
import { join } from "path";

const [, , packDir, ...rest] = process.argv;
const apply = rest.includes("--apply");
const CONTENT_FOLDERS = ["mods", "resourcepacks", "shaderpacks"];

function load(dir) {
	return CONTENT_FOLDERS.flatMap((folder) => {
		const full = join(dir, folder);
		try {
			return readdirSync(full)
				.filter((f) => f.endsWith(".bun.json"))
				.map((f) => ({ folder, file: f, path: join(full, f), data: JSON.parse(readFileSync(join(full, f), "utf8")) }));
		} catch {
			return [];
		}
	});
}

const entries = load(packDir);
const byFilename = new Map();
for (const e of entries) {
	if (!e.data.filename) continue;
	if (!byFilename.has(e.data.filename)) byFilename.set(e.data.filename, []);
	byFilename.get(e.data.filename).push(e);
}

for (const [filename, group] of byFilename) {
	if (group.length !== 2) continue;
	const [a, b] = group;
	const aProv = Object.keys(a.data.downloads || {});
	const bProv = Object.keys(b.data.downloads || {});
	const isMr = (p) => p.length === 1 && p[0] === "modrinth";
	const isCf = (p) => p.length === 1 && p[0] === "curseforge";
	let mr, cf;
	if (isMr(aProv) && isCf(bProv)) [mr, cf] = [a, b];
	else if (isMr(bProv) && isCf(aProv)) [mr, cf] = [b, a];
	else {
		console.log(`SKIP (not a clean mr/cf pair) "${filename}": ${a.path} [${aProv}] vs ${b.path} [${bProv}]`);
		continue;
	}
	console.log(`MERGE "${filename}": keep ${mr.path}, fold in ${cf.path}, delete ${cf.path}`);
	if (apply) {
		const merged = structuredClone(mr.data);
		merged.downloads = merged.downloads || {};
		merged.downloads.curseforge = cf.data.downloads.curseforge;
		merged.update = merged.update || {};
		if (cf.data.update?.curseforge) merged.update.curseforge = cf.data.update.curseforge;
		writeFileSync(mr.path, JSON.stringify(merged, null, 2) + "\n");
		unlinkSync(cf.path);
	}
}
if (!apply) console.log("\n(dry run — pass --apply to write changes)");
