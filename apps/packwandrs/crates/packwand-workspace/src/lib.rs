//! Multi-project workspace discovery and lifecycle operations.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use packwand_ops::Workspace as PackWorkspace;
use packwand_pack::{CURRENT_PACK_FORMAT, Index, Pack, PackIndex};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod script;

pub use script::{GeneratedScript, ScriptPreset, ScriptRequest, generate_script};

const CATEGORIES: [&str; 4] = ["mods", "modpacks", "datapacks", "resourcepacks"];
const DEFAULT_MC_VERSION: &str = "26.1.2";
const DEFAULT_VERSION: &str = "26.x";
const PACKWIZ_IGNORE: &str = "Logs\n*.zip\n*.mrpack\n";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid manifest JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid pack TOML: {0}")]
    TomlDecode(#[from] toml::de::Error),
    #[error("could not encode pack TOML: {0}")]
    TomlEncode(#[from] toml::ser::Error),
    #[error("pack operation failed: {0}")]
    Pack(#[from] packwand_ops::OpsError),
    #[error("invalid project id {0:?}")]
    InvalidId(String),
    #[error("unknown project category {0:?}")]
    InvalidCategory(String),
    #[error("project {0:?} already exists")]
    AlreadyExists(String),
    #[error("project {0:?} was not found")]
    NotFound(String),
    #[error("subdirectory {0:?} is not an -mr or -cf pack")]
    InvalidSubdir(String),
    #[error("mods require at least one loader-suffixed variant")]
    ModVariantsRequired,
    #[error("variant {0:?} must end with a supported loader name")]
    InvalidVariant(String),
    #[error("base and consumes roles are mutually exclusive")]
    ConflictingRole,
    #[error("workspace sync failed: {0}")]
    Sync(String),
    #[error("could not find a repository root above {0}")]
    WorkspaceRootNotFound(PathBuf),
    #[error("invalid .pw4 script name {0:?}")]
    InvalidScriptName(String),
    #[error("{0} requires --project")]
    MissingScriptProject(String),
    #[error("script {0} already exists; pass --force to replace it")]
    ScriptAlreadyExists(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variant {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mc_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loader: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gradle_project: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Variant {
    /// How this variant names its subdirs, before the `-mr`/`-cf` suffix.
    #[must_use]
    pub fn key(&self) -> Option<&str> {
        non_empty(self.id.as_deref()).or_else(|| non_empty(self.mc_version.as_deref()))
    }
}

/// Environments a project or variant may declare.
pub const ENVIRONMENTS: [&str; 3] = ["client", "server", "both"];

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// Per-mod convention checks a pack opts into.
///
/// Packwand imposes no required files of its own: a check is dormant until the
/// pack names it here. Keys are check ids (see `packwand_diagnostics::CHECKS`),
/// so a pack declares exactly the mods whose configuration it wants validated
/// and nothing else is reported.
///
/// ```jsonc
/// "conventions": {
///   "bcc": true,                        // enable at its default level
///   "ftbquests": { "level": "warn" },   // enable, but never block a release
///   "options": { "path": "config/modpack_defaults/options.txt" }
/// }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Conventions {
    pub checks: BTreeMap<String, ConventionCheck>,
}

impl Conventions {
    /// The declared setting for `id`, or `None` when the pack did not opt in
    /// (or opted in and then disabled it).
    #[must_use]
    pub fn check(&self, id: &str) -> Option<&ConventionCheck> {
        self.checks.get(id).filter(|check| check.enabled())
    }
}

/// A single opted-in check. `true`/`false` shorthand is accepted in place of an
/// object so the common case stays a one-liner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConventionCheck {
    Enabled(bool),
    Settings(ConventionSettings),
}

impl ConventionCheck {
    #[must_use]
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => *enabled,
            Self::Settings(settings) => settings.enabled.unwrap_or(true),
        }
    }

    /// Declared severity override, if any.
    #[must_use]
    pub fn level(&self) -> Option<&str> {
        match self {
            Self::Enabled(_) => None,
            Self::Settings(settings) => non_empty(settings.level.as_deref()),
        }
    }

    /// Explicit path override, for packs whose file sits somewhere unusual.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        match self {
            Self::Enabled(_) => None,
            Self::Settings(settings) => non_empty(settings.path.as_deref()),
        }
    }

    /// Expected name, when it should differ from the manifest's own name.
    #[must_use]
    pub fn expect_name(&self) -> Option<&str> {
        match self {
            Self::Enabled(_) => None,
            Self::Settings(settings) => non_empty(settings.expect_name.as_deref()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConventionSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect_name: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Automation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_update: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_promo: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sync_exclude: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub environment_exempt: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_on_build: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sync_variants: Vec<Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub freeze: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_auto: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type", default)]
    pub project_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loader: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mc_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<Variant>,
    #[serde(default)]
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modrinth_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curseforge_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gitea_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gitlab_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_assets: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automation: Option<Automation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conventions: Option<Conventions>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Manifest {
    /// Convention checks this pack opted into. Empty when the field is absent,
    /// which is why adding this never changes an existing manifest's meaning.
    #[must_use]
    pub fn conventions(&self) -> Conventions {
        self.conventions.clone().unwrap_or_default()
    }

    pub fn effective_name(&self) -> &str {
        if self.name.is_empty() {
            &self.id
        } else {
            &self.name
        }
    }

    pub fn lifecycle(&self) -> &str {
        self.lifecycle.as_deref().unwrap_or("active")
    }

    /// The environment this project ships into. `both` when undeclared, which
    /// keeps every existing manifest meaning exactly what it meant before the
    /// field existed.
    #[must_use]
    pub fn environment(&self) -> &str {
        non_empty(self.environment.as_deref()).unwrap_or("both")
    }

    /// The environment for one variant, falling back to the project-level
    /// declaration. `key` is a subdir name with its `-mr`/`-cf` suffix
    /// removed, which is how variants name themselves on disk.
    #[must_use]
    pub fn environment_for(&self, key: &str) -> &str {
        self.variants
            .iter()
            .find(|variant| variant.key() == Some(key))
            .and_then(|variant| non_empty(variant.environment.as_deref()))
            .unwrap_or_else(|| self.environment())
    }

    pub fn automation(&self) -> Automation {
        self.automation.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub category: String,
    pub root: PathBuf,
    pub manifest: Manifest,
    pub subdirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRole {
    None,
    Base,
    Consumes(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewProject {
    pub category: String,
    pub id: String,
    pub name: Option<String>,
    pub minecraft_version: Option<String>,
    pub loader: Option<String>,
    #[serde(default)]
    pub variants: Vec<String>,
    pub role: ProjectRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitPack {
    pub name: String,
    pub author: String,
    pub version: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncReport {
    pub dry_run: bool,
    pub jobs: Vec<SyncJobReport>,
    pub copied: usize,
    pub deleted: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncJobReport {
    pub consumer: String,
    pub base: String,
    pub source: String,
    pub target: String,
    pub copied: Vec<String>,
    pub deleted: Vec<String>,
    pub excluded: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SyncState {
    version: u32,
    #[serde(default)]
    files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SyncStateRef<'a> {
    version: u32,
    files: &'a [String],
}

pub fn discover(root: impl AsRef<Path>) -> Result<Vec<Project>> {
    let root = root.as_ref();
    let mut projects = Vec::new();
    for category in CATEGORIES {
        let category_root = root.join(category);
        let entries = match fs::read_dir(&category_root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() && entry.path().join("manifest.json").is_file() {
                projects.push(read_project(root, &entry.path())?);
            }
        }
    }
    projects.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    Ok(projects)
}

/// Synchronize performance-base content into manifest-declared consumers.
/// Only files recorded in sync.json from a previous run may be pruned.
pub fn sync_performance_bases(root: impl AsRef<Path>, dry_run: bool) -> Result<SyncReport> {
    const FOLDERS: [&str; 4] = ["mods", "config", "resourcepacks", "global_packs"];
    let root = root.as_ref();
    let projects = discover(root)?;
    let by_id = projects
        .iter()
        .map(|project| (project.manifest.id.as_str(), project))
        .collect::<BTreeMap<_, _>>();
    let mut report = SyncReport {
        dry_run,
        ..SyncReport::default()
    };
    for consumer in &projects {
        let Some(role) = consumer
            .manifest
            .role
            .as_ref()
            .and_then(|role| role.get("performance_base"))
        else {
            continue;
        };
        let base_id = role
            .get("pack")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                Error::Sync(format!(
                    "{} performance_base has no pack",
                    consumer.manifest.id
                ))
            })?;
        let base = by_id.get(base_id).ok_or_else(|| {
            Error::Sync(format!(
                "{} references missing base {base_id:?}",
                consumer.manifest.id
            ))
        })?;
        if base.manifest.role.as_ref().and_then(Value::as_str) != Some("base") {
            return Err(Error::Sync(format!(
                "{} references {base_id:?}, whose role is not base",
                consumer.manifest.id
            )));
        }
        let mappings = role
            .get("mappings")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                Error::Sync(format!(
                    "{} performance_base has no mappings",
                    consumer.manifest.id
                ))
            })?;
        for mapping in mappings {
            let source = mapping
                .get("source")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::Sync("mapping has no source".into()))?;
            let target = mapping
                .get("target")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::Sync("mapping has no target".into()))?;
            validate_sync_subdir(source)?;
            validate_sync_subdir(target)?;
            if platform_suffix(source) != platform_suffix(target) {
                return Err(Error::Sync(format!(
                    "forbidden cross-platform mapping {source} -> {target}"
                )));
            }
            let source_root = base.root.join(source);
            let target_root = consumer.root.join(target);
            if !source_root.is_dir() || !target_root.is_dir() {
                return Err(Error::Sync(format!(
                    "{} -> {} requires existing source and target directories",
                    source_root.display(),
                    target_root.display()
                )));
            }
            let mut excluded = read_string_list(&target_root.join("sync-exclude.json"));
            excluded.extend(
                consumer
                    .manifest
                    .automation()
                    .sync_exclude
                    .into_iter()
                    .filter(|path| safe_relative(path)),
            );
            let mut provided = BTreeSet::new();
            for folder in FOLDERS {
                let directory = source_root.join(folder);
                if !directory.is_dir() {
                    continue;
                }
                for path in files_under(&directory)? {
                    let relative = path
                        .strip_prefix(&directory)
                        .map_err(|_| Error::Sync(path.display().to_string()))?;
                    let slash = format!("{folder}/{}", slash_path(relative));
                    if !excluded.contains(&slash) {
                        provided.insert(slash);
                    }
                }
            }
            let previous = read_sync_state(&target_root.join("sync.json"));
            let deleted = previous
                .difference(&provided)
                .filter(|path| !excluded.contains(*path))
                .cloned()
                .collect::<Vec<_>>();
            if deleted.len() > provided.len() && !provided.is_empty() {
                return Err(Error::Sync(format!(
                    "delete-set for {} exceeds files supplied by the base; remove stale sync.json and retry",
                    target_root.display()
                )));
            }
            if !dry_run {
                for relative in &provided {
                    let (folder, rest) = relative
                        .split_once('/')
                        .ok_or_else(|| Error::Sync(format!("invalid sync path {relative:?}")))?;
                    let source_path = source_root.join(folder).join(rest);
                    let target_path = target_root.join(folder).join(rest);
                    if let Some(parent) = target_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(source_path, target_path)?;
                }
                for relative in &deleted {
                    if !safe_relative(relative) {
                        return Err(Error::Sync(format!("unsafe sync state path {relative:?}")));
                    }
                    let path = target_root.join(relative);
                    if path.is_file() {
                        fs::remove_file(path)?;
                    }
                }
                let files = provided.iter().cloned().collect::<Vec<_>>();
                let mut bytes = serde_json::to_vec_pretty(&SyncStateRef {
                    version: 2,
                    files: &files,
                })?;
                bytes.push(b'\n');
                atomic_write(&target_root.join("sync.json"), &bytes)?;
                PackWorkspace::open(&target_root)?.refresh_metadata_index()?;
            }
            let job = SyncJobReport {
                consumer: consumer.manifest.id.clone(),
                base: base_id.into(),
                source: slash_path(source_root.strip_prefix(root).unwrap_or(&source_root)),
                target: slash_path(target_root.strip_prefix(root).unwrap_or(&target_root)),
                copied: provided.into_iter().collect(),
                deleted,
                excluded: excluded.into_iter().collect(),
            };
            report.copied += job.copied.len();
            report.deleted += job.deleted.len();
            report.jobs.push(job);
        }
    }
    Ok(report)
}

fn validate_sync_subdir(value: &str) -> Result<()> {
    if !safe_relative(value) || platform_suffix(value).is_none() {
        return Err(Error::Sync(format!(
            "sync subdir {value:?} must be a safe -mr or -cf path"
        )));
    }
    Ok(())
}

fn platform_suffix(value: &str) -> Option<&'static str> {
    if value.ends_with("-mr") {
        Some("mr")
    } else if value.ends_with("-cf") {
        Some("cf")
    } else {
        None
    }
}

fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn files_under(root: &Path) -> Result<Vec<PathBuf>> {
    let mut output = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                output.push(entry.path());
            }
        }
    }
    output.sort();
    Ok(output)
}

fn read_string_list(path: &Path) -> BTreeSet<String> {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Vec<String>>(&bytes).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|value| safe_relative(value))
        .collect()
}

