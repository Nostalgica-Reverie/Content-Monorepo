// Pilot script: merge a pack's -mr and -cf subdirectories into one directory,
// matching mods by their raw "name" field (not filename), per omo50's
// correction. Matched pairs get one sidecar with both providers' downloads;
// unmatched mods are copied through as single-provider entries; matched pairs
// whose file hashes disagree are NOT merged (flagged for manual review
// instead, since bundle's schema assumes a shared filename across providers).
//
// Usage: bun scripts/merge-pack-variants.mjs <pack-dir> <mr-subdir> <cf-subdir> <out-subdir> [--apply]
// Without --apply, only prints the plan (dry run).

import { readdirSync, readFileSync, writeFileSync, mkdirSync, existsSync, cpSync, rmSync } from "fs";
import { join, basename } from "path";

const [, , packDir, mrName, cfName, outName, ...rest] = process.argv;
const apply = rest.includes("--apply");
if (!packDir || !mrName || !cfName || !outName) {
	console.error("usage: merge-pack-variants.mjs <pack-dir> <mr-subdir> <cf-subdir> <out-subdir> [--apply]");
	process.exit(2);
}

const mrDir = join(packDir, mrName);
const cfDir = join(packDir, cfName);
const outDir = join(packDir, outName);

function normalize(name) {
	return name
		.toLowerCase()
		.replace(/\([^)]*\)/g, "") // parenthetical qualifiers: "(Fabric)", "(YACL)"
		.replace(/\b(fabric|forge|neoforge|quilt)\b/g, "") // bare loader-tag words
		.replace(/[^a-z0-9]/g, ""); // punctuation/slashes/spaces
}

function loadSidecars(dir, subfolder) {
	const folder = join(dir, subfolder);
	if (!existsSync(folder)) return [];
	return readdirSync(folder)
		.filter((f) => f.endsWith(".bun.json"))
		.map((f) => {
			const path = join(folder, f);
			const data = JSON.parse(readFileSync(path, "utf8"));
			return { file: f, subfolder, path, data };
		});
}

const CONTENT_FOLDERS = ["mods", "resourcepacks", "shaderpacks"];

function loadAll(dir) {
	return CONTENT_FOLDERS.flatMap((folder) => loadSidecars(dir, folder));
}

const mrEntries = loadAll(mrDir);
const cfEntries = loadAll(cfDir);

const mrByNorm = new Map();
for (const e of mrEntries) {
	const key = normalize(e.data.name || "");
	if (!mrByNorm.has(key)) mrByNorm.set(key, []);
	mrByNorm.get(key).push(e);
}

const matched = [];
const mrOnly = [];
const cfOnly = [];
const ambiguous = [];
const usedMr = new Set();

for (const cfEntry of cfEntries) {
	const key = normalize(cfEntry.data.name || "");
	const candidates = (mrByNorm.get(key) || []).filter((m) => !usedMr.has(m.file + m.subfolder));
	if (candidates.length === 0) {
		cfOnly.push(cfEntry);
		continue;
	}
	const mrEntry = candidates[0];
	usedMr.add(mrEntry.file + mrEntry.subfolder);

	const mrHash = mrEntry.data.download?.hash;
	const cfHash = cfEntry.data.download?.hash;
	const mrFilename = mrEntry.data.filename;
	const cfFilename = cfEntry.data.filename;
	if (mrHash && cfHash && mrHash !== cfHash && mrFilename !== cfFilename) {
		ambiguous.push({ mr: mrEntry, cf: cfEntry, reason: "different filename+hash under matched name" });
		continue;
	}
	matched.push({ mr: mrEntry, cf: cfEntry });
}
for (const e of mrEntries) {
	if (!usedMr.has(e.file + e.subfolder)) mrOnly.push(e);
}

console.log(`${packDir}: mr=${mrEntries.length} cf=${cfEntries.length}`);
console.log(`  matched (merge): ${matched.length}`);
console.log(`  mr-only (copy through): ${mrOnly.length}`);
console.log(`  cf-only (copy through): ${cfOnly.length}`);
console.log(`  ambiguous (needs review): ${ambiguous.length}`);
for (const a of ambiguous) {
	console.log(`    - "${a.mr.data.name}" [${a.mr.file}] vs "${a.cf.data.name}" [${a.cf.file}]: ${a.reason}`);
}

if (!apply) {
	console.log("\n(dry run — pass --apply to write the merged directory)");
	process.exit(0);
}

// Build the merged directory: copy the mr side wholesale as the base (it has
// full config/, icon, pack.json etc.), then layer in cf-only content and
// rewrite matched sidecars with a combined `downloads`/`update` map.
if (existsSync(outDir)) {
	console.error(`refusing to overwrite existing ${outDir}`);
	process.exit(1);
}
cpSync(mrDir, outDir, { recursive: true });

function mergeSidecar(mrData, cfData) {
	const merged = structuredClone(mrData);
	merged.downloads = merged.downloads || {};
	// mr's own provider entry, if migrate29 hadn't already filled it in.
	if (Object.keys(merged.downloads).length === 0 && merged.download?.hash) {
		merged.downloads.modrinth = { ...merged.download };
	}
	if (cfData.downloads?.curseforge) {
		merged.downloads.curseforge = cfData.downloads.curseforge;
	} else if (cfData.download) {
		merged.downloads.curseforge = { ...cfData.download };
	}
	merged.update = merged.update || {};
	if (cfData.update?.curseforge) {
		merged.update.curseforge = cfData.update.curseforge;
	}
	return merged;
}

for (const { mr, cf } of matched) {
	const outPath = join(outDir, mr.subfolder, mr.file);
	const merged = mergeSidecar(mr.data, cf.data);
	writeFileSync(outPath, JSON.stringify(merged, null, 2) + "\n");
}

for (const cf of cfOnly) {
	const destDir = join(outDir, cf.subfolder);
	mkdirSync(destDir, { recursive: true });
	writeFileSync(join(destDir, cf.file), JSON.stringify(cf.data, null, 2) + "\n");
}

console.log(`\nwrote merged pack to ${outDir}`);
console.log(`mr dir ${mrDir} and cf dir ${cfDir} left untouched — remove manually after verification`);
