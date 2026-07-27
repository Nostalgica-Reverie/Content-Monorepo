//! Side-effect-free export planning shared by the desktop app and future CLI.

#![forbid(unsafe_code)]

mod installer;
mod packeater;
mod publish;

pub use installer::{InstallerTestReport, test_with_installer};
pub use packeater::{
    PACKEATER_MARKER, archive_content_directory, discover_packeater_markers, run_packeater,
};
pub use publish::{
    PublishArtifact, PublishMatrixEntry, PublishTarget, PublishUploadReport, build_publish_target,
    list_publish_targets, resolve_publish_target, upload_publish_target, verify_publish_target,
};

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use packwand_pack::{HashFormat, Index, Mod, Pack, hash_bytes};
use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPlan {
    pub pack_name: String,
    pub pack_version: String,
    pub output_stem: String,
    pub indexed_files: usize,
    pub metadata_files: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Modrinth,
    CurseForge,
}

impl ExportFormat {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Modrinth => "mrpack",
            Self::CurseForge => "zip",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportArtifact {
    pub path: PathBuf,
    pub format: ExportFormat,
    pub files: usize,
    pub bytes: u64,
}

/// Transactionally zip the contents of a standalone datapack or resource
/// pack directory. The directory itself is not included as a wrapper folder.
pub fn archive_directory(
    root: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<u64, BuildError> {
    let root = root.as_ref();
    let output = output.as_ref();
    if !root.is_dir() {
        return Err(BuildError::InvalidPack(format!(
            "{} is not a directory",
            root.display()
        )));
    }
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| BuildError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let temporary = tempfile::NamedTempFile::new_in(parent).map_err(|source| BuildError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let mut archive = ZipWriter::new(temporary);
    write_directory_entries(root, root, &mut archive)?;
    let temporary = archive.finish().map_err(BuildError::Zip)?.into_temp_path();
    let size = fs::metadata(&temporary)
        .map_err(|source| BuildError::Io {
            path: temporary.to_path_buf(),
            source,
        })?
        .len();
    temporary.persist(output).map_err(|error| BuildError::Io {
        path: output.to_path_buf(),
        source: error.error,
    })?;
    Ok(size)
}

fn write_directory_entries<W: Write + Seek>(
    root: &Path,
    directory: &Path,
    archive: &mut ZipWriter<W>,
) -> Result<(), BuildError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|source| BuildError::Io {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|source| BuildError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| BuildError::UnsafePath(path.display().to_string()))?
            .to_string_lossy()
            .replace('\\', "/");
        let file_type = entry.file_type().map_err(|source| BuildError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            archive
                .add_directory(format!("{relative}/"), zip_options())
                .map_err(BuildError::Zip)?;
            write_directory_entries(root, &path, archive)?;
        } else if file_type.is_file() {
            write_file(archive, &relative, &path)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub path: PathBuf,
    pub name: String,
    pub version: String,
    pub minecraft_version: Option<String>,
    pub loader: Option<String>,
    pub files: usize,
    pub metadata_files: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportOptions {
    pub restrict_modrinth_domains: bool,
    pub verify_hashes: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            restrict_modrinth_domains: true,
            verify_hashes: false,
        }
    }
}

pub fn plan_export(root: impl AsRef<Path>) -> Result<ExportPlan, BuildError> {
    let root = root.as_ref();
    let pack_path = root.join("pack.toml");
    let pack: Pack = read_toml(&pack_path)?;
    pack.format()
        .map_err(|error| BuildError::InvalidPack(error.to_string()))?;
    let index_path = root.join(&pack.index.file);
    let index: Index = read_toml(&index_path)?;
    let name = if pack.name.trim().is_empty() {
        root.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("pack")
            .to_string()
    } else {
        pack.name.clone()
    };
    let output_stem = [name.as_str(), pack.version.as_str()]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    Ok(ExportPlan {
        pack_name: name,
        pack_version: pack.version,
        output_stem,
        indexed_files: index.files.len(),
        metadata_files: index.files.iter().filter(|file| file.metafile).count(),
    })
}

/// Build a launcher-compatible archive. The archive is first written beside
/// the destination and is only persisted after every entry succeeds.
pub fn export_pack(
    root: impl AsRef<Path>,
    format: ExportFormat,
    output: Option<impl AsRef<Path>>,
    options: ExportOptions,
) -> Result<ExportArtifact, BuildError> {
    let root = root.as_ref();
    let plan = plan_export(root)?;
    let destination = output
        .map(|path| path.as_ref().to_path_buf())
        .unwrap_or_else(|| root.join(format!("{}.{}", plan.output_stem, format.extension())));
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| BuildError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let temp = tempfile::NamedTempFile::new_in(parent).map_err(|source| BuildError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let mut archive = ZipWriter::new(temp);
    let (pack, index) = load(root)?;
    let count = match format {
        ExportFormat::Modrinth => write_modrinth(root, &pack, &index, &mut archive, options)?,
        ExportFormat::CurseForge => write_curseforge(root, &pack, &index, &mut archive)?,
    };
    let temp = archive.finish().map_err(BuildError::Zip)?.into_temp_path();
    let bytes = fs::metadata(&temp)
        .map_err(|source| BuildError::Io {
            path: temp.to_path_buf(),
            source,
        })?
        .len();
    temp.persist(&destination).map_err(|error| BuildError::Io {
        path: destination.clone(),
        source: error.error,
    })?;
    Ok(ExportArtifact {
        path: destination,
        format,
        files: count,
        bytes,
    })
}

/// Import a Modrinth archive into a new packwiz/Packwand directory. Archive
/// paths are validated and the completed tree is renamed into place at once.
pub fn import_modrinth_archive(
    archive_path: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<ImportResult, BuildError> {
    let archive_path = archive_path.as_ref();
    let destination = destination.as_ref();
    if destination.exists() {
        return Err(BuildError::DestinationExists(destination.to_path_buf()));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| BuildError::UnsafePath(destination.display().to_string()))?;
    fs::create_dir_all(parent).map_err(|source| BuildError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let file = fs::File::open(archive_path).map_err(|source| BuildError::Io {
        path: archive_path.to_path_buf(),
        source,
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(BuildError::Zip)?;
    let manifest: ImportedModrinthManifest = {
        let mut entry = archive
            .by_name("modrinth.index.json")
            .map_err(|_| BuildError::UnsupportedArchive("missing modrinth.index.json".into()))?;
        if entry.size() > 8 * 1024 * 1024 {
            return Err(BuildError::UnsupportedArchive(
                "manifest exceeds 8 MiB".into(),
            ));
        }
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|source| BuildError::Io {
                path: archive_path.to_path_buf(),
                source,
            })?;
        serde_json::from_slice(&bytes).map_err(BuildError::Json)?
    };
    if manifest.format_version != 1 || manifest.game != "minecraft" {
        return Err(BuildError::UnsupportedArchive(format!(
            "unsupported Modrinth format {} for game {:?}",
            manifest.format_version, manifest.game
        )));
    }
    let temporary = tempfile::tempdir_in(parent).map_err(|source| BuildError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let staging = temporary.path().join("pack");
    fs::create_dir(&staging).map_err(|source| BuildError::Io {
        path: staging.clone(),
        source,
    })?;
    let mut index = Index::default();
    let mut metadata_count = 0usize;
    let mut used_metadata = std::collections::BTreeSet::new();
    for imported in manifest.files {
        let path = normalize_index_path(&imported.path)?;
        let filename = Path::new(&path)
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| BuildError::UnsafePath(path.clone()))?;
        let parent_path = Path::new(&path)
            .parent()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .replace('\\', "/");
        let mut slug = slugify(filename.trim_end_matches(".jar"));
        if slug.is_empty() {
            slug = format!("external-{}", metadata_count + 1);
        }
        let mut metadata_path = if parent_path.is_empty() {
            format!("{slug}.pw.toml")
        } else {
            format!("{parent_path}/{slug}.pw.toml")
        };
        let mut suffix = 2;
        while !used_metadata.insert(metadata_path.clone()) {
            metadata_path = if parent_path.is_empty() {
                format!("{slug}-{suffix}.pw.toml")
            } else {
                format!("{parent_path}/{slug}-{suffix}.pw.toml")
            };
            suffix += 1;
        }
        let (hash_format, hash) = imported
            .hashes
            .get("sha512")
            .map(|hash| ("sha512", hash.clone()))
            .or_else(|| {
                imported
                    .hashes
                    .get("sha1")
                    .map(|hash| ("sha1", hash.clone()))
            })
            .ok_or_else(|| BuildError::InvalidMetadata(format!("{path} has no SHA hash")))?;
        let side = match (&imported.env.client, &imported.env.server) {
            (client, server) if client == "unsupported" && server != "unsupported" => "server",
            (client, server) if server == "unsupported" && client != "unsupported" => "client",
            _ => "both",
        };
        let optional = imported.env.client == "optional" || imported.env.server == "optional";
        let url =
            imported.downloads.first().cloned().ok_or_else(|| {
                BuildError::InvalidMetadata(format!("{path} has no download URL"))
            })?;
        let mut extra_hashes = imported.hashes;
        extra_hashes.remove(hash_format);
        let mut update = BTreeMap::new();
        if let Ok(parsed) = url::Url::parse(&url) {
            let parts = parsed
                .path_segments()
                .map(Iterator::collect::<Vec<_>>)
                .unwrap_or_default();
            if parsed.host_str() == Some("cdn.modrinth.com")
                && parts.first() == Some(&"data")
                && parts.get(2) == Some(&"versions")
                && parts.len() >= 4
            {
                update.insert(
                    "modrinth".into(),
                    toml::Table::from_iter([
                        ("mod-id".into(), toml::Value::String(parts[1].into())),
                        ("version".into(), toml::Value::String(parts[3].into())),
                    ]),
                );
            }
        }
        let metadata = Mod {
            name: filename.trim_end_matches(".jar").replace(['-', '_'], " "),
            filename: filename.to_owned(),
            side: side.into(),
            download: packwand_pack::Download {
                url,
                hash_format: hash_format.into(),
                hash,
                extra_hashes,
                size: imported.file_size,
                ..packwand_pack::Download::default()
            },
            option: optional.then(|| packwand_pack::ModOption {
                optional: true,
                default: true,
                ..packwand_pack::ModOption::default()
            }),
            update,
            ..Mod::default()
        };
        let bytes = metadata
            .to_toml()
            .map_err(BuildError::TomlEncode)?
            .into_bytes();
        let target = staging.join(metadata_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| BuildError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&target, &bytes).map_err(|source| BuildError::Io {
            path: target,
            source,
        })?;
        index.files.push(packwand_pack::IndexFile {
            file: metadata_path,
            hash: hash_bytes(HashFormat::Sha512, &bytes),
            metafile: true,
            ..packwand_pack::IndexFile::default()
        });
        metadata_count += 1;
    }
    let mut extracted_bytes = 0u64;
    for position in 0..archive.len() {
        let mut entry = archive.by_index(position).map_err(BuildError::Zip)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().replace('\\', "/");
        let relative = ["overrides/", "client-overrides/", "server-overrides/"]
            .into_iter()
            .find_map(|prefix| name.strip_prefix(prefix));
        let Some(relative) = relative else { continue };
        let relative = normalize_index_path(relative)?;
        if entry.size() > 512 * 1024 * 1024 {
            return Err(BuildError::UnsupportedArchive(format!(
                "{relative} exceeds 512 MiB"
            )));
        }
        extracted_bytes = extracted_bytes.saturating_add(entry.size());
        if extracted_bytes > 2 * 1024 * 1024 * 1024 {
            return Err(BuildError::UnsupportedArchive(
                "expanded archive exceeds 2 GiB".into(),
            ));
        }
        let target = staging.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| BuildError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut bytes)
            .map_err(|source| BuildError::Io {
                path: archive_path.to_path_buf(),
                source,
            })?;
        fs::write(&target, &bytes).map_err(|source| BuildError::Io {
            path: target,
            source,
        })?;
        index.files.push(packwand_pack::IndexFile {
            file: relative,
            hash: hash_bytes(HashFormat::Sha512, &bytes),
            ..packwand_pack::IndexFile::default()
        });
    }
    index
        .files
        .sort_by(|left, right| left.file.cmp(&right.file));
    let mut versions = manifest.dependencies;
    for (source, target) in [("fabric-loader", "fabric"), ("quilt-loader", "quilt")] {
        if let Some(version) = versions.remove(source) {
            versions.insert(target.into(), version);
        }
    }
    let pack = Pack {
        name: manifest.name.clone(),
        version: manifest.version_id.clone(),
        description: manifest.summary,
        pack_format: packwand_pack::CURRENT_PACK_FORMAT.into(),
        versions: versions.clone(),
        ..Pack::default()
    };
    fs::write(
        staging.join("pack.toml"),
        pack.to_toml().map_err(BuildError::TomlEncode)?,
    )
    .map_err(|source| BuildError::Io {
        path: staging.join("pack.toml"),
        source,
    })?;
    fs::write(
        staging.join("index.toml"),
        toml::to_string(&index).map_err(BuildError::TomlEncode)?,
    )
    .map_err(|source| BuildError::Io {
        path: staging.join("index.toml"),
        source,
    })?;
    fs::write(staging.join(".packwizignore"), "Logs\n*.zip\n*.mrpack\n").map_err(|source| {
        BuildError::Io {
            path: staging.join(".packwizignore"),
            source,
        }
    })?;
    fs::rename(&staging, destination).map_err(|source| BuildError::Io {
        path: destination.to_path_buf(),
        source,
    })?;
    let loader = ["fabric", "forge", "neoforge", "quilt"]
        .into_iter()
        .find(|loader| versions.contains_key(*loader))
        .map(str::to_owned);
    Ok(ImportResult {
        path: destination.to_path_buf(),
        name: manifest.name,
        version: manifest.version_id,
        minecraft_version: versions.get("minecraft").cloned(),
        loader,
        files: index.files.len(),
        metadata_files: metadata_count,
    })
}

/// Import a CurseForge launcher archive. Project/file metadata is resolved by
/// the caller so the archive engine remains independent of API credentials
/// and can be shared by the CLI and desktop job system.
pub fn import_curseforge_archive<F>(
    archive_path: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    mut resolve: F,
) -> Result<ImportResult, BuildError>
where
    F: FnMut(i64, i64) -> Result<(String, Mod), String>,
{
    let archive_path = archive_path.as_ref();
    let destination = destination.as_ref();
    if destination.exists() {
        return Err(BuildError::DestinationExists(destination.to_path_buf()));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| BuildError::UnsafePath(destination.display().to_string()))?;
    fs::create_dir_all(parent).map_err(|source| BuildError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let file = fs::File::open(archive_path).map_err(|source| BuildError::Io {
        path: archive_path.to_path_buf(),
        source,
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(BuildError::Zip)?;
    let manifest: ImportedCurseForgeManifest = {
        let mut entry = archive
            .by_name("manifest.json")
            .map_err(|_| BuildError::UnsupportedArchive("missing manifest.json".into()))?;
        if entry.size() > 8 * 1024 * 1024 {
            return Err(BuildError::UnsupportedArchive(
                "manifest exceeds 8 MiB".into(),
            ));
        }
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|source| BuildError::Io {
                path: archive_path.to_path_buf(),
                source,
            })?;
        serde_json::from_slice(&bytes).map_err(BuildError::Json)?
    };
    if manifest.manifest_type != "minecraftModpack" || manifest.manifest_version != 1 {
        return Err(BuildError::UnsupportedArchive(format!(
            "unsupported CurseForge manifest type {:?} version {}",
            manifest.manifest_type, manifest.manifest_version
        )));
    }
    let temporary = tempfile::tempdir_in(parent).map_err(|source| BuildError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let staging = temporary.path().join("pack");
    fs::create_dir(&staging).map_err(|source| BuildError::Io {
        path: staging.clone(),
        source,
    })?;
    let mut index = Index::default();
    let mut metadata_count = 0usize;
    let mut used_metadata = std::collections::BTreeSet::new();
    for imported in manifest.files {
        let (suggested_path, mut metadata) = resolve(imported.project_id, imported.file_id)
            .map_err(|message| BuildError::ProviderResolution {
                project_id: imported.project_id,
                file_id: imported.file_id,
                message,
            })?;
        if !imported.required {
            metadata.option = Some(packwand_pack::ModOption {
                optional: true,
                default: false,
                description: String::new(),
            });
        }
        let path = unique_metadata_path(&suggested_path, metadata_count, &mut used_metadata)?;
        let bytes = metadata
            .to_toml()
            .map_err(BuildError::TomlEncode)?
            .into_bytes();
        let target = staging.join(path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| BuildError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&target, &bytes).map_err(|source| BuildError::Io {
            path: target,
            source,
        })?;
        index.files.push(packwand_pack::IndexFile {
            file: path,
            hash: hash_bytes(HashFormat::Sha512, &bytes),
            metafile: true,
            ..packwand_pack::IndexFile::default()
        });
        metadata_count += 1;
    }

    let override_prefix = manifest.overrides.trim_matches(['/', '\\']);
    if override_prefix.is_empty() {
        return Err(BuildError::UnsafePath(manifest.overrides));
    }
    let prefix = format!("{}/", override_prefix.replace('\\', "/"));
    let mut extracted_bytes = 0u64;
    for position in 0..archive.len() {
        let mut entry = archive.by_index(position).map_err(BuildError::Zip)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().replace('\\', "/");
        let Some(relative) = name.strip_prefix(&prefix) else {
            continue;
        };
        let relative = normalize_index_path(relative)?;
        if entry.size() > 512 * 1024 * 1024 {
            return Err(BuildError::UnsupportedArchive(format!(
                "{relative} exceeds 512 MiB"
            )));
        }
        extracted_bytes = extracted_bytes.saturating_add(entry.size());
        if extracted_bytes > 2 * 1024 * 1024 * 1024 {
            return Err(BuildError::UnsupportedArchive(
                "expanded archive exceeds 2 GiB".into(),
            ));
        }
        let target = staging.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| BuildError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut bytes)
            .map_err(|source| BuildError::Io {
                path: archive_path.to_path_buf(),
                source,
            })?;
        fs::write(&target, &bytes).map_err(|source| BuildError::Io {
            path: target,
            source,
        })?;
        index.files.push(packwand_pack::IndexFile {
            file: relative,
            hash: hash_bytes(HashFormat::Sha512, &bytes),
            ..packwand_pack::IndexFile::default()
        });
    }
    index
        .files
        .sort_by(|left, right| left.file.cmp(&right.file));
    let loader = manifest
        .minecraft
        .mod_loaders
        .iter()
        .find(|loader| loader.primary)
        .or_else(|| manifest.minecraft.mod_loaders.first())
        .and_then(|loader| parse_curseforge_loader(&loader.id));
    let mut versions = BTreeMap::from([("minecraft".into(), manifest.minecraft.version.clone())]);
    if let Some((name, version)) = &loader {
        versions.insert(name.clone(), version.clone());
    }
    let pack = Pack {
        name: manifest.name.clone(),
        author: manifest.author,
        version: manifest.version.clone(),
        pack_format: packwand_pack::CURRENT_PACK_FORMAT.into(),
        versions,
        ..Pack::default()
    };
    fs::write(
        staging.join("pack.toml"),
        pack.to_toml().map_err(BuildError::TomlEncode)?,
    )
    .map_err(|source| BuildError::Io {
        path: staging.join("pack.toml"),
        source,
    })?;
    fs::write(
        staging.join("index.toml"),
        toml::to_string(&index).map_err(BuildError::TomlEncode)?,
    )
    .map_err(|source| BuildError::Io {
        path: staging.join("index.toml"),
        source,
    })?;
    fs::write(staging.join(".packwizignore"), "Logs\n*.zip\n*.mrpack\n").map_err(|source| {
        BuildError::Io {
            path: staging.join(".packwizignore"),
            source,
        }
    })?;
    fs::rename(&staging, destination).map_err(|source| BuildError::Io {
        path: destination.to_path_buf(),
        source,
    })?;
    Ok(ImportResult {
        path: destination.to_path_buf(),
        name: manifest.name,
        version: manifest.version,
        minecraft_version: Some(manifest.minecraft.version),
        loader: loader.map(|(name, _)| name),
        files: index.files.len(),
        metadata_files: metadata_count,
    })
}

fn unique_metadata_path(
    suggested: &str,
    index: usize,
    used: &mut std::collections::BTreeSet<String>,
) -> Result<String, BuildError> {
    let suggested = normalize_index_path(suggested)?;
    let (parent, filename) = suggested
        .rsplit_once('/')
        .map_or(("", suggested.as_str()), |(parent, filename)| {
            (parent, filename)
        });
    let stem = filename.trim_end_matches(".pw.toml");
    let stem = if stem.is_empty() {
        format!("curseforge-{}", index + 1)
    } else {
        stem.to_owned()
    };
    let mut suffix = 1usize;
    loop {
        let filename = if suffix == 1 {
            format!("{stem}.pw.toml")
        } else {
            format!("{stem}-{suffix}.pw.toml")
        };
        let candidate = if parent.is_empty() {
            filename
        } else {
            format!("{parent}/{filename}")
        };
        if used.insert(candidate.clone()) {
            return Ok(candidate);
        }
        suffix += 1;
    }
}

fn parse_curseforge_loader(value: &str) -> Option<(String, String)> {
    for loader in ["neoforge", "fabric", "forge", "quilt"] {
        if let Some(version) = value.strip_prefix(&format!("{loader}-"))
            && !version.is_empty()
        {
            return Some((loader.into(), version.into()));
        }
    }
    None
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportedCurseForgeManifest {
    minecraft: ImportedCurseForgeMinecraft,
    manifest_type: String,
    manifest_version: u32,
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    files: Vec<ImportedCurseForgeFile>,
    #[serde(default = "default_overrides")]
    overrides: String,
}

fn default_overrides() -> String {
    "overrides".into()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportedCurseForgeMinecraft {
    version: String,
    #[serde(default)]
    mod_loaders: Vec<ImportedCurseForgeLoader>,
}

#[derive(Deserialize)]
struct ImportedCurseForgeLoader {
    id: String,
    #[serde(default)]
    primary: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportedCurseForgeFile {
    #[serde(rename = "projectID")]
    project_id: i64,
    #[serde(rename = "fileID")]
    file_id: i64,
    #[serde(default = "required_file")]
    required: bool,
}

const fn required_file() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportedModrinthManifest {
    format_version: u32,
    game: String,
    version_id: String,
    name: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    files: Vec<ImportedModrinthFile>,
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportedModrinthFile {
    path: String,
    hashes: BTreeMap<String, String>,
    #[serde(default)]
    env: ImportedModrinthEnv,
    #[serde(default)]
    downloads: Vec<String>,
    #[serde(default)]
    file_size: u64,
}

#[derive(Default, Deserialize)]
struct ImportedModrinthEnv {
    #[serde(default = "required_environment")]
    client: String,
    #[serde(default = "required_environment")]
    server: String,
}

fn required_environment() -> String {
    "required".into()
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            separator = false;
        } else if !separator && !slug.is_empty() {
            slug.push('-');
            separator = true;
        }
    }
    slug.trim_end_matches('-').into()
}

fn load(root: &Path) -> Result<(Pack, Index), BuildError> {
    let pack: Pack = read_toml(&root.join("pack.toml"))?;
    pack.format()
        .map_err(|error| BuildError::InvalidPack(error.to_string()))?;
    let index = read_toml(&root.join(&pack.index.file))?;
    Ok((pack, index))
}

fn zip_options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)
}

fn write_bytes<W: Write + Seek>(
    archive: &mut ZipWriter<W>,
    path: &str,
    bytes: &[u8],
) -> Result<(), BuildError> {
    validate_archive_path(path)?;
    archive
        .start_file(path, zip_options())
        .map_err(BuildError::Zip)?;
    archive.write_all(bytes).map_err(|source| BuildError::Io {
        path: PathBuf::from(path),
        source,
    })
}

fn write_file<W: Write + Seek>(
    archive: &mut ZipWriter<W>,
    path: &str,
    source: &Path,
) -> Result<(), BuildError> {
    let bytes = fs::read(source).map_err(|error| BuildError::Io {
        path: source.to_path_buf(),
        source: error,
    })?;
    write_bytes(archive, path, &bytes)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModrinthManifest {
    format_version: u32,
    game: &'static str,
    version_id: String,
    name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    summary: String,
    files: Vec<ModrinthFile>,
    dependencies: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModrinthFile {
    path: String,
    hashes: BTreeMap<String, String>,
    env: ModrinthEnv,
    downloads: Vec<String>,
    file_size: u64,
}

#[derive(Serialize)]
struct ModrinthEnv {
    client: &'static str,
    server: &'static str,
}

fn write_modrinth<W: Write + Seek>(
    root: &Path,
    pack: &Pack,
    index: &Index,
    archive: &mut ZipWriter<W>,
    options: ExportOptions,
) -> Result<usize, BuildError> {
    archive
        .add_directory("overrides/", zip_options())
        .map_err(BuildError::Zip)?;
    let mut manifest_files = Vec::new();
    let mut archive_files = 1;
    for item in &index.files {
        let indexed_path = normalize_index_path(&item.file)?;
        let source = root.join(indexed_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !item.metafile {
            write_file(archive, &format!("overrides/{indexed_path}"), &source)?;
            archive_files += 1;
            continue;
        }
        let metadata: Mod = read_toml(&source)?;
        let destination = metadata_destination(&indexed_path, &metadata.filename)?;
        let direct =
            !options.restrict_modrinth_domains || modrinth_host_allowed(&metadata.download.url);
        if !direct {
            let bytes = download(&metadata.download.url)?;
            let prefix = match metadata.side.as_str() {
                "client" => "client-overrides",
                "server" => "server-overrides",
                _ => "overrides",
            };
            write_bytes(archive, &format!("{prefix}/{destination}"), &bytes)?;
            archive_files += 1;
            continue;
        }
        let bytes = (options.verify_hashes
            || metadata.download.size == 0
            || export_hash(&metadata, "sha1").is_none()
            || export_hash(&metadata, "sha512").is_none())
        .then(|| download(&metadata.download.url))
        .transpose()?;
        let sha1 = export_hash(&metadata, "sha1")
            .filter(|_| !options.verify_hashes)
            .map(str::to_owned)
            .unwrap_or_else(|| hash_bytes(HashFormat::Sha1, bytes.as_deref().unwrap_or_default()));
        let sha512 = export_hash(&metadata, "sha512")
            .filter(|_| !options.verify_hashes)
            .map(str::to_owned)
            .unwrap_or_else(|| {
                hash_bytes(HashFormat::Sha512, bytes.as_deref().unwrap_or_default())
            });
        let size = bytes
            .as_ref()
            .map(|value| value.len() as u64)
            .unwrap_or(metadata.download.size);
        let installed = if metadata
            .option
            .as_ref()
            .is_some_and(|option| option.optional)
        {
            "optional"
        } else {
            "required"
        };
        let env = match metadata.side.as_str() {
            "client" => ModrinthEnv {
                client: installed,
                server: "unsupported",
            },
            "server" => ModrinthEnv {
                client: "unsupported",
                server: installed,
            },
            _ => ModrinthEnv {
                client: installed,
                server: installed,
            },
        };
        manifest_files.push(ModrinthFile {
            path: destination,
            hashes: BTreeMap::from([("sha1".into(), sha1), ("sha512".into(), sha512)]),
            env,
            downloads: vec![metadata.download.url],
            file_size: size,
        });
    }
    manifest_files.sort_by(|left, right| left.path.cmp(&right.path));
    let mut dependencies = BTreeMap::new();
    if let Some(version) = pack.versions.get("minecraft") {
        dependencies.insert("minecraft".into(), version.clone());
    }
    for (source, target) in [
        ("quilt", "quilt-loader"),
        ("fabric", "fabric-loader"),
        ("forge", "forge"),
        ("neoforge", "neoforge"),
    ] {
        if let Some(version) = pack.versions.get(source) {
            dependencies.insert(target.into(), version.clone());
            break;
        }
    }
    let manifest = ModrinthManifest {
        format_version: 1,
        game: "minecraft",
        version_id: pack.version.clone(),
        name: pack.name.clone(),
        summary: pack.description.clone(),
        files: manifest_files,
        dependencies,
    };
    let mut json = serde_json::to_vec_pretty(&manifest).map_err(BuildError::Json)?;
    json.push(b'\n');
    write_bytes(archive, "modrinth.index.json", &json)?;
    Ok(archive_files + 1)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeManifest {
    minecraft: CurseForgeMinecraft,
    manifest_type: &'static str,
    manifest_version: u32,
    name: String,
    version: String,
    author: String,
    #[serde(rename = "projectID")]
    project_id: u64,
    files: Vec<CurseForgeFile>,
    overrides: &'static str,
}

#[derive(Serialize)]
struct CurseForgeMinecraft {
    version: String,
    #[serde(rename = "modLoaders")]
    mod_loaders: Vec<CurseForgeLoader>,
}

#[derive(Serialize)]
struct CurseForgeLoader {
    id: String,
    primary: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeFile {
    #[serde(rename = "projectID")]
    project_id: i64,
    #[serde(rename = "fileID")]
    file_id: i64,
    required: bool,
}

fn write_curseforge<W: Write + Seek>(
    root: &Path,
    pack: &Pack,
    index: &Index,
    archive: &mut ZipWriter<W>,
) -> Result<usize, BuildError> {
    archive
        .add_directory("overrides/", zip_options())
        .map_err(BuildError::Zip)?;
    let mut references = Vec::new();
    let mut mod_names = Vec::new();
    let mut archive_files = 1;
    for item in &index.files {
        let indexed_path = normalize_index_path(&item.file)?;
        let source = root.join(indexed_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !item.metafile {
            write_file(archive, &format!("overrides/{indexed_path}"), &source)?;
            archive_files += 1;
            continue;
        }
        let metadata: Mod = read_toml(&source)?;
        mod_names.push(metadata.name.clone());
        let cf = metadata.update.get("curseforge");
        let ids = cf.and_then(|table| {
            Some((
                table.get("project-id")?.as_integer()?,
                table.get("file-id")?.as_integer()?,
            ))
        });
        if let Some((project_id, file_id)) = ids {
            references.push(CurseForgeFile {
                project_id,
                file_id,
                required: !metadata
                    .option
                    .as_ref()
                    .is_some_and(|option| option.optional && !option.default),
            });
        } else {
            let destination = metadata_destination(&indexed_path, &metadata.filename)?;
            let bytes = download(&metadata.download.url)?;
            write_bytes(archive, &format!("overrides/{destination}"), &bytes)?;
            archive_files += 1;
        }
    }
    references.sort_by_key(|entry| (entry.project_id, entry.file_id));
    mod_names.sort_by_key(|name| name.to_lowercase());
    let mod_loaders = ["fabric", "forge", "neoforge", "quilt"]
        .into_iter()
        .find_map(|loader| {
            pack.versions.get(loader).map(|version| CurseForgeLoader {
                id: format!("{loader}-{version}"),
                primary: true,
            })
        })
        .into_iter()
        .collect();
    let project_id = pack
        .export
        .get("curseforge")
        .and_then(|table| table.get("project-id"))
        .and_then(toml::Value::as_integer)
        .unwrap_or(0) as u64;
    let manifest = CurseForgeManifest {
        minecraft: CurseForgeMinecraft {
            version: pack.versions.get("minecraft").cloned().unwrap_or_default(),
            mod_loaders,
        },
        manifest_type: "minecraftModpack",
        manifest_version: 1,
        name: pack.name.clone(),
        version: pack.version.clone(),
        author: pack.author.clone(),
        project_id,
        files: references,
        overrides: "overrides",
    };
    let mut json = serde_json::to_vec_pretty(&manifest).map_err(BuildError::Json)?;
    json.push(b'\n');
    write_bytes(archive, "manifest.json", &json)?;
    let mut html = String::from("<ul>\r\n");
    for name in mod_names {
        html.push_str(&format!("<li>{}</li>\r\n", html_escape(&name)));
    }
    html.push_str("</ul>\r\n");
    write_bytes(archive, "modlist.html", html.as_bytes())?;
    Ok(archive_files + 2)
}

fn normalize_index_path(value: &str) -> Result<String, BuildError> {
    let normalized = value.replace('\\', "/");
    validate_archive_path(&normalized)?;
    Ok(normalized)
}

fn validate_archive_path(value: &str) -> Result<(), BuildError> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(BuildError::UnsafePath(value.into()));
    }
    Ok(())
}

fn metadata_destination(metadata_path: &str, filename: &str) -> Result<String, BuildError> {
    if filename.is_empty() {
        return Err(BuildError::InvalidMetadata(format!(
            "{metadata_path} has no filename"
        )));
    }
    let parent = metadata_path.rsplit_once('/').map(|(parent, _)| parent);
    let value = parent.map_or_else(
        || filename.to_owned(),
        |parent| format!("{parent}/{filename}"),
    );
    validate_archive_path(&value)?;
    Ok(value)
}

fn export_hash<'a>(metadata: &'a Mod, format: &str) -> Option<&'a str> {
    if metadata.download.hash_format == format && !metadata.download.hash.is_empty() {
        Some(&metadata.download.hash)
    } else {
        metadata
            .download
            .extra_hashes
            .get(format)
            .map(String::as_str)
    }
}

fn modrinth_host_allowed(raw_url: &str) -> bool {
    url::Url::parse(raw_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .is_some_and(|host| {
            matches!(
                host.as_str(),
                "cdn.modrinth.com" | "github.com" | "raw.githubusercontent.com" | "gitlab.com"
            )
        })
}

fn download(raw_url: &str) -> Result<Vec<u8>, BuildError> {
    if raw_url.is_empty() {
        return Err(BuildError::InvalidMetadata("download URL is empty".into()));
    }
    let response = ureq::get(raw_url)
        .call()
        .map_err(|error| BuildError::Download {
            url: raw_url.into(),
            message: error.to_string(),
        })?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|error| BuildError::Download {
            url: raw_url.into(),
            message: error.to_string(),
        })?;
    Ok(bytes)
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn read_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, BuildError> {
    let source = fs::read_to_string(path).map_err(|source| BuildError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&source).map_err(|source| BuildError::Toml {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to decode TOML at {path}: {source}")]
    Toml {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("invalid pack: {0}")]
    InvalidPack(String),
    #[error("invalid external metadata: {0}")]
    InvalidMetadata(String),
    #[error("unsafe archive path: {0}")]
    UnsafePath(String),
    #[error("failed to download {url}: {message}")]
    Download { url: String, message: String },
    #[error("failed to encode JSON: {0}")]
    Json(serde_json::Error),
    #[error("archive error: {0}")]
    Zip(zip::result::ZipError),
    #[error("failed to encode TOML: {0}")]
    TomlEncode(toml::ser::Error),
    #[error("destination already exists: {0}")]
    DestinationExists(PathBuf),
    #[error("unsupported archive: {0}")]
    UnsupportedArchive(String),
    #[error("could not resolve CurseForge project {project_id} file {file_id}: {message}")]
    ProviderResolution {
        project_id: i64,
        file_id: i64,
        message: String,
    },
    #[error("external tool {program} failed: {message}")]
    ExternalTool { program: PathBuf, message: String },
    #[error("optimizer failed: {0}")]
    Optimizer(String),
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use super::{
        ExportFormat, ExportOptions, export_pack, import_curseforge_archive,
        import_modrinth_archive, plan_export,
    };

    fn sample_pack() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(directory.path().join("mods")).unwrap();
        std::fs::create_dir_all(directory.path().join("config")).unwrap();
        std::fs::write(
            directory.path().join("pack.toml"),
            "name = \"Example Pack\"\nauthor = \"Packwand\"\nversion = \"1.0\"\ndescription = \"Example\"\npack-format = \"packwand:26\"\n[index]\nfile = \"index.toml\"\n[versions]\nminecraft = \"1.21.1\"\nfabric = \"0.16.10\"\n",
        )
        .unwrap();
        std::fs::write(
            directory.path().join("index.toml"),
            "hash-format = \"sha512\"\n[[files]]\nfile = \"config/example.json\"\nhash = \"a\"\n\n[[files]]\nfile = \"mods/a.pw.toml\"\nhash = \"b\"\nmetafile = true\n",
        )
        .unwrap();
        std::fs::write(directory.path().join("config/example.json"), b"{}\n").unwrap();
        std::fs::write(
            directory.path().join("mods/a.pw.toml"),
            "name = \"Example Mod\"\nfilename = \"example.jar\"\nside = \"both\"\n\n[download]\nurl = \"https://cdn.modrinth.com/data/example.jar\"\nhash-format = \"sha512\"\nhash = \"def\"\nsize = 3\n\n[download.extra-hashes]\nsha1 = \"abc\"\n\n[update.curseforge]\nproject-id = 10\nfile-id = 20\n",
        )
        .unwrap();
        directory
    }

    #[test]
    fn plans_an_export_without_writing_files() {
        let directory = sample_pack();
        let plan = plan_export(directory.path()).unwrap();
        assert_eq!(plan.output_stem, "Example-Pack-1.0");
        assert_eq!(plan.indexed_files, 2);
        assert_eq!(plan.metadata_files, 1);
    }

    #[test]
    fn imports_curseforge_manifest_and_overrides() {
        let root = tempfile::tempdir().unwrap();
        let archive_path = root.path().join("input.zip");
        let file = std::fs::File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file("manifest.json", super::zip_options())
            .unwrap();
        archive
            .write_all(
                br#"{
  "minecraft": {
    "version": "1.21.1",
    "modLoaders": [{"id": "fabric-0.16.10", "primary": true}]
  },
  "manifestType": "minecraftModpack",
  "manifestVersion": 1,
  "name": "Imported CF",
  "version": "2.0",
  "author": "Packwand",
  "files": [{"projectID": 10, "fileID": 20, "required": false}],
  "overrides": "overrides"
}"#,
            )
            .unwrap();
        archive
            .start_file("overrides/config/example.json", super::zip_options())
            .unwrap();
        archive.write_all(b"{}\n").unwrap();
        archive.finish().unwrap();

        let destination = root.path().join("imported");
        let result =
            import_curseforge_archive(&archive_path, &destination, |project_id, file_id| {
                assert_eq!((project_id, file_id), (10, 20));
                let mut update = std::collections::BTreeMap::new();
                update.insert(
                    "curseforge".into(),
                    toml::Table::from_iter([
                        ("project-id".into(), project_id.into()),
                        ("file-id".into(), file_id.into()),
                    ]),
                );
                Ok((
                    "mods/example.pw.toml".into(),
                    packwand_pack::Mod {
                        name: "Example".into(),
                        filename: "example.jar".into(),
                        side: "both".into(),
                        download: packwand_pack::Download {
                            hash_format: "sha1".into(),
                            hash: "abc".into(),
                            mode: "metadata:curseforge".into(),
                            ..packwand_pack::Download::default()
                        },
                        update,
                        ..packwand_pack::Mod::default()
                    },
                ))
            })
            .unwrap();
        assert_eq!(result.loader.as_deref(), Some("fabric"));
        assert_eq!(result.metadata_files, 1);
        assert!(destination.join("config/example.json").is_file());
        let metadata: packwand_pack::Mod = toml::from_str(
            &std::fs::read_to_string(destination.join("mods/example.pw.toml")).unwrap(),
        )
        .unwrap();
        assert!(
            metadata
                .option
                .is_some_and(|option| option.optional && !option.default)
        );
    }

    #[test]
    fn writes_modrinth_archive_with_manifest_and_overrides() {
        let directory = sample_pack();
        let output = directory.path().join("example.mrpack");
        let artifact = export_pack(
            directory.path(),
            ExportFormat::Modrinth,
            Some(&output),
            ExportOptions::default(),
        )
        .unwrap();
        assert_eq!(artifact.path, output);
        let file = std::fs::File::open(&artifact.path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert!(archive.by_name("overrides/config/example.json").is_ok());
        let mut manifest = String::new();
        archive
            .by_name("modrinth.index.json")
            .unwrap()
            .read_to_string(&mut manifest)
            .unwrap();
        let manifest: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        assert_eq!(manifest["formatVersion"], 1);
        assert_eq!(manifest["files"][0]["path"], "mods/example.jar");
        assert_eq!(manifest["files"][0]["hashes"]["sha1"], "abc");
    }

    #[test]
    fn writes_curseforge_ids_with_expected_wire_casing() {
        let directory = sample_pack();
        let output = directory.path().join("example.zip");
        export_pack(
            directory.path(),
            ExportFormat::CurseForge,
            Some(&output),
            ExportOptions::default(),
        )
        .unwrap();
        let file = std::fs::File::open(output).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut manifest = String::new();
        archive
            .by_name("manifest.json")
            .unwrap()
            .read_to_string(&mut manifest)
            .unwrap();
        let manifest: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        assert_eq!(manifest["manifestType"], "minecraftModpack");
        assert_eq!(manifest["files"][0]["projectID"], 10);
        assert_eq!(manifest["files"][0]["fileID"], 20);
    }

    #[test]
    fn round_trips_a_modrinth_archive_without_downloading() {
        let directory = sample_pack();
        let output = directory.path().join("example.mrpack");
        export_pack(
            directory.path(),
            ExportFormat::Modrinth,
            Some(&output),
            ExportOptions::default(),
        )
        .unwrap();
        let imported = directory.path().join("imported");
        let result = import_modrinth_archive(&output, &imported).unwrap();
        assert_eq!(result.name, "Example Pack");
        assert_eq!(result.metadata_files, 1);
        assert!(imported.join("mods/example.pw.toml").is_file());
        assert_eq!(
            std::fs::read(imported.join("config/example.json")).unwrap(),
            b"{}\n"
        );
        let pack: packwand_pack::Pack =
            toml::from_str(&std::fs::read_to_string(imported.join("pack.toml")).unwrap()).unwrap();
        assert_eq!(pack.versions["fabric"], "0.16.10");
    }
}