fn read_sync_state(path: &Path) -> BTreeSet<String> {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<SyncState>(&bytes).ok())
        .filter(|state| state.version == 2)
        .map(|state| {
            state
                .files
                .into_iter()
                .filter(|value| safe_relative(value))
                .collect()
        })
        .unwrap_or_default()
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn read_project(workspace_root: &Path, project_root: &Path) -> Result<Project> {
    // Name the file that is missing: a bare io::Error surfaces as "The system
    // cannot find the file specified. (os error 2)", which tells the caller
    // nothing about which path failed.
    let manifest_path = project_root.join("manifest.json");
    let manifest: Manifest = match fs::read(&manifest_path) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::NotFound(manifest_path.display().to_string()));
        }
        Err(error) => return Err(error.into()),
    };
    let category = project_root
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::InvalidCategory(project_root.display().to_string()))?
        .to_owned();
    if !CATEGORIES.contains(&category.as_str()) || !project_root.starts_with(workspace_root) {
        return Err(Error::InvalidCategory(category));
    }
    Ok(Project {
        category,
        root: project_root.to_path_buf(),
        subdirs: subdirs_of(project_root)?,
        manifest,
    })
}

pub fn find(root: impl AsRef<Path>, id: &str) -> Result<Project> {
    discover(root)?
        .into_iter()
        .find(|project| project.manifest.id == id)
        .ok_or_else(|| Error::NotFound(id.to_owned()))
}

