import * as fs from 'fs';
import * as path from 'path';

interface Manifest {
  id: string;
  name: string;
  type: 'modpack' | 'datapack';
  loader: string;
  mc_version: string;
  version?: string;
  release_type: 'release' | 'beta' | 'alpha';
  modrinth_id?: string;
  curseforge_id?: string;
}

function fail(msg: string): never {
  console.error(`::error::${msg}`);
  process.exit(1);
}

function warn(msg: string): void {
  console.warn(`::warning::${msg}`);
}

function validate(manifestPath: string): void {
  if (!fs.existsSync(manifestPath)) {
    fail(`manifest not found: ${manifestPath}`);
  }

  const filename = path.basename(manifestPath);
  if (filename !== 'manifest.json' && filename !== 'manifest-experimental.json') {
    fail(`unknown manifest filename: ${filename} (expected 'manifest.json' or 'manifest-experimental.json')`);
  }
  const isExperimental = filename === 'manifest-experimental.json';

  let manifest: Manifest;
  try {
    const raw = fs.readFileSync(manifestPath, 'utf-8');
    manifest = JSON.parse(raw);
  } catch (e) {
    fail(`failed to parse ${manifestPath}: ${e instanceof Error ? e.message : e}`);
  }

  const required: (keyof Manifest)[] = [
    'id', 'name', 'type', 'loader', 'mc_version', 'release_type',
  ];
  if (!isExperimental) {
    required.push('version');
  }

  for (const field of required) {
    const v = manifest[field];
    if (v === undefined || v === null || v === '') {
      fail(`manifest missing required field: ${field}`);
    }
  }

  if (!['modpack', 'datapack'].includes(manifest.type)) {
    fail(`invalid 'type': ${manifest.type} (must be 'modpack' or 'datapack')`);
  }
  if (!['release', 'beta', 'alpha'].includes(manifest.release_type)) {
    fail(`invalid 'release_type': ${manifest.release_type} (must be 'release', 'beta', or 'alpha')`);
  }

  if (isExperimental && manifest.release_type !== 'alpha') {
    warn(`experimental manifest uses release_type='${manifest.release_type}'; convention is 'alpha'`);
  }

  const hasMr = manifest.modrinth_id && manifest.modrinth_id.trim() !== '';
  const hasCf = manifest.curseforge_id && manifest.curseforge_id.trim() !== '';
  if (!hasMr && !hasCf) {
    fail('manifest must set at least one of modrinth_id or curseforge_id');
  }

  const packDir = path.dirname(manifestPath);

  if (!isExperimental) {
    const changelogPath = path.join(packDir, 'changelog.md');
    if (!fs.existsSync(changelogPath)) {
      fail(`changelog.md is missing at ${changelogPath}`);
    }
    const changelog = fs.readFileSync(changelogPath, 'utf-8').trim();
    if (changelog === '') {
      fail(`changelog.md is empty at ${changelogPath}`);
    }
    const contentLines = changelog.split('\n').filter(l => l.trim() && !l.trim().startsWith('#'));
    if (contentLines.length === 0) {
      fail(`changelog.md has headers but no content at ${changelogPath}`);
    }
  }

  if (manifest.type === 'modpack') {
    const mr = path.join(packDir, `${manifest.mc_version}-mr`);
    const cf = path.join(packDir, `${manifest.mc_version}-cf`);

    if (hasMr && !fs.existsSync(mr)) {
      fail(`modrinth_id is set but ${mr} does not exist`);
    }
    if (hasCf && !fs.existsSync(cf)) {
      fail(`curseforge_id is set but ${cf} does not exist`);
    }
    if (fs.existsSync(mr) && !hasMr) {
      warn(`${mr} exists but modrinth_id is not set`);
    }
    if (fs.existsSync(cf) && !hasCf) {
      warn(`${cf} exists but curseforge_id is not set`);
    }
  }

  if (manifest.type === 'datapack') {
    const content = path.join(packDir, 'content');
    if (!fs.existsSync(content)) {
      fail(`datapack content directory missing: ${content}`);
    }
  }

  const label = isExperimental ? 'EXPERIMENTAL' : 'production';
  const version = manifest.version ?? '(generated)';
  console.log(`✓ ${manifest.id} ${version} (${manifest.release_type}, ${label}) — manifest OK`);
}

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('usage: tsx validate.ts <path/to/manifest.json> [more manifests...]');
  process.exit(1);
}

for (const manifestPath of args) {
  validate(manifestPath);
}
