import * as fs from 'fs';
import * as path from 'path';

interface VariantEntry {
  mc_version: string;
  id?: string;
  name?: string;
  version?: string;
  release_type?: 'release' | 'beta' | 'alpha';
}

interface Mapping {
  source: string;
  target: string;
}

interface PerformanceBase {
  pack: string;
  mappings: Mapping[];
}

interface Manifest {
  id: string;
  name: string;
  type: 'modpack' | 'datapack';
  loader: string;
  mc_version?: string;
  variants?: VariantEntry[];
  version?: string;
  release_type: 'release' | 'beta' | 'alpha';
  modrinth_id?: string;
  curseforge_id?: string;
  base?: boolean;
  performance_base?: PerformanceBase;
  shared_assets?: string;
}

const MODPACKS_DIR = process.env.MODPACKS_DIR || 'modpacks';

function fail(msg: string): never {
  console.error(`::error::${msg}`);
  process.exit(1);
}
function warn(msg: string): void {
  console.warn(`::warning::${msg}`);
}

function readManifest(p: string): Manifest {
  if (!fs.existsSync(p)) fail(`manifest not found: ${p}`);
  try {
    return JSON.parse(fs.readFileSync(p, 'utf-8'));
  } catch (e) {
    fail(`failed to parse ${p}: ${e instanceof Error ? e.message : e}`);
  }
}

function loadReferencedManifest(packId: string): Manifest | null {
  const refPath = path.join(MODPACKS_DIR, packId, 'manifest.json');
  if (!fs.existsSync(refPath)) return null;
  try {
    return JSON.parse(fs.readFileSync(refPath, 'utf-8'));
  } catch {
    return null;
  }
}

function platformSuffix(subdir: string): 'mr' | 'cf' | null {
  if (subdir.endsWith('-mr')) return 'mr';
  if (subdir.endsWith('-cf')) return 'cf';
  return null;
}