pub fn subdirs_of(project_root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut subdirs = Vec::new();
    for entry in fs::read_dir(project_root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with("-mr") || name.ends_with("-cf") {
                subdirs.push(entry.path());
            }
        }
    }
    subdirs.sort();
    Ok(subdirs)
}

pub fn write_manifest(project_root: impl AsRef<Path>, manifest: &Manifest) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(manifest)?;
    bytes.push(b'\n');
    atomic_write(&project_root.as_ref().join("manifest.json"), &bytes)
}

pub fn update_manifest(
    workspace_root: impl AsRef<Path>,
    id: &str,
    edit: impl FnOnce(&mut Manifest) -> Result<()>,
) -> Result<Manifest> {
    let project = find(workspace_root, id)?;
    let mut manifest = project.manifest;
    edit(&mut manifest)?;
    write_manifest(project.root, &manifest)?;
    Ok(manifest)
}

pub fn set_frozen(
    workspace_root: impl AsRef<Path>,
    id: &str,
    subdir: &str,
    slugs: &[String],
    frozen: bool,
) -> Result<Vec<String>> {
    if !(subdir.ends_with("-mr") || subdir.ends_with("-cf")) {
        return Err(Error::InvalidSubdir(subdir.to_owned()));
    }
    let project = find(&workspace_root, id)?;
    let target = project.root.join(subdir);
    if !target.is_dir() {
        return Err(Error::InvalidSubdir(subdir.to_owned()));
    }
    let mut changed = Vec::new();
    for slug in slugs {
        let metadata_path = format!("mods/{slug}.pw.toml");
        if !target.join(&metadata_path).is_file() {
            continue;
        }
        if PackWorkspace::open(&target)?.set_pinned(&metadata_path, frozen)? {
            changed.push(slug.clone());
        }
    }
    if changed.is_empty() {
        return Ok(changed);
    }
    update_manifest(workspace_root, id, |manifest| {
        let automation = manifest.automation.get_or_insert_with(Automation::default);
        let current = automation.freeze.entry(subdir.to_owned()).or_default();
        let mut values = current.iter().cloned().collect::<BTreeSet<_>>();
        for slug in &changed {
            if frozen {
                values.insert(slug.clone());
            } else {
                values.remove(slug);
            }
        }
        *current = values.into_iter().collect();
        if current.is_empty() {
            automation.freeze.remove(subdir);
        }
        Ok(())
    })?;
    Ok(changed)
}

