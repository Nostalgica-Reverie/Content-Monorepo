// Audit: within one merged pack directory, find sidecar pairs that share the
// same `filename` (the actual distributed file) but live as separate
// metadata files — i.e. a merge that should have happened by filename even
// though the `name` fields didn't normalize to the same key.
import { readdirSync, readFileSync } from "fs";
import { join } from "path";

const [, , packDir] = process.argv;
const CONTENT_FOLDERS = ["mods", "resourcepacks", "shaderpacks"];

function load(dir) {
	return CONTENT_FOLDERS.flatMap((folder) => {
		const full = join(dir, folder);
		try {
			return readdirSync(full)
				.filter((f) => f.endsWith(".bun.json"))
				.map((f) => {
					const data = JSON.parse(readFileSync(join(full, f), "utf8"));
					return { folder, file: f, data };
				});
		} catch {
			return [];
		}
	});
}

const entries = load(packDir);
const byFilename = new Map();
for (const e of entries) {
	const key = e.data.filename;
	if (!key) continue;
	if (!byFilename.has(key)) byFilename.set(key, []);
	byFilename.get(key).push(e);
}
let found = 0;
for (const [filename, group] of byFilename) {
	if (group.length > 1) {
		found++;
		console.log(`${packDir}: DUPLICATE filename "${filename}"`);
		for (const g of group) {
			console.log(`    ${g.folder}/${g.file}  name="${g.data.name}"  providers=${Object.keys(g.data.downloads || {}).join(",")}`);
		}
	}
}
if (found === 0) console.log(`${packDir}: clean`);