function validate(manifestPath: string): void {
  const filename = path.basename(manifestPath);
  if (filename !== 'manifest.json' && filename !== 'manifest-experimental.json') {
    fail(`unknown manifest filename: ${filename}`);
  }
  const isExperimental = filename === 'manifest-experimental.json';
  const manifest = readManifest(manifestPath);
  const packDir = path.dirname(manifestPath);

  for (const field of ['id', 'name', 'type', 'loader', 'release_type'] as (keyof Manifest)[]) {
    const v = manifest[field];
    if (v === undefined || v === null || v === '') fail(`manifest missing required field: ${field}`);
  }

  const hasMcVersion = manifest.mc_version !== undefined;
  const hasVariants = manifest.variants !== undefined;
  if (hasMcVersion && hasVariants) fail(`manifest declares both 'mc_version' and 'variants' \u2014 use exactly one`);
  if (!hasMcVersion && !hasVariants) fail(`manifest must declare either 'mc_version' or 'variants'`);

  if (!isExperimental && !manifest.version) fail(`manifest missing required field: version`);

  if (!['modpack', 'datapack'].includes(manifest.type)) fail(`invalid 'type': ${manifest.type}`);
  if (!['release', 'beta', 'alpha'].includes(manifest.release_type)) fail(`invalid 'release_type': ${manifest.release_type}`);
  if (isExperimental && manifest.release_type !== 'alpha') {
    warn(`experimental manifest uses release_type='${manifest.release_type}'; convention is 'alpha'`);
  }

  const hasMr = !!(manifest.modrinth_id && manifest.modrinth_id.trim());
  const hasCf = !!(manifest.curseforge_id && manifest.curseforge_id.trim());
  if (!hasMr && !hasCf) fail(`manifest must set at least one of modrinth_id or curseforge_id`);

  if (isExperimental && manifest.base === true) {
    fail(`experimental manifests cannot declare 'base: true' (bases must be stable)`);
  }
  if (manifest.base === true && manifest.performance_base) {
    fail(`pack is 'base: true' and cannot also declare 'performance_base' (no chains)`);
  }

  if (manifest.performance_base) {
    const pb = manifest.performance_base;

    if (!pb.pack || !Array.isArray(pb.mappings) || pb.mappings.length === 0) {
      fail(`'performance_base' must have a 'pack' and a non-empty 'mappings' array`);
    }
    if (pb.pack === manifest.id) {
      fail(`'performance_base.pack' cannot reference the pack itself ('${manifest.id}')`);
    }

    const base = loadReferencedManifest(pb.pack);
    if (!base) {
      fail(`'performance_base.pack' references unknown pack '${pb.pack}' (no manifest.json at ${MODPACKS_DIR}/${pb.pack}/)`);
    }
    if (base.base !== true) {
      fail(`'performance_base.pack' references '${pb.pack}', but that pack does not declare 'base: true'`);
    }

    const basePackDir = path.join(MODPACKS_DIR, pb.pack);

    for (const m of pb.mappings) {
      if (!m.source || !m.target) {
        fail(`each performance_base mapping needs both 'source' and 'target'`);
      }

      const sSuffix = platformSuffix(m.source);
      const tSuffix = platformSuffix(m.target);

      if (!sSuffix) fail(`mapping source '${m.source}' must end in '-mr' or '-cf'`);
      if (!tSuffix) fail(`mapping target '${m.target}' must end in '-mr' or '-cf'`);

      if (sSuffix !== tSuffix) {
        fail(`FORBIDDEN cross-platform mapping: source '${m.source}' (${sSuffix}) \u2192 target '${m.target}' (${tSuffix}). Modrinth and CurseForge substrates must never cross (license risk).`);
      }

      const sourcePath = path.join(basePackDir, m.source);
      if (!fs.existsSync(sourcePath)) {
        fail(`mapping source '${m.source}' does not exist in base pack '${pb.pack}' (looked at ${sourcePath})`);
      }

      const targetPath = path.join(packDir, m.target);
      if (!fs.existsSync(targetPath)) {
        fail(`mapping target '${m.target}' does not exist in this pack (looked at ${targetPath})`);
      }
    }
  }

  if (manifest.shared_assets) {
    if (manifest.shared_assets === manifest.id) {
      fail(`'shared_assets' cannot reference the pack itself ('${manifest.id}')`);
    }
    if (!loadReferencedManifest(manifest.shared_assets)) {
      fail(`'shared_assets' references unknown pack '${manifest.shared_assets}' (no manifest.json at ${MODPACKS_DIR}/${manifest.shared_assets}/)`);
    }
  }

  if (!isExperimental) {
    const changelogPath = path.join(packDir, 'changelog.md');
    if (!fs.existsSync(changelogPath)) fail(`changelog.md is missing at ${changelogPath}`);
    const changelog = fs.readFileSync(changelogPath, 'utf-8').trim();
    if (changelog === '') fail(`changelog.md is empty at ${changelogPath}`);
    const contentLines = changelog.split('\n').filter((l) => l.trim() && !l.trim().startsWith('#'));
    if (contentLines.length === 0) fail(`changelog.md has headers but no content at ${changelogPath}`);
  }

  if (manifest.type === 'modpack') {
    if (hasMcVersion) {
      const mr = path.join(packDir, `${manifest.mc_version}-mr`);
      const cf = path.join(packDir, `${manifest.mc_version}-cf`);
      if (hasMr && !fs.existsSync(mr)) fail(`modrinth_id is set but ${mr} does not exist`);
      if (hasCf && !fs.existsSync(cf)) fail(`curseforge_id is set but ${cf} does not exist`);
      if (fs.existsSync(mr) && !hasMr) warn(`${mr} exists but modrinth_id is not set`);
      if (fs.existsSync(cf) && !hasCf) warn(`${cf} exists but curseforge_id is not set`);
    } else if (hasVariants) {
      for (const v of manifest.variants!) {
        const key = v.id ?? v.mc_version;
        const mr = path.join(packDir, `${key}-mr`);
        const cf = path.join(packDir, `${key}-cf`);
        if (hasMr && !fs.existsSync(mr)) fail(`variant ${key}: modrinth_id is set but ${mr} does not exist`);
        if (hasCf && !fs.existsSync(cf)) fail(`variant ${key}: curseforge_id is set but ${cf} does not exist`);
      }
    }
  }

  if (manifest.type === 'datapack') {
    const content = path.join(packDir, 'content');
    if (!fs.existsSync(content)) fail(`datapack content directory missing: ${content}`);
  }

  const label = isExperimental ? 'EXPERIMENTAL' : 'production';
  const version = manifest.version ?? '(generated)';
  const shape = hasVariants ? `multi-variant (${manifest.variants!.length})` : 'single-version';
  const roleInfo = [
    manifest.base ? 'base' : null,
    manifest.performance_base ? `consumes ${manifest.performance_base.pack} (${manifest.performance_base.mappings.length} mappings)` : null,
    manifest.shared_assets ? `assets from ${manifest.shared_assets}` : null,
  ].filter(Boolean).join(', ');
  const roleSuffix = roleInfo ? ` [${roleInfo}]` : '';
  console.log(`\u2713 ${manifest.id} ${version} (${manifest.release_type}, ${label}, ${shape})${roleSuffix} \u2014 manifest OK`);
}

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('usage: tsx validate.ts <path/to/manifest.json> [more manifests...]');
  process.exit(1);
}
for (const manifestPath of args) validate(manifestPath);