pub fn bump(workspace_root: impl AsRef<Path>, id: &str, version: &str) -> Result<(String, String)> {
    let new_version = version.trim();
    if new_version.is_empty() {
        return Err(Error::InvalidId(version.to_owned()));
    }
    let mut previous = String::new();
    update_manifest(workspace_root, id, |manifest| {
        previous = std::mem::replace(&mut manifest.version, new_version.to_owned());
        Ok(())
    })?;
    Ok((previous, new_version.to_owned()))
}

pub fn init_pack(root: impl AsRef<Path>, request: &InitPack) -> Result<()> {
    let root = root.as_ref();
    fs::create_dir_all(root)?;
    if root.join("pack.toml").exists() {
        return Err(Error::AlreadyExists(root.display().to_string()));
    }
    let mut versions = BTreeMap::new();
    versions.insert("minecraft".to_owned(), request.minecraft_version.clone());
    versions.insert(request.loader.clone(), request.loader_version.clone());
    let pack = Pack {
        name: request.name.clone(),
        author: request.author.clone(),
        version: request.version.clone(),
        pack_format: CURRENT_PACK_FORMAT.to_owned(),
        index: PackIndex::default(),
        versions,
        ..Pack::default()
    };
    atomic_write(&root.join("pack.toml"), pack.to_toml()?.as_bytes())?;
    atomic_write(
        &root.join("index.toml"),
        toml::to_string(&Index::default())?.as_bytes(),
    )?;
    atomic_write(&root.join(".packwizignore"), PACKWIZ_IGNORE.as_bytes())
}

pub fn create_project(root: impl AsRef<Path>, request: &NewProject) -> Result<Project> {
    validate_id(&request.id)?;
    if !CATEGORIES.contains(&request.category.as_str()) {
        return Err(Error::InvalidCategory(request.category.clone()));
    }
    if request.role != ProjectRole::None && request.category == "mods" {
        return Err(Error::ConflictingRole);
    }
    if request.category == "mods" && request.variants.is_empty() {
        return Err(Error::ModVariantsRequired);
    }
    let workspace_root = root.as_ref();
    let project_root = workspace_root.join(&request.category).join(&request.id);
    if project_root.exists() {
        return Err(Error::AlreadyExists(request.id.clone()));
    }
    fs::create_dir_all(&project_root)?;
    let result = create_project_inner(workspace_root, &project_root, request);
    if result.is_err() {
        let _ = fs::remove_dir_all(&project_root);
    }
    result
}

fn create_project_inner(
    workspace_root: &Path,
    project_root: &Path,
    request: &NewProject,
) -> Result<Project> {
    let mc = request
        .minecraft_version
        .as_deref()
        .unwrap_or(DEFAULT_MC_VERSION);
    let loader = request.loader.as_deref().unwrap_or("fabric");
    validate_loader(loader)?;
    let keys = if request.variants.is_empty() {
        vec![mc.to_owned()]
    } else {
        request.variants.clone()
    };
    let variants = make_variants(&request.category, mc, &request.variants)?;
    let role = match &request.role {
        ProjectRole::None => Value::String("none".to_owned()),
        ProjectRole::Base => Value::String("base".to_owned()),
        ProjectRole::Consumes(pack) => serde_json::json!({
            "performance_base": {
                "pack": pack,
                "mappings": keys.iter().flat_map(|key| ["mr", "cf"].map(move |platform| {
                    serde_json::json!({"source": format!("CHANGEME-{platform}"), "target": format!("{key}-{platform}")})
                })).collect::<Vec<_>>()
            }
        }),
    };
    let manifest = Manifest {
        schema: Some("../../tools/manifest/schema.json".to_owned()),
        id: request.id.clone(),
        name: request.name.clone().unwrap_or_else(|| request.id.clone()),
        project_type: category_type(&request.category).to_owned(),
        loader: (request.category == "modpacks").then(|| loader.to_owned()),
        mc_version: (request.category == "modpacks" && request.variants.is_empty())
            .then(|| mc.to_owned()),
        variants,
        version: DEFAULT_VERSION.to_owned(),
        release_type: Some("release".to_owned()),
        modrinth_id: Some(request.id.clone()),
        role: Some(role),
        ..Manifest::default()
    };
    write_manifest(project_root, &manifest)?;
    atomic_write(
        &project_root.join("changelog.md"),
        format!(
            "# {}\n\nInitial scaffold. Describe the first release here.\n",
            manifest.effective_name()
        )
        .as_bytes(),
    )?;
    if request.category == "modpacks" {
        for key in keys {
            for platform in ["mr", "cf"] {
                init_pack(
                    project_root.join(format!("{key}-{platform}")),
                    &InitPack {
                        name: manifest.effective_name().to_owned(),
                        author: "CHANGEME".to_owned(),
                        version: DEFAULT_VERSION.to_owned(),
                        minecraft_version: mc.to_owned(),
                        loader: loader.to_owned(),
                        loader_version: "latest".to_owned(),
                    },
                )?;
            }
        }
    }
    read_project(workspace_root, project_root)
}

