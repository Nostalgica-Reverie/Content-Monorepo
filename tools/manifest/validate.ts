import * as fs from 'node:fs';
import * as path from 'node:path';

interface VariantEntry {
  mc_version: string;
  id?: string;
  name?: string;
  version?: string;
  release_type?: 'release' | 'beta' | 'alpha';
  loader?: string;
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

interface Automation {
  auto_update?: boolean;
  server_promo?: boolean;
  sync_exclude?: string[];
  freeze?: Record<string, string[]>;
}

interface Manifest {
  id: string;
  name: string;
  type: 'modpack' | 'datapack' | 'resourcepack';
  loader?: string;
  mc_version?: string;
  variants?: VariantEntry[];
  version?: string;
  release_type: 'release' | 'beta' | 'alpha';
  description?: string;
  modrinth_id?: string;
  curseforge_id?: string;
  role: Role;
  shared_assets?: string;
  automation?: Automation;
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
    return JSON.parse(fs.readFileSync(p, 'utf-8')) as Manifest;
  } catch (e) {
    fail(`failed to parse ${p}: ${e instanceof Error ? e.message : e}`);
  }
}

function loadReferencedManifest(packId: string): Manifest | null {
  const refPath = path.join(MODPACKS_DIR, packId, 'manifest.json');
  if (!fs.existsSync(refPath)) return null;
  try {
    return JSON.parse(fs.readFileSync(refPath, 'utf-8')) as Manifest;
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

  const variants = manifest.variants ?? [];

  if (manifest.type === 'modpack' && variants.length === 0 && (!manifest.loader || manifest.loader.trim() === '')) {
    fail(`modpack manifests must declare a 'loader'`);
  }

  if (manifest.type === 'modpack' && variants.length > 0) {
    const byVersion = new Map<string, VariantEntry[]>();
    for (const v of variants) {
      const list = byVersion.get(v.mc_version) ?? [];
      list.push(v);
      byVersion.set(v.mc_version, list);
    }
    for (const [mc, list] of byVersion) {
      if (list.length > 1) {
        for (const v of list) {
          if (!v.id || v.id.trim() === '') {
            fail(`variant for mc_version '${mc}' shares that version with another variant and must declare a distinct 'id' (e.g. '${mc}-fabric')`);
          }
        }
        const ids = list.map((v) => v.id ?? '');
        const dupeIds = ids.filter((id, i) => ids.indexOf(id) !== i);
        if (dupeIds.length > 0) {
          fail(`duplicate variant id(s) for mc_version '${mc}': ${[...new Set(dupeIds)].join(', ')}`);
        }
        const withLoader = list.filter((v) => v.loader && v.loader.trim() !== '');
        const loaders = withLoader.map((v) => v.loader);
        const dupeLoaders = loaders.filter((l, i) => loaders.indexOf(l) !== i);
        if (dupeLoaders.length > 0) {
          fail(`two variants share both mc_version '${mc}' and loader '${[...new Set(dupeLoaders)].join(', ')}' \u2014 give them distinct ids or loaders`);
        }
      }
    }
    for (const v of variants) {
      const resolvedLoader = v.loader ?? manifest.loader;
      if (!resolvedLoader || resolvedLoader.trim() === '') {
        const key = v.id ?? v.mc_version;
        fail(`variant '${key}' has no loader: set a variant 'loader' or a pack-level 'loader'`);
      }
    }
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
    } else {
      for (const v of variants) {
        const key = v.id ?? v.mc_version;
        const mr = path.join(packDir, `${key}-mr`);
        const cf = path.join(packDir, `${key}-cf`);
        const mrPresent = fs.existsSync(mr);
        const cfPresent = fs.existsSync(cf);
        if (hasMr && !mrPresent && !cfPresent) {
          fail(`variant ${key}: has neither ${path.basename(mr)} nor ${path.basename(cf)} — nothing to publish`);
        }
        if (hasMr && !mrPresent) warn(`variant ${key}: ${mr} absent — this variant will NOT publish to Modrinth`);
        if (hasCf && !cfPresent) warn(`variant ${key}: ${cf} absent — this variant will NOT publish to CurseForge`);
      }
    }
  } else {
    validateZipPackStructure(packDir, manifest.type);
  }

  validateAutomation(manifestPath, manifest.automation, packDir);
  validateOptOut(packDir);
  if (fs.existsSync(path.join(packDir, 'auto-update-ignore.json'))) {
    warn(`${packDir}: legacy auto-update-ignore.json — migrate to manifest.json "automation"`);
  }

  const label = isExperimental ? 'EXPERIMENTAL' : 'production';
  const version = manifest.version ?? '(generated)';
  const shape = hasVariants ? `multi-variant (${variants.length})` : 'single-version';
  const roleStr = pb ? `consumes ${pb.pack} (${pb.mappings.length} mappings)` : (role as string);
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
  const versionDir = path.join(packDir, versionDirs[0] ?? '');
  if (!fs.existsSync(path.join(versionDir, 'pack.mcmeta'))) {
    warn(`${type} version dir ${versionDir} has no pack.mcmeta at its root (Minecraft requires it)`);
  }
}

