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

type Role = 'none' | 'base' | { performance_base: PerformanceBase };

interface Manifest {
  id: string;
  name: string;
  type: 'modpack' | 'datapack' | 'resourcepack';
  loader?: string;
  mc_version?: string;
  variants?: VariantEntry[];
  version?: string;
  release_type: 'release' | 'beta' | 'alpha';
  modrinth_id?: string;
  curseforge_id?: string;
  role: Role;
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

function isBaseRole(role: Role): boolean {
  return role === 'base';
}
function getPerformanceBase(role: Role): PerformanceBase | null {
  if (typeof role === 'object' && role.performance_base) return role.performance_base;
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

  for (const field of ['id', 'name', 'type', 'release_type', 'role'] as (keyof Manifest)[]) {
    const v = manifest[field];
    if (v === undefined || v === null || v === '') fail(`manifest missing required field: ${field}`);
  }

  if (!['modpack', 'datapack', 'resourcepack'].includes(manifest.type)) {
    fail(`invalid 'type': ${manifest.type}`);
  }

  if (manifest.type === 'modpack' && (!manifest.loader || manifest.loader.trim() === '')) {
    fail(`modpack manifests must declare a 'loader'`);
  }

  const hasMcVersion = manifest.mc_version !== undefined;
  const hasVariants = manifest.variants !== undefined;
  if (hasMcVersion && hasVariants) fail(`manifest declares both 'mc_version' and 'variants' \u2014 use exactly one`);
  if (!hasMcVersion && !hasVariants) fail(`manifest must declare either 'mc_version' or 'variants'`);

  if (!isExperimental && !manifest.version) fail(`manifest missing required field: version`);

  if (!['release', 'beta', 'alpha'].includes(manifest.release_type)) fail(`invalid 'release_type': ${manifest.release_type}`);
  if (isExperimental && manifest.release_type !== 'alpha') {
    warn(`experimental manifest uses release_type='${manifest.release_type}'; convention is 'alpha'`);
  }

  const hasMr = !!(manifest.modrinth_id && manifest.modrinth_id.trim());
  const hasCf = !!(manifest.curseforge_id && manifest.curseforge_id.trim());
  if (!hasMr && !hasCf) fail(`manifest must set at least one of modrinth_id or curseforge_id`);

  // --- Role validation ---
  const role = manifest.role;
  const roleIsString = typeof role === 'string';
  if (roleIsString && role !== 'none' && role !== 'base') {
    fail(`invalid 'role' string '${role}' (expected 'none', 'base', or a performance_base object)`);
  }

  if (isExperimental && isBaseRole(role)) {
    fail(`experimental manifests cannot have role 'base' (bases must be stable)`);
  }

  const pb = getPerformanceBase(role);
  if (pb) {
    if (!pb.pack || !Array.isArray(pb.mappings) || pb.mappings.length === 0) {
      fail(`role.performance_base must have a 'pack' and a non-empty 'mappings' array`);
    }
    if (pb.pack === manifest.id) {
      fail(`performance_base.pack cannot reference the pack itself ('${manifest.id}')`);
    }

    const base = loadReferencedManifest(pb.pack);
    if (!base) {
      fail(`performance_base.pack references unknown pack '${pb.pack}' (no manifest.json at ${MODPACKS_DIR}/${pb.pack}/)`);
    }
    if (!isBaseRole(base.role)) {
      fail(`performance_base.pack references '${pb.pack}', but that pack's role is not 'base'`);
    }

    const basePackDir = path.join(MODPACKS_DIR, pb.pack);
    for (const m of pb.mappings) {
      if (!m.source || !m.target) fail(`each performance_base mapping needs both 'source' and 'target'`);
      const sSuffix = platformSuffix(m.source);
      const tSuffix = platformSuffix(m.target);
      if (!sSuffix) fail(`mapping source '${m.source}' must end in '-mr' or '-cf'`);
      if (!tSuffix) fail(`mapping target '${m.target}' must end in '-mr' or '-cf'`);
      if (sSuffix !== tSuffix) {
        fail(`FORBIDDEN cross-platform mapping: source '${m.source}' (${sSuffix}) \u2192 target '${m.target}' (${tSuffix}). Modrinth and CurseForge substrates must never cross (license risk).`);
      }
      if (!fs.existsSync(path.join(basePackDir, m.source))) {
        fail(`mapping source '${m.source}' does not exist in base pack '${pb.pack}'`);
      }
      if (!fs.existsSync(path.join(packDir, m.target))) {
        fail(`mapping target '${m.target}' does not exist in this pack`);
      }
    }
  }

  if (manifest.shared_assets) {
    if (manifest.shared_assets === manifest.id) {
      fail(`'shared_assets' cannot reference the pack itself ('${manifest.id}')`);
    }
    if (!loadReferencedManifest(manifest.shared_assets)) {
      fail(`'shared_assets' references unknown pack '${manifest.shared_assets}'`);
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
  } else {
    validateZipPackStructure(packDir, manifest.type);
  }

  const label = isExperimental ? 'EXPERIMENTAL' : 'production';
  const version = manifest.version ?? '(generated)';
  const shape = hasVariants ? `multi-variant (${manifest.variants!.length})` : 'single-version';
  const roleStr = roleIsString ? (role as string) : `consumes ${pb!.pack} (${pb!.mappings.length} mappings)`;
  const sharedStr = manifest.shared_assets ? `, assets from ${manifest.shared_assets}` : '';
  console.log(`\u2713 ${manifest.id} ${version} (${manifest.release_type}, ${label}, ${shape}) [${roleStr}${sharedStr}] \u2014 manifest OK`);
}

function validateZipPackStructure(packDir: string, type: string): void {
  const entries = fs.readdirSync(packDir, { withFileTypes: true });
  const versionDirs = entries.filter((e) => e.isDirectory()).map((e) => e.name);
  if (versionDirs.length === 0) {
    fail(`${type} '${path.basename(packDir)}' has no version directory (expected ${path.basename(packDir)}/{version}/)`);
  }
  if (versionDirs.length > 1) {
    fail(`${type} '${path.basename(packDir)}' must have exactly one version directory, found ${versionDirs.length}: ${versionDirs.join(', ')}`);
  }
  const versionDir = path.join(packDir, versionDirs[0]);
  if (!fs.existsSync(path.join(versionDir, 'pack.mcmeta'))) {
    warn(`${type} version dir ${versionDir} has no pack.mcmeta at its root (Minecraft requires it)`);
  }
}

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('usage: tsx validate.ts <path/to/manifest.json> [more manifests...]');
  process.exit(1);
}
for (const manifestPath of args) validate(manifestPath);