fn make_variants(category: &str, mc: &str, values: &[String]) -> Result<Vec<Variant>> {
    values
        .iter()
        .map(|id| {
            let mut variant = Variant {
                id: Some(id.clone()),
                name: Some(id.clone()),
                mc_version: Some(mc.to_owned()),
                ..Variant::default()
            };
            if category == "mods" {
                let (version, loader) = id
                    .rsplit_once('-')
                    .ok_or_else(|| Error::InvalidVariant(id.clone()))?;
                validate_loader(loader).map_err(|_| Error::InvalidVariant(id.clone()))?;
                variant.mc_version = Some(version.to_owned());
                variant.loader = Some(loader.to_owned());
                variant.gradle_project = Some(id.clone());
            }
            Ok(variant)
        })
        .collect()
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || Path::new(id)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || id.contains(['/', '\\'])
    {
        return Err(Error::InvalidId(id.to_owned()));
    }
    Ok(())
}

fn validate_loader(loader: &str) -> Result<()> {
    if matches!(loader, "fabric" | "forge" | "neoforge" | "quilt") {
        Ok(())
    } else {
        Err(Error::InvalidVariant(loader.to_owned()))
    }
}

fn category_type(category: &str) -> &str {
    match category {
        "mods" => "mod",
        "modpacks" => "modpack",
        "datapacks" => "datapack",
        "resourcepacks" => "resourcepack",
        _ => unreachable!("category is validated before conversion"),
    }
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| Error::InvalidId(path.display().to_string()))?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".{}.packwand-tmp",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    fs::write(&temp, bytes)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(category: &str, id: &str) -> NewProject {
        NewProject {
            category: category.to_owned(),
            id: id.to_owned(),
            name: None,
            minecraft_version: Some("1.21.1".to_owned()),
            loader: Some("fabric".to_owned()),
            variants: Vec::new(),
            role: ProjectRole::None,
        }
    }

    #[test]
    fn scaffolds_and_discovers_modpack_variants() {
        let root = tempfile::tempdir().unwrap();
        let mut input = request("modpacks", "example");
        input.variants = vec!["1.20.1".to_owned(), "1.21.1".to_owned()];
        let project = create_project(root.path(), &input).unwrap();
        assert_eq!(project.subdirs.len(), 4);
        assert_eq!(discover(root.path()).unwrap().len(), 1);
        for subdir in project.subdirs {
            let pack: Pack =
                toml::from_str(&fs::read_to_string(subdir.join("pack.toml")).unwrap()).unwrap();
            assert_eq!(
                pack.format().unwrap(),
                packwand_pack::PackFormat::Packwand(26)
            );
        }
    }

    #[test]
    fn preserves_unknown_manifest_fields() {
        let root = tempfile::tempdir().unwrap();
        let project_root = root.path().join("datapacks/example");
        fs::create_dir_all(&project_root).unwrap();
        fs::write(
            project_root.join("manifest.json"),
            r#"{"id":"example","type":"datapack","future":{"enabled":true}}"#,
        )
        .unwrap();
        bump(root.path(), "example", "2.0").unwrap();
        let value: Value =
            serde_json::from_slice(&fs::read(project_root.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(value["future"]["enabled"], true);
        assert_eq!(value["version"], "2.0");
    }

    #[test]
    fn rejects_traversal_ids_without_leaving_a_directory() {
        let root = tempfile::tempdir().unwrap();
        let error = create_project(root.path(), &request("modpacks", "../escape")).unwrap_err();
        assert!(matches!(error, Error::InvalidId(_)));
        assert!(!root.path().join("escape").exists());
    }

    #[test]
    fn syncs_performance_base_files_and_supports_dry_run() {
        let root = tempfile::tempdir().unwrap();
        let base = create_project(root.path(), &request("modpacks", "base")).unwrap();
        let consumer = create_project(root.path(), &request("modpacks", "consumer")).unwrap();
        let mut base_manifest = base.manifest;
        base_manifest.role = Some(Value::String("base".into()));
        write_manifest(&base.root, &base_manifest).unwrap();
        let mut consumer_manifest = consumer.manifest;
        consumer_manifest.role = Some(serde_json::json!({
            "performance_base": {
                "pack": "base",
                "mappings": [{"source":"1.21.1-mr","target":"1.21.1-mr"}]
            }
        }));
        write_manifest(&consumer.root, &consumer_manifest).unwrap();
        fs::create_dir_all(base.root.join("1.21.1-mr/config")).unwrap();
        fs::write(base.root.join("1.21.1-mr/config/base.json"), "{}").unwrap();

        let report = sync_performance_bases(root.path(), true).unwrap();
        assert_eq!(report.jobs.len(), 1);
        assert_eq!(report.copied, 1);
        assert!(!consumer.root.join("1.21.1-mr/config/base.json").exists());

        sync_performance_bases(root.path(), false).unwrap();
        assert!(consumer.root.join("1.21.1-mr/config/base.json").is_file());
        assert!(consumer.root.join("1.21.1-mr/sync.json").is_file());
    }

    #[test]
    fn discovers_only_immediate_manifest_projects() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("modpacks/a/nested")).unwrap();
        fs::write(
            root.path().join("modpacks/a/manifest.json"),
            r#"{"id":"a","type":"modpack"}"#,
        )
        .unwrap();
        fs::write(
            root.path().join("modpacks/a/nested/manifest.json"),
            r#"{"id":"nested","type":"modpack"}"#,
        )
        .unwrap();
        let projects = discover(root.path()).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].manifest.id, "a");
    }
}
