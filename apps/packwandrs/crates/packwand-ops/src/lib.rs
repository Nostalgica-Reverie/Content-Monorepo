//! Transactional add, remove, and metadata refresh operations.

#![forbid(unsafe_code)]

mod transaction;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use packwand_pack::{HashFormat, Index, IndexFile, Mod, Pack, hash_bytes};
use packwand_providers::{
    CurseForgeClient, ForgejoClient, GitHubClient, GitLabClient, HttpRequest, ModrinthClient,
    ProjectType, ProviderError, ProviderKind, ProviderResolver, ReleaseChannel, ResolveRequest,
    ResolvedProject, ResolvedVersion, Transport, TransportError, UreqTransport,
};
use serde::{Deserialize, Serialize};
use transaction::{FileMutation, FileTransaction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddOutcome {
    pub metadata_path: String,
    pub filename: String,
    pub replaced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOutcome {
    pub metadata_path: String,
    pub old_filename: String,
    pub new_filename: String,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRecord {
    pub path: String,
    pub name: String,
    pub provider: String,
    pub old_filename: String,
    pub new_filename: String,
    pub changed: bool,
    pub applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Resolve and optionally apply the newest compatible release for one metadata
/// file or every unaliased metadata file in a pack.
pub fn update_latest(
    root: impl Into<PathBuf>,
    selected: Option<&str>,
    all: bool,
    dry_run: bool,
) -> Result<Vec<UpdateRecord>, OpsError> {
    let root = root.into();
    // Ask Modrinth about every installed file at once before falling back to
    // per-mod lookups. Modrinth's budget is 300 requests/minute, so a
    // workspace with thousands of mods is bound by request count: batching is
    // what makes the difference, not concurrency.
    let prefetched = prefetch_modrinth(&root, selected, all).unwrap_or_default();
    update_latest_with_prefetch(root, selected, all, dry_run, resolve_provider, &prefetched)
}

/// Latest versions keyed by the installed file's hash, resolved in bulk.
type PrefetchedVersions = BTreeMap<String, ResolvedVersion>;

/// Collects the sha512/sha1 hashes of every Modrinth-backed file in scope and
/// resolves their latest versions in a handful of requests.
///
/// Best-effort: any failure returns nothing, and the caller falls back to
/// per-mod resolution, which is slower but always correct.
fn prefetch_modrinth(
    root: &Path,
    selected: Option<&str>,
    all: bool,
) -> Result<PrefetchedVersions, OpsError> {
    if !all {
        return Ok(BTreeMap::new());
    }
    let _ = selected;
    let workspace = Workspace::open(root.to_path_buf())?;
    let pack = workspace.pack().clone();
    let paths = workspace
        .index()
        .files
        .iter()
        .filter(|entry| entry.metafile && entry.alias.is_none())
        .map(|entry| entry.file.clone())
        .collect::<Vec<_>>();

    // Modrinth matches on one algorithm per call, so group by hash format and
    // ask once per group.
    let mut by_algorithm: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in &paths {
        let Ok((_, source)) = safe_relative_path(root, path) else {
            continue;
        };
        let Ok(metadata) = read_toml::<Mod>(&source) else {
            continue;
        };
        if metadata.pin || !metadata.update.contains_key("modrinth") {
            continue;
        }
        let algorithm = metadata.download.hash_format.as_str();
        if !matches!(algorithm, "sha1" | "sha512") || metadata.download.hash.is_empty() {
            continue;
        }
        by_algorithm
            .entry(algorithm.to_owned())
            .or_default()
            .push(metadata.download.hash.clone());
    }
    if by_algorithm.is_empty() {
        return Ok(BTreeMap::new());
    }

    // Same derivation the per-mod path uses, so batched and fallback lookups
    // filter identically.
    let game_versions = pack.supported_game_versions();
    let loaders = packwand_providers::modrinth_search_loaders(&pack.compatible_loaders(), false);
    let channels = [
        ReleaseChannel::Release,
        ReleaseChannel::Beta,
        ReleaseChannel::Alpha,
    ];
    let client = ModrinthClient::new(UreqTransport::new());
    let mut resolved = BTreeMap::new();
    for (algorithm, hashes) in by_algorithm {
        match client.latest_versions_by_hash(
            &hashes,
            &algorithm,
            &loaders,
            &game_versions,
            &channels,
        ) {
            Ok(batch) => resolved.extend(batch),
            // A failed batch is not fatal: those mods fall through to
            // per-mod resolution below.
            Err(_) => continue,
        }
    }
    Ok(resolved)
}

pub fn update_latest_with<F>(
    root: impl Into<PathBuf>,
    selected: Option<&str>,
    all: bool,
    dry_run: bool,
    resolver: F,
) -> Result<Vec<UpdateRecord>, OpsError>
where
    // `Fn + Sync` rather than `FnMut`: resolution runs concurrently, so the
    // resolver is shared across threads rather than called in sequence.
    F: Fn(ProviderKind, &ResolveRequest, Option<String>) -> Result<ResolvedProject, ProviderError>
        + Sync,
{
    update_latest_with_prefetch(root, selected, all, dry_run, resolver, &BTreeMap::new())
}

fn update_latest_with_prefetch<F>(
    root: impl Into<PathBuf>,
    selected: Option<&str>,
    all: bool,
    dry_run: bool,
    resolver: F,
    prefetched: &PrefetchedVersions,
) -> Result<Vec<UpdateRecord>, OpsError>
where
    F: Fn(ProviderKind, &ResolveRequest, Option<String>) -> Result<ResolvedProject, ProviderError>
        + Sync,
{
    let root = root.into();
    let mut workspace = Workspace::open(root.clone())?;
    let paths = if all {
        workspace
            .index()
            .files
            .iter()
            .filter(|entry| entry.metafile && entry.alias.is_none())
            .map(|entry| entry.file.clone())
            .collect::<Vec<_>>()
    } else if let Some(selected) = selected {
        vec![find_metadata_path(&workspace, selected)?]
    } else {
        return Err(OpsError::UpdateSelection);
    };

    /// One metadata file that still needs a provider lookup.
    struct Planned {
        path: String,
        name: String,
        old_filename: String,
        provider: ProviderKind,
        request: ResolveRequest,
        instance: Option<String>,
        installed: Option<String>,
        batched: Option<ResolvedProject>,
    }

    // Decide what each file needs before touching the network, so the lookups
    // that remain can all be issued at once.
    let mut decided: Vec<UpdateRecord> = Vec::new();
    let mut planned: Vec<Planned> = Vec::new();
    for path in paths {
        let (_, source) = safe_relative_path(&root, &path)?;
        let metadata = read_toml::<Mod>(&source)?;
        let name = metadata.name.clone();
        let old_filename = metadata.filename.clone();
        if metadata.pin {
            decided.push(update_error(path, name, old_filename, "pinned"));
            continue;
        }
        let (provider, mut request, instance) = match update_request(&metadata, workspace.pack()) {
            Ok(value) => value,
            Err(error) => {
                decided.push(update_error(path, name, old_filename, &error.to_string()));
                continue;
            }
        };
        request.channels = vec![
            ReleaseChannel::Release,
            ReleaseChannel::Beta,
            ReleaseChannel::Alpha,
        ];
        // The bulk lookup already answered for this file: no request needed.
        let batched = (provider == ProviderKind::Modrinth)
            .then(|| prefetched.get(&metadata.download.hash))
            .flatten()
            .map(|version| ResolvedProject {
                provider,
                id: request.project.clone(),
                slug: request.project.clone(),
                title: metadata.name.clone(),
                project_type: ProjectType::Mod,
                side: metadata.side.clone(),
                repository_release: None,
                version: version.clone(),
            });
        planned.push(Planned {
            path,
            name,
            old_filename,
            provider,
            request,
            instance,
            installed: installed_version(&metadata, provider),
            batched,
        });
    }

    // Resolve what the bulk lookup did not cover, concurrently. Providers with
    // a request budget still serialize on the transport's per-host gate, so
    // this speeds up the providers that have no budget without breaching the
    // ones that do.
    let resolved: Vec<Result<ResolvedProject, ProviderError>> = packwand_parallel::map(
        &planned,
        packwand_parallel::configured(),
        |entry| match &entry.batched {
            Some(project) => Ok(project.clone()),
            None => resolver(entry.provider, &entry.request, entry.instance.clone()),
        },
    );

    // Applying mutates the pack index, so it stays sequential and in order.
    let mut records = decided;
    for (entry, outcome) in planned.into_iter().zip(resolved) {
        match outcome {
            Ok(resolved) => {
                let new_filename = resolved.version.file.filename.clone();
                let changed = entry
                    .installed
                    .is_none_or(|current| current != resolved.version.id);
                let applied = changed && !dry_run;
                let error = if applied {
                    workspace
                        .update_resolved(&entry.path, resolved)
                        .err()
                        .map(|error| error.to_string())
                } else {
                    None
                };
                records.push(UpdateRecord {
                    path: entry.path,
                    name: entry.name,
                    provider: entry.provider.name().into(),
                    old_filename: entry.old_filename,
                    new_filename,
                    changed,
                    applied: applied && error.is_none(),
                    error,
                });
            }
            // Finding nothing newer is a successful check, not a failure. It
            // is the normal answer whenever a project has no release for the
            // pack's declared Minecraft version — common for resource packs,
            // which are published per game version. Reporting it as an error
            // made a healthy workspace look broken and failed the exit code.
            Err(ProviderError::NoCompatibleVersion) => records.push(UpdateRecord {
                path: entry.path,
                name: entry.name,
                provider: entry.provider.name().into(),
                old_filename: entry.old_filename.clone(),
                new_filename: entry.old_filename,
                changed: false,
                applied: false,
                error: None,
            }),
            Err(error) => records.push(UpdateRecord {
                path: entry.path,
                name: entry.name,
                provider: entry.provider.name().into(),
                old_filename: entry.old_filename.clone(),
                new_filename: entry.old_filename,
                changed: false,
                applied: false,
                error: Some(error.to_string()),
            }),
        }
    }
    Ok(records)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RefreshReport {
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RehashReport {
    pub indexed_files: usize,
    pub downloads: usize,
    pub metadata_files: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImportMergeReport {
    pub files: usize,
    pub metadata_files: usize,
}

pub struct Workspace {
    root: PathBuf,
    pack_path: PathBuf,
    index_path: PathBuf,
    pack: Pack,
    index: Index,
}

impl Workspace {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, OpsError> {
        let root = root.into();
        let pack_path = root.join("pack.toml");
        let pack = read_toml::<Pack>(&pack_path)?;
        pack.format()
            .map_err(|error| OpsError::PackFormat(error.to_string()))?;
        let (_, index_path) = safe_relative_path(&root, &pack.index.file)?;
        let index = match fs::read_to_string(&index_path) {
            Ok(source) => toml::from_str(&source).map_err(|source| OpsError::Toml {
                path: index_path.clone(),
                source,
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Index::default(),
            Err(source) => {
                return Err(OpsError::Io {
                    path: index_path,
                    source,
                });
            }
        };
        Ok(Self {
            root,
            pack_path,
            index_path,
            pack,
            index,
        })
    }

    pub fn pack(&self) -> &Pack {
        &self.pack
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    /// Migrates the pack format marker without changing indexed content.
    pub fn migrate_format(&mut self) -> Result<(String, String), OpsError> {
        let old = if self.pack.pack_format.is_empty() {
            "packwiz:1.1.0".to_owned()
        } else {
            self.pack.pack_format.clone()
        };
        let new = packwand_pack::CURRENT_PACK_FORMAT.to_owned();
        if old == new {
            return Ok((old, new));
        }
        let mut pack = self.pack.clone();
        pack.pack_format = new.clone();
        pack.index.hash.clear();
        commit_documents(
            &self.pack_path,
            &self.index_path,
            &pack,
            &self.index,
            Vec::new(),
        )?;
        self.pack = pack;
        Ok((old, new))
    }

    /// Changes a Minecraft or loader version and commits the pack document.
    pub fn set_version(
        &mut self,
        component: &str,
        version: &str,
    ) -> Result<Option<String>, OpsError> {
        if component.trim().is_empty() || version.trim().is_empty() {
            return Err(OpsError::PackFormat(
                "version component and value must not be empty".into(),
            ));
        }
        let mut pack = self.pack.clone();
        let old = pack
            .versions
            .insert(component.to_owned(), version.to_owned());
        pack.index.hash.clear();
        commit_documents(
            &self.pack_path,
            &self.index_path,
            &pack,
            &self.index,
            Vec::new(),
        )?;
        self.pack = pack;
        Ok(old)
    }

    /// Replaces a string-list pack option and commits pack.toml.
    pub fn set_string_list_option(
        &mut self,
        key: &str,
        values: Vec<String>,
    ) -> Result<Vec<String>, OpsError> {
        if key.trim().is_empty() {
            return Err(OpsError::PackFormat("option key must not be empty".into()));
        }
        let previous = self
            .pack
            .options
            .get(key)
            .and_then(toml::Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(toml::Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let mut pack = self.pack.clone();
        pack.options.insert(
            key.into(),
            toml::Value::Array(values.into_iter().map(toml::Value::String).collect()),
        );
        pack.index.hash.clear();
        commit_documents(
            &self.pack_path,
            &self.index_path,
            &pack,
            &self.index,
            Vec::new(),
        )?;
        self.pack = pack;
        Ok(previous)
    }

    pub fn add_resolved(
        &mut self,
        resolved: ResolvedProject,
        replace: bool,
    ) -> Result<AddOutcome, OpsError> {
        let metadata_path = resolved.metadata_path();
        let metadata = resolved.into_mod()?;
        self.add_metadata(&metadata_path, metadata, replace)
    }

    /// Replace a local jar/litemod with resolved metadata as one transaction.
    /// This is used by CurseForge fingerprint detection so the local file and
    /// its old index entry cannot be left behind after metadata is created.
    pub fn replace_local_with_resolved(
        &mut self,
        local_relative: &str,
        resolved: ResolvedProject,
    ) -> Result<AddOutcome, OpsError> {
        let (_, local_path) = safe_relative_path(&self.root, local_relative)?;
        if !local_path.is_file() {
            return Err(OpsError::NotFound(local_relative.into()));
        }
        let metadata_relative = resolved.metadata_path();
        let metadata = resolved.into_mod()?;
        let (metadata_relative, metadata_path) =
            safe_relative_path(&self.root, &metadata_relative)?;
        if metadata_path.exists() {
            return Err(OpsError::AlreadyExists(metadata_relative));
        }
        let metadata_bytes = metadata.to_toml()?.into_bytes();
        let mut index = self.index.clone();
        index.files.retain(|entry| entry.file != local_relative);
        upsert_index_file(
            &mut index,
            &metadata_relative,
            hash_bytes(HashFormat::Sha512, &metadata_bytes),
        );
        let mut pack = self.pack.clone();
        pack.index.hash_format = "sha512".into();
        pack.index.hash.clear();
        commit_documents(
            &self.pack_path,
            &self.index_path,
            &pack,
            &index,
            vec![
                FileMutation::write(metadata_path, metadata_bytes),
                FileMutation::remove(local_path),
            ],
        )?;
        self.pack = pack;
        self.index = index;
        Ok(AddOutcome {
            metadata_path: metadata_relative,
            filename: metadata.filename,
            replaced: false,
        })
    }

    /// Merge the indexed content and version matrix from an imported pack.
    /// All copied files plus the generated index and pack documents commit in
    /// one transaction, preserving existing pack identity and settings.
    pub fn merge_imported_pack(
        &mut self,
        imported_root: impl AsRef<Path>,
    ) -> Result<ImportMergeReport, OpsError> {
        let imported_root = imported_root.as_ref();
        let imported_pack: Pack = read_toml(&imported_root.join("pack.toml"))?;
        let imported_index: Index = read_toml(&imported_root.join(&imported_pack.index.file))?;
        let format = self
            .index
            .hash_format
            .parse::<HashFormat>()
            .map_err(|error| OpsError::PackFormat(error.to_string()))?;
        let mut index = self.index.clone();
        let mut mutations = Vec::new();
        let mut report = ImportMergeReport::default();
        for imported in imported_index.files {
            let (relative, source) = safe_relative_path(imported_root, &imported.file)?;
            let (_, target) = safe_relative_path(&self.root, &relative)?;
            if target.exists() && !target.is_file() {
                return Err(OpsError::InvalidMetadataPath(relative));
            }
            let bytes = fs::read(&source).map_err(|error| OpsError::Io {
                path: source.clone(),
                source: error,
            })?;
            index
                .files
                .retain(|entry| entry.file != relative || entry.alias != imported.alias);
            let mut entry = imported;
            entry.file = relative;
            entry.hash = hash_bytes(format, &bytes);
            entry.hash_format = None;
            if entry.metafile {
                report.metadata_files += 1;
            }
            report.files += 1;
            index.files.push(entry);
            mutations.push(FileMutation::write(target, bytes));
        }
        index.files.sort_by(|left, right| {
            left.file
                .cmp(&right.file)
                .then_with(|| left.alias.cmp(&right.alias))
        });
        let mut pack = self.pack.clone();
        pack.versions.extend(imported_pack.versions);
        pack.index.hash_format = index.hash_format.clone();
        pack.index.hash.clear();
        commit_documents(&self.pack_path, &self.index_path, &pack, &index, mutations)?;
        self.pack = pack;
        self.index = index;
        Ok(report)
    }

    pub fn add_metadata(
        &mut self,
        relative: &str,
        metadata: Mod,
        replace: bool,
    ) -> Result<AddOutcome, OpsError> {
        if !relative.ends_with(".pw.toml") {
            return Err(OpsError::InvalidMetadataPath(relative.to_string()));
        }
        let (relative, metadata_path) = safe_relative_path(&self.root, relative)?;
        let replaced = metadata_path.exists();
        if replaced && !metadata_path.is_file() {
            return Err(OpsError::InvalidMetadataPath(relative));
        }
        if replaced && !replace {
            return Err(OpsError::AlreadyExists(relative));
        }
        let metadata_bytes = metadata.to_toml()?.into_bytes();
        let hash = hash_bytes(HashFormat::Sha512, &metadata_bytes);
        let mut index = self.index.clone();
        upsert_index_file(&mut index, &relative, hash);
        let mut pack = self.pack.clone();
        pack.index.hash_format = "sha512".to_string();
        pack.index.hash.clear();
        commit_documents(
            &self.pack_path,
            &self.index_path,
            &pack,
            &index,
            vec![FileMutation::write(metadata_path, metadata_bytes)],
        )?;
        self.pack = pack;
        self.index = index;
        Ok(AddOutcome {
            metadata_path: relative,
            filename: metadata.filename,
            replaced,
        })
    }

    pub fn remove_metadata(&mut self, relative: &str) -> Result<(), OpsError> {
        let (relative, metadata_path) = safe_relative_path(&self.root, relative)?;
        if !metadata_path.is_file() {
            return Err(OpsError::NotFound(relative));
        }
        // Validate before staging deletion so corrupt metadata is surfaced.
        let _: Mod = read_toml(&metadata_path)?;
        let mut index = self.index.clone();
        index.files.retain(|entry| entry.file != relative);
        let mut pack = self.pack.clone();
        pack.index.hash_format = "sha512".to_string();
        pack.index.hash.clear();
        commit_documents(
            &self.pack_path,
            &self.index_path,
            &pack,
            &index,
            vec![FileMutation::remove(metadata_path)],
        )?;
        self.pack = pack;
        self.index = index;
        Ok(())
    }

    /// Changes the packwiz pin flag and commits the metadata, index and pack
    /// hashes as one transaction.
    pub fn set_pinned(&mut self, relative: &str, pinned: bool) -> Result<bool, OpsError> {
        let (_, metadata_path) = safe_relative_path(&self.root, relative)?;
        let mut metadata: Mod = read_toml(&metadata_path)?;
        if metadata.pin == pinned {
            return Ok(false);
        }
        metadata.pin = pinned;
        self.add_metadata(relative, metadata, true)?;
        Ok(true)
    }

    /// Changes the side declaration and commits all generated hashes together.
    pub fn set_side(&mut self, relative: &str, side: &str) -> Result<bool, OpsError> {
        if !matches!(side, "client" | "server" | "both") {
            return Err(OpsError::InvalidMetadataPath(format!(
                "invalid side {side:?}"
            )));
        }
        let (_, metadata_path) = safe_relative_path(&self.root, relative)?;
        let mut metadata: Mod = read_toml(&metadata_path)?;
        if metadata.side == side {
            return Ok(false);
        }
        metadata.side = side.to_owned();
        self.add_metadata(relative, metadata, true)?;
        Ok(true)
    }

    pub fn update_resolved(
        &mut self,
        relative: &str,
        resolved: ResolvedProject,
    ) -> Result<UpdateOutcome, OpsError> {
        let (relative, metadata_path) = safe_relative_path(&self.root, relative)?;
        let existing: Mod = read_toml(&metadata_path)?;
        if existing.pin {
            return Err(OpsError::Pinned(relative));
        }
        let provider = resolved.provider;
        let provider_name = provider.name();
        let provider_data = existing
            .update
            .get(provider_name)
            .ok_or_else(|| OpsError::MissingProviderMetadata(provider_name.to_string()))?;
        let installed_project = match provider {
            ProviderKind::Modrinth => provider_data
                .get("mod-id")
                .and_then(toml::Value::as_str)
                .map(str::to_string),
            ProviderKind::CurseForge => provider_data
                .get("project-id")
                .and_then(toml::Value::as_integer)
                .map(|id| id.to_string()),
            ProviderKind::GitHub => provider_data
                .get("slug")
                .and_then(toml::Value::as_str)
                .map(str::to_string),
            ProviderKind::Forgejo | ProviderKind::GitLab => {
                let instance = provider_data.get("instance").and_then(toml::Value::as_str);
                let slug = provider_data.get("slug").and_then(toml::Value::as_str);
                instance
                    .zip(slug)
                    .map(|(instance, slug)| format!("{instance}/{slug}"))
            }
        };
        let resolved_project = match provider {
            ProviderKind::Forgejo | ProviderKind::GitLab => resolved
                .repository_release
                .as_ref()
                .and_then(|release| release.instance.as_deref())
                .map(|instance| format!("{instance}/{}", resolved.id)),
            _ => Some(resolved.id.clone()),
        };
        if installed_project != resolved_project {
            return Err(OpsError::ProviderProjectMismatch {
                expected: installed_project.unwrap_or_else(|| "<missing>".into()),
                resolved: resolved_project.unwrap_or_else(|| "<missing>".into()),
            });
        }
        let installed_id = match provider {
            ProviderKind::Modrinth => provider_data
                .get("version")
                .and_then(toml::Value::as_str)
                .map(str::to_string),
            ProviderKind::CurseForge => provider_data
                .get("file-id")
                .and_then(toml::Value::as_integer)
                .map(|id| id.to_string()),
            ProviderKind::Forgejo | ProviderKind::GitHub | ProviderKind::GitLab => provider_data
                .get("tag")
                .and_then(toml::Value::as_str)
                .map(str::to_string),
        };
        if installed_id.as_deref() == Some(resolved.version.id.as_str()) {
            return Ok(UpdateOutcome {
                metadata_path: relative,
                old_filename: existing.filename.clone(),
                new_filename: existing.filename,
                changed: false,
            });
        }

        let old_filename = existing.filename.clone();
        let mut updated = resolved.into_mod()?;
        updated.name = existing.name.clone();
        updated.side = existing.side;
        updated.pin = existing.pin;
        updated.option = existing.option;
        let new_provider_data = updated
            .update
            .remove(provider_name)
            .ok_or_else(|| OpsError::ProviderMismatch(provider_name.to_string()))?;
        updated.update = existing.update;
        let provider_data = updated.update.entry(provider_name.to_string()).or_default();
        provider_data.extend(new_provider_data);
        let new_filename = updated.filename.clone();
        self.add_metadata(&relative, updated, true)?;
        Ok(UpdateOutcome {
            metadata_path: relative,
            old_filename,
            new_filename,
            changed: true,
        })
    }

    /// Replace one metadata entry with an explicitly selected provider file.
    ///
    /// Unlike ordinary update this intentionally permits changing providers,
    /// e.g. when a release is available on CurseForge before Modrinth.
    pub fn replace_with_resolved(
        &mut self,
        relative: &str,
        resolved: ResolvedProject,
    ) -> Result<UpdateOutcome, OpsError> {
        let (relative, source) = safe_relative_path(&self.root, relative)?;
        let existing = read_toml::<Mod>(&source)?;
        let old_filename = existing.filename.clone();
        let mut updated = resolved.into_mod()?;
        updated.name = existing.name;
        updated.side = existing.side;
        updated.pin = existing.pin;
        updated.option = existing.option;
        let new_filename = updated.filename.clone();
        let changed = old_filename != new_filename || existing.update != updated.update;
        if changed {
            self.add_metadata(&relative, updated, true)?;
        }
        Ok(UpdateOutcome {
            metadata_path: relative,
            old_filename,
            new_filename,
            changed,
        })
    }

    /// Re-hashes one edited `.pw.toml` and updates its generated index entry.
    pub fn refresh_metadata(&mut self, relative: &str) -> Result<String, OpsError> {
        let (relative, metadata_path) = safe_relative_path(&self.root, relative)?;
        let bytes = fs::read(&metadata_path).map_err(|source| OpsError::Io {
            path: metadata_path.clone(),
            source,
        })?;
        toml::from_str::<Mod>(
            std::str::from_utf8(&bytes)
                .map_err(|_| OpsError::InvalidMetadataPath(format!("{relative} is not UTF-8")))?,
        )
        .map_err(|source| OpsError::Toml {
            path: metadata_path,
            source,
        })?;
        let hash = hash_bytes(HashFormat::Sha512, &bytes);
        let mut index = self.index.clone();
        upsert_index_file(&mut index, &relative, hash.clone());
        let mut pack = self.pack.clone();
        pack.index.hash_format = "sha512".to_string();
        pack.index.hash.clear();
        commit_documents(&self.pack_path, &self.index_path, &pack, &index, Vec::new())?;
        self.pack = pack;
        self.index = index;
        Ok(hash)
    }

    /// Rebuilds the generated index entries for every `.pw.toml` below the pack root.
    ///
    /// Non-metafile entries are preserved. Missing metafiles are removed, and all
    /// discovered metadata is parsed before either generated document is replaced.
    pub fn refresh_metadata_index(&mut self) -> Result<RefreshReport, OpsError> {
        let mut paths = Vec::new();
        collect_metadata_paths(&self.root, &mut paths)?;
        paths.sort();

        let previous: BTreeMap<String, String> = self
            .index
            .files
            .iter()
            .filter(|entry| entry.metafile && entry.alias.is_none())
            .map(|entry| (entry.file.clone(), entry.hash.clone()))
            .collect();
        let mut found = BTreeSet::new();
        let mut refreshed = Vec::with_capacity(paths.len());
        let mut report = RefreshReport::default();
        for path in paths {
            let relative = pack_relative_path(&self.root, &path)?;
            let bytes = fs::read(&path).map_err(|source| OpsError::Io {
                path: path.clone(),
                source,
            })?;
            let source = std::str::from_utf8(&bytes)
                .map_err(|_| OpsError::InvalidMetadataPath(format!("{relative} is not UTF-8")))?;
            toml::from_str::<Mod>(source).map_err(|source| OpsError::Toml {
                path: path.clone(),
                source,
            })?;
            let hash = hash_bytes(HashFormat::Sha512, &bytes);
            match previous.get(&relative) {
                None => report.added += 1,
                Some(old_hash) if old_hash != &hash => report.updated += 1,
                Some(_) => {}
            }
            found.insert(relative.clone());
            refreshed.push((relative, hash));
        }
        report.removed = previous
            .keys()
            .filter(|path| !found.contains(*path))
            .count();

        let mut index = self.index.clone();
        index
            .files
            .retain(|entry| !entry.metafile || entry.alias.is_some());
        for (relative, hash) in refreshed {
            upsert_index_file(&mut index, &relative, hash);
        }
        let mut pack = self.pack.clone();
        pack.index.hash_format = "sha512".to_string();
        pack.index.hash.clear();
        commit_documents(&self.pack_path, &self.index_path, &pack, &index, Vec::new())?;
        self.pack = pack;
        self.index = index;
        Ok(report)
    }

    /// Migrates index and external-download hashes as one transaction.
    pub fn rehash(&mut self, format: HashFormat) -> Result<RehashReport, OpsError> {
        if format.is_internal() || matches!(format, HashFormat::Murmur2 | HashFormat::Md5) {
            return Err(OpsError::PackFormat(format!(
                "{} is not supported for pack rehashing",
                format.as_str()
            )));
        }
        // Rehashing re-downloads each external file to hash it, so this needs
        // the transfer-scale client rather than the API one.
        let transport = UreqTransport::for_downloads();
        let mut index = self.index.clone();
        let mut mutations = Vec::new();
        let mut report = RehashReport::default();
        for entry in &mut index.files {
            let (_, source_path) = safe_relative_path(&self.root, &entry.file)?;
            if entry.metafile {
                let mut metadata: Mod = read_toml(&source_path)?;
                if !metadata.download.url.is_empty() {
                    let bytes = transport.get(HttpRequest::get(&metadata.download.url))?;
                    if !metadata.download.hash.is_empty()
                        && metadata.download.hash_format != format.as_str()
                    {
                        metadata.download.extra_hashes.insert(
                            metadata.download.hash_format.clone(),
                            metadata.download.hash.clone(),
                        );
                    }
                    metadata.download.hash_format = format.as_str().to_owned();
                    metadata.download.hash = hash_bytes(format, &bytes);
                    metadata.download.size = bytes.len() as u64;
                    report.downloads += 1;
                }
                let bytes = metadata.to_toml()?.into_bytes();
                entry.hash = hash_bytes(format, &bytes);
                entry.hash_format = None;
                mutations.push(FileMutation::write(source_path, bytes));
                report.metadata_files += 1;
            } else {
                let bytes = fs::read(&source_path).map_err(|source| OpsError::Io {
                    path: source_path.clone(),
                    source,
                })?;
                entry.hash = hash_bytes(format, &bytes);
                entry.hash_format = None;
            }
            report.indexed_files += 1;
        }
        index.hash_format = format.as_str().to_owned();
        let mut pack = self.pack.clone();
        pack.index.hash_format = format.as_str().to_owned();
        pack.index.hash.clear();
        commit_documents(&self.pack_path, &self.index_path, &pack, &index, mutations)?;
        self.pack = pack;
        self.index = index;
        Ok(report)
    }
}

fn collect_metadata_paths(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), OpsError> {
    let entries = fs::read_dir(directory).map_err(|source| OpsError::Io {
        path: directory.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| OpsError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| OpsError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            let name = entry.file_name();
            if name != ".git" && name != ".packwand-launcher" && name != "target" {
                collect_metadata_paths(&path, paths)?;
            }
        } else if file_type.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".pw.toml"))
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn pack_relative_path(root: &Path, path: &Path) -> Result<String, OpsError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| OpsError::UnsafePath(path.display().to_string()))?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            _ => return Err(OpsError::UnsafePath(path.display().to_string())),
        }
    }
    Ok(parts.join("/"))
}

fn read_toml<T: for<'de> serde::Deserialize<'de>>(path: &Path) -> Result<T, OpsError> {
    let source = fs::read_to_string(path).map_err(|source| OpsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&source).map_err(|source| OpsError::Toml {
        path: path.to_path_buf(),
        source,
    })
}

fn upsert_index_file(index: &mut Index, relative: &str, hash: String) {
    if let Some(entry) = index
        .files
        .iter_mut()
        .find(|entry| entry.file == relative && entry.alias.is_none())
    {
        entry.hash = hash;
        entry.hash_format = None;
        entry.metafile = true;
    } else {
        index.files.push(IndexFile {
            file: relative.to_string(),
            hash,
            metafile: true,
            ..IndexFile::default()
        });
    }
    index.hash_format = "sha512".to_string();
    index.files.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then_with(|| left.alias.cmp(&right.alias))
    });
}

fn commit_documents(
    pack_path: &Path,
    index_path: &Path,
    pack: &Pack,
    index: &Index,
    mut mutations: Vec<FileMutation>,
) -> Result<(), OpsError> {
    let index_bytes = toml::to_string(index)?.into_bytes();
    let format = index
        .hash_format
        .parse::<HashFormat>()
        .map_err(|error| OpsError::PackFormat(error.to_string()))?;
    let mut pack = pack.clone();
    pack.index.hash_format = index.hash_format.clone();
    pack.index.hash = hash_bytes(format, &index_bytes);
    mutations.push(FileMutation::write(index_path.to_path_buf(), index_bytes));
    mutations.push(FileMutation::write(
        pack_path.to_path_buf(),
        pack.to_toml()?.into_bytes(),
    ));
    FileTransaction::new(mutations).commit()?;
    Ok(())
}

fn safe_relative_path(root: &Path, relative: &str) -> Result<(String, PathBuf), OpsError> {
    let path = Path::new(relative);
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            _ => return Err(OpsError::UnsafePath(relative.to_string())),
        }
    }
    if parts.is_empty() {
        return Err(OpsError::UnsafePath(relative.to_string()));
    }
    let normalized = parts.join("/");
    Ok((
        normalized,
        parts
            .iter()
            .fold(root.to_path_buf(), |path, part| path.join(part)),
    ))
}

fn find_metadata_path(workspace: &Workspace, selected: &str) -> Result<String, OpsError> {
    let normalized = selected.replace('\\', "/");
    if normalized.ends_with(".pw.toml") {
        return workspace
            .index()
            .files
            .iter()
            .find(|entry| entry.file == normalized && entry.metafile)
            .map(|entry| entry.file.clone())
            .ok_or(OpsError::NotFound(normalized));
    }
    workspace
        .index()
        .files
        .iter()
        .filter(|entry| entry.metafile)
        .find(|entry| {
            Path::new(&entry.file)
                .file_stem()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.trim_end_matches(".pw") == selected)
        })
        .map(|entry| entry.file.clone())
        .ok_or_else(|| OpsError::NotFound(selected.into()))
}

fn update_error(path: String, name: String, filename: String, error: &str) -> UpdateRecord {
    UpdateRecord {
        path,
        name,
        provider: String::new(),
        old_filename: filename.clone(),
        new_filename: filename,
        changed: false,
        applied: false,
        error: Some(error.into()),
    }
}

fn update_request(
    metadata: &Mod,
    pack: &Pack,
) -> Result<(ProviderKind, ResolveRequest, Option<String>), OpsError> {
    let (provider, table, project) = if let Some(table) = metadata.update.get("modrinth") {
        (
            ProviderKind::Modrinth,
            table,
            table
                .get("mod-id")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .to_owned(),
        )
    } else if let Some(table) = metadata.update.get("curseforge") {
        (
            ProviderKind::CurseForge,
            table,
            table
                .get("project-id")
                .and_then(toml::Value::as_integer)
                .map(|value| value.to_string())
                .unwrap_or_default(),
        )
    } else if let Some(table) = metadata.update.get("github") {
        (
            ProviderKind::GitHub,
            table,
            table
                .get("slug")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .to_owned(),
        )
    } else if let Some(table) = metadata.update.get("forgejo") {
        (
            ProviderKind::Forgejo,
            table,
            table
                .get("slug")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .to_owned(),
        )
    } else if let Some(table) = metadata.update.get("gitlab") {
        (
            ProviderKind::GitLab,
            table,
            table
                .get("slug")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .to_owned(),
        )
    } else {
        return Err(OpsError::UpdateMetadata(
            "metadata has no supported update provider".into(),
        ));
    };
    if project.is_empty() {
        return Err(OpsError::UpdateMetadata(format!(
            "{} update metadata has no project id",
            provider.name()
        )));
    }
    let mut request = ResolveRequest::new(project);
    request.game_versions = pack.supported_game_versions();
    // Modrinth files resource packs under the "minecraft" loader and shaders
    // under "vanilla"/"iris"/etc., so a search restricted to the pack's mod
    // loader alone reports those as having no compatible version.
    request.loaders = if provider == ProviderKind::Modrinth {
        packwand_providers::modrinth_search_loaders(&pack.compatible_loaders(), false)
    } else {
        pack.compatible_loaders()
    };
    request.branch = table
        .get("branch")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    request.asset_pattern = table
        .get("regex")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    let instance = table
        .get("instance")
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    Ok((provider, request, instance))
}

fn installed_version(metadata: &Mod, provider: ProviderKind) -> Option<String> {
    let table = metadata.update.get(provider.name())?;
    let key = match provider {
        ProviderKind::Modrinth => "version",
        ProviderKind::CurseForge => "file-id",
        ProviderKind::Forgejo | ProviderKind::GitHub | ProviderKind::GitLab => "tag",
    };
    table.get(key).and_then(|value| {
        value
            .as_str()
            .map(str::to_owned)
            .or_else(|| value.as_integer().map(|value| value.to_string()))
    })
}

fn resolve_provider(
    provider: ProviderKind,
    request: &ResolveRequest,
    instance: Option<String>,
) -> Result<ResolvedProject, ProviderError> {
    let transport = UreqTransport::new();
    Ok(match provider {
        ProviderKind::Modrinth => ModrinthClient::new(transport).resolve(request)?,
        ProviderKind::CurseForge => {
            CurseForgeClient::new(transport, packwand_providers::configured_api_key())
                .resolve(request)?
        }
        ProviderKind::GitHub => {
            GitHubClient::new(transport, std::env::var("GITHUB_TOKEN").unwrap_or_default())
                .resolve(request)?
        }
        ProviderKind::Forgejo => match instance {
            Some(instance) => ForgejoClient::for_instance(
                transport,
                instance,
                std::env::var("FORGEJO_TOKEN").unwrap_or_default(),
            )
            .resolve(request)?,
            None => ForgejoClient::new(
                transport,
                std::env::var("FORGEJO_TOKEN").unwrap_or_default(),
            )
            .resolve(request)?,
        },
        ProviderKind::GitLab => match instance {
            Some(instance) => GitLabClient::for_instance(
                transport,
                instance,
                std::env::var("GITLAB_TOKEN").unwrap_or_default(),
            )
            .resolve(request)?,
            None => GitLabClient::new(transport, std::env::var("GITLAB_TOKEN").unwrap_or_default())
                .resolve(request)?,
        },
    })
}

#[derive(Debug, thiserror::Error)]
pub enum OpsError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error("invalid pack format: {0}")]
    PackFormat(String),
    #[error("unsafe pack-relative path {0:?}")]
    UnsafePath(String),
    #[error("metadata path must end in .pw.toml: {0:?}")]
    InvalidMetadataPath(String),
    #[error("metadata file already exists: {0}")]
    AlreadyExists(String),
    #[error("metadata file was not found: {0}")]
    NotFound(String),
    #[error("metadata file is pinned and cannot be updated: {0}")]
    Pinned(String),
    #[error("resolved project did not contain {0} update metadata")]
    ProviderMismatch(String),
    #[error("metadata file does not contain {0} update metadata")]
    MissingProviderMetadata(String),
    #[error("provide a metadata name or request all metadata")]
    UpdateSelection,
    #[error("invalid update metadata: {0}")]
    UpdateMetadata(String),
    #[error("resolved provider project {resolved} does not match installed project {expected}")]
    ProviderProjectMismatch { expected: String, resolved: String },
    #[error("failed to access {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to decode TOML at {path}: {source}")]
    Toml {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error(transparent)]
    Serialize(#[from] toml::ser::Error),
    #[error(transparent)]
    Transaction(#[from] transaction::TransactionError),
}