function validateAutomation(manifestPath: string, auto: Automation | undefined, packDir: string): void {
  if (auto === undefined) return;
  if (typeof auto !== 'object' || auto === null || Array.isArray(auto)) {
    fail(`${manifestPath}: 'automation' must be an object`);
  }
  const allowed = ['auto_update', 'server_promo', 'sync_exclude', 'freeze'];
  for (const key of Object.keys(auto)) {
    if (!allowed.includes(key)) {
      fail(`${manifestPath}: automation: unknown key '${key}' (allowed: ${allowed.join(', ')})`);
    }
  }
  const o = auto as Record<string, unknown>;
  for (const boolKey of ['auto_update', 'server_promo']) {
    if (boolKey in o && typeof o[boolKey] !== 'boolean') {
      fail(`${manifestPath}: automation.'${boolKey}' must be a boolean`);
    }
  }
  if ('sync_exclude' in o) {
    const v = o.sync_exclude;
    if (!Array.isArray(v) || v.some((x) => typeof x !== 'string')) {
      fail(`${manifestPath}: automation.sync_exclude must be an array of strings`);
    }
  }
  if ('freeze' in o) {
    const f = o.freeze;
    if (typeof f !== 'object' || f === null || Array.isArray(f)) {
      fail(`${manifestPath}: automation.freeze must be an object of subdir-key -> string array`);
    }
    for (const [sub, list] of Object.entries(f as Record<string, unknown>)) {
      if (!Array.isArray(list) || list.some((x) => typeof x !== 'string')) {
        fail(`${manifestPath}: automation.freeze['${sub}'] must be an array of strings`);
      }
      if (!fs.existsSync(path.join(packDir, sub))) {
        warn(`${manifestPath}: automation.freeze references subdir '${sub}' which does not exist`);
      }
    }
  }
}

function validateOptOut(packDir: string): void {
  const p = path.join(packDir, 'opt-out.json');
  if (!fs.existsSync(p)) return;
  warn(`${p}: opt-out.json is deprecated — migrate into manifest.json "automation"`);
  let obj: unknown;
  try {
    obj = JSON.parse(fs.readFileSync(p, 'utf-8'));
  } catch (e) {
    fail(`invalid JSON in ${p}: ${e instanceof Error ? e.message : e}`);
  }
  if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) {
    fail(`${p} must be a JSON object`);
  }
  const allowed = ['auto_update', 'server_promo', 'sync_exclude', 'freeze'];
  for (const key of Object.keys(obj)) {
    if (!allowed.includes(key)) {
      fail(`${p}: unknown key '${key}' (allowed: ${allowed.join(', ')})`);
    }
  }
  const o = obj as Record<string, unknown>;
  for (const boolKey of ['auto_update', 'server_promo']) {
    if (boolKey in o && typeof o[boolKey] !== 'boolean') {
      fail(`${p}: '${boolKey}' must be a boolean`);
    }
  }
  for (const listKey of ['sync_exclude', 'freeze']) {
    if (listKey in o) {
      const v = o[listKey];
      if (!Array.isArray(v) || v.some((x) => typeof x !== 'string')) {
        fail(`${p}: '${listKey}' must be an array of strings`);
      }
    }
  }
}

function discoverManifests(): string[] {
  const found: string[] = [];
  for (const root of [MODPACKS_DIR, 'datapacks', 'resourcepacks']) {
    if (!fs.existsSync(root)) continue;
    for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;
      for (const name of ['manifest.json', 'manifest-experimental.json']) {
        const p = path.join(root, entry.name, name);
        if (fs.existsSync(p)) found.push(p);
      }
    }
  }
  return found;
}

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('usage: bun validate.ts <path/to/manifest.json> [more manifests...] | bun validate.ts --all');
  process.exit(1);
}

const targets = args[0] === '--all' ? discoverManifests() : args;
if (targets.length === 0) fail('--all found no manifests (run from the repo root)');
for (const manifestPath of targets) validate(manifestPath);
if (args[0] === '--all') console.log(`\u2713 all ${targets.length} manifest(s) OK`);