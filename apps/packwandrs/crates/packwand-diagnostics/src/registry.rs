use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use packwand_parallel::Jobs;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegistryKind {
    Datapack,
    Config,
    Resourcepack,
    Kubejs,
}

impl RegistryKind {
    pub const ALL: [Self; 4] = [
        Self::Datapack,
        Self::Config,
        Self::Resourcepack,
        Self::Kubejs,
    ];
}

impl fmt::Display for RegistryKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Datapack => "datapack",
            Self::Config => "config",
            Self::Resourcepack => "resourcepack",
            Self::Kubejs => "kubejs",
        })
    }
}

impl FromStr for RegistryKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "datapack" => Ok(Self::Datapack),
            "config" => Ok(Self::Config),
            "resourcepack" | "rp" => Ok(Self::Resourcepack),
            "kubejs" => Ok(Self::Kubejs),
            _ => Err(format!(
                "unknown registry kind {value:?} (want datapack, config, resourcepack, kubejs, or all)"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub kind: String,
    pub origin: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub owner: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub schema_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentRegistry {
    pub scope: String,
    pub kind: RegistryKind,
    pub version: String,
    pub sources: Vec<String>,
    pub entries: Vec<RegistryEntry>,
}

#[derive(Debug)]
struct Source {
    root: PathBuf,
    origin: String,
}

/// How many files are read ahead of the hasher at once. Bounds peak memory to
/// roughly one batch of file contents rather than a whole pack.
const READ_BATCH: usize = 64;

pub fn build_all_registries(
    root: impl AsRef<Path>,
) -> Result<Vec<ContentRegistry>, Box<dyn Error>> {
    build_all_registries_with(root, packwand_parallel::configured())
}

pub fn build_all_registries_with(
    root: impl AsRef<Path>,
    jobs: Jobs,
) -> Result<Vec<ContentRegistry>, Box<dyn Error>> {
    RegistryKind::ALL
        .into_iter()
        .map(|kind| build_registry_with(root.as_ref(), kind, jobs))
        .collect()
}

pub fn build_registry(
    root: impl AsRef<Path>,
    kind: RegistryKind,
) -> Result<ContentRegistry, Box<dyn Error>> {
    build_registry_with(root, kind, packwand_parallel::configured())
}

pub fn build_registry_with(
    root: impl AsRef<Path>,
    kind: RegistryKind,
    jobs: Jobs,
) -> Result<ContentRegistry, Box<dyn Error>> {
    let root = root.as_ref();
    if !root.is_dir() {
        return Err(format!("registry scope {} is not a directory", root.display()).into());
    }
    let sources = sources(root, kind)?;
    let slugs = mod_slugs(root)?;
    let mut entries = Vec::new();
    let mut hasher = Sha256::new();
    for source in &sources {
        let mut files = WalkDir::new(&source.root)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.path().to_path_buf())
            .collect::<Vec<_>>();
        files.sort();
        // The registry version is a single hash stream over the sorted files,
        // so hashing itself has to stay ordered. Only the reads are
        // parallelized, in bounded batches: the disk work overlaps while the
        // resulting hash stays byte-identical to a sequential pass, and at
        // most one batch of file contents is resident at a time.
        for batch in files.chunks(READ_BATCH) {
            let contents = packwand_parallel::try_map(batch, jobs, |path| fs::read(path));
            for (path, content) in batch.iter().zip(contents) {
                let relative = slash(path.strip_prefix(&source.root).unwrap_or(path));
                hasher.update(source.origin.as_bytes());
                hasher.update([0]);
                hasher.update(relative.as_bytes());
                hasher.update([0]);
                hasher.update(content?);
                if let Some(mut entry) = classify(kind, &relative, &source.origin, &slugs) {
                    entry.origin = source.origin.clone();
                    entries.push(entry);
                }
            }
        }
    }
    let mut source_origins: Vec<String> = sources.into_iter().map(|source| source.origin).collect();
    // The pack's mod slugs belong to the KubeJS registry because they drive
    // Platform.isLoaded completion — but only for a pack that actually has a
    // kubejs/ tree. Without one the registry is empty, rather than a bare
    // list of every mod in the pack.
    if kind == RegistryKind::Kubejs && root.join("kubejs").is_dir() && !slugs.is_empty() {
        source_origins.push("mods".into());
        for slug in &slugs {
            entries.push(RegistryEntry {
                id: slug.clone(),
                kind: "mod".into(),
                origin: "mods".into(),
                path: String::new(),
                owner: String::new(),
                schema_ref: String::new(),
            });
        }
    }
    entries.sort_by(|left, right| {
        (&left.id, &left.kind, &left.origin).cmp(&(&right.id, &right.kind, &right.origin))
    });
    for entry in &entries {
        hasher.update(entry.id.as_bytes());
        hasher.update(entry.kind.as_bytes());
        hasher.update(entry.origin.as_bytes());
        hasher.update(entry.path.as_bytes());
    }
    Ok(ContentRegistry {
        scope: slash(root),
        kind,
        version: format!("{:x}", hasher.finalize()),
        sources: source_origins,
        entries,
    })
}

fn sources(root: &Path, kind: RegistryKind) -> Result<Vec<Source>, Box<dyn Error>> {
    let packwiz = root.join("pack.toml").is_file();
    let mut output = Vec::new();
    match kind {
        RegistryKind::Datapack if packwiz => {
            child_packs(root, "global_packs/required_data", "data", &mut output)?;
            child_packs(root, "global_packs/optional_data", "data", &mut output)?;
            push_if(root, "kubejs", "kubejs", "data", &mut output);
        }
        RegistryKind::Resourcepack if packwiz => {
            child_packs(root, "resourcepacks", "assets", &mut output)?;
            child_packs(
                root,
                "global_packs/required_resources",
                "assets",
                &mut output,
            )?;
            push_if(root, "kubejs", "kubejs", "assets", &mut output);
        }
        RegistryKind::Datapack => content_roots(root, "data", &mut output)?,
        RegistryKind::Resourcepack => content_roots(root, "assets", &mut output)?,
        RegistryKind::Config => {
            push_directory(root, "config", &mut output);
            push_directory(root, "defaultconfigs", &mut output);
        }
        RegistryKind::Kubejs => {
            for folder in [
                "startup_scripts",
                "server_scripts",
                "client_scripts",
                "exported",
            ] {
                push_directory(root, &format!("kubejs/{folder}"), &mut output);
            }
        }
    }
    output.sort_by(|left, right| left.origin.cmp(&right.origin));
    Ok(output)
}

fn content_roots(root: &Path, top: &str, output: &mut Vec<Source>) -> Result<(), Box<dyn Error>> {
    if root.join(top).is_dir() || root.join("pack.mcmeta").is_file() {
        output.push(Source {
            root: root.to_path_buf(),
            origin: ".".into(),
        });
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && (entry.path().join(top).is_dir() || entry.path().join("pack.mcmeta").is_file())
        {
            output.push(Source {
                root: entry.path(),
                origin: entry.file_name().to_string_lossy().into_owned(),
            });
        }
    }
    Ok(())
}

fn child_packs(
    root: &Path,
    base: &str,
    top: &str,
    output: &mut Vec<Source>,
) -> Result<(), Box<dyn Error>> {
    let directory = root.join(base);
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && entry.path().join(top).is_dir() {
            output.push(Source {
                root: entry.path(),
                origin: format!("{base}/{}", entry.file_name().to_string_lossy()),
            });
        }
    }
    Ok(())
}

fn push_if(root: &Path, directory: &str, origin: &str, required: &str, output: &mut Vec<Source>) {
    if root.join(directory).join(required).is_dir() {
        output.push(Source {
            root: root.join(directory),
            origin: origin.into(),
        });
    }
}

fn push_directory(root: &Path, directory: &str, output: &mut Vec<Source>) {
    if root.join(directory).is_dir() {
        output.push(Source {
            root: root.join(directory),
            origin: directory.replace('\\', "/"),
        });
    }
}

fn classify(
    kind: RegistryKind,
    relative: &str,
    origin: &str,
    slugs: &[String],
) -> Option<RegistryEntry> {
    match kind {
        RegistryKind::Datapack => classify_datapack(relative),
        RegistryKind::Resourcepack => classify_resourcepack(relative),
        RegistryKind::Config => Some(RegistryEntry {
            id: format!("{origin}/{relative}"),
            kind: "config_file".into(),
            origin: String::new(),
            path: relative.to_owned(),
            owner: config_owner(relative, slugs),
            schema_ref: String::new(),
        }),
        RegistryKind::Kubejs => classify_kubejs(relative, origin),
    }
}

fn classify_datapack(path: &str) -> Option<RegistryEntry> {
    if path == "pack.mcmeta" {
        return Some(entry("pack.mcmeta", "pack_mcmeta", path));
    }
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() < 4 || parts[0] != "data" {
        return None;
    }
    let (kind, start) = match parts[2] {
        "tags" if parts.len() >= 5 => (format!("tag/{}", parts[3].trim_end_matches('s')), 4),
        "worldgen" if parts.len() >= 5 => (format!("worldgen/{}", parts[3]), 4),
        value => (
            match value {
                "function" | "functions" => "function".into(),
                "predicate" | "predicates" => "predicate".into(),
                "loot_table" | "loot_tables" => "loot_table".into(),
                "recipe" | "recipes" => "recipe".into(),
                "advancement" | "advancements" => "advancement".into(),
                "structure" | "structures" => "structure".into(),
                "item_modifier" | "item_modifiers" => "item_modifier".into(),
                other => format!("data/{other}"),
            },
            3,
        ),
    };
    let id = format!(
        "{}:{}",
        parts[1],
        strip_extension(&parts[start..].join("/"))
    );
    Some(entry(&id, &kind, path))
}

fn classify_resourcepack(path: &str) -> Option<RegistryEntry> {
    if path == "pack.mcmeta" {
        return Some(entry("pack.mcmeta", "pack_mcmeta", path));
    }
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() < 3 || parts[0] != "assets" || path.ends_with(".mcmeta") {
        return None;
    }
    let (kind, resource) = if parts.len() == 3 {
        (
            if parts[2] == "sounds.json" {
                "sound_definitions"
            } else {
                "asset"
            },
            strip_extension(parts[2]).to_owned(),
        )
    } else {
        (
            match parts[2] {
                "textures" => "texture",
                "models" => "model",
                "blockstates" => "blockstate",
                "lang" => "lang",
                "sounds" => "sound",
                "font" => "font",
                "atlases" => "atlas",
                "particles" => "particle",
                "shaders" => "shader",
                "texts" => "text",
                _ => "asset",
            },
            strip_extension(&parts[3..].join("/")).to_owned(),
        )
    };
    Some(entry(&format!("{}:{resource}", parts[1]), kind, path))
}

fn classify_kubejs(path: &str, origin: &str) -> Option<RegistryEntry> {
    let kind = match origin.rsplit('/').next().unwrap_or_default() {
        "startup_scripts" => "script/startup",
        "server_scripts" => "script/server",
        "client_scripts" => "script/client",
        "exported" => "type_dump",
        _ => return None,
    };
    (kind == "type_dump" || path.ends_with(".js") || path.ends_with(".ts"))
        .then(|| entry(path, kind, path))
}

fn entry(id: &str, kind: &str, path: &str) -> RegistryEntry {
    RegistryEntry {
        id: id.into(),
        kind: kind.into(),
        origin: String::new(),
        path: path.into(),
        owner: String::new(),
        schema_ref: String::new(),
    }
}

fn mod_slugs(root: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut slugs = Vec::new();
    if !root.join("mods").is_dir() {
        return Ok(slugs);
    }
    for entry in fs::read_dir(root.join("mods"))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type()?.is_file() && name.ends_with(".pw.toml") {
            slugs.push(name.trim_end_matches(".pw.toml").into());
        }
    }
    slugs.sort();
    Ok(slugs)
}

fn config_owner(path: &str, slugs: &[String]) -> String {
    let owners = slugs
        .iter()
        .map(|slug| (normalize(slug), slug))
        .collect::<BTreeMap<_, _>>();
    let filename = path.rsplit('/').next().unwrap_or(path);
    let stem = strip_extension(filename);
    let mut candidates = vec![stem];
    if let Some((directory, _)) = path.split_once('/') {
        candidates.push(directory);
    }
    for candidate in candidates {
        let normalized = normalize(candidate);
        for suffix in ["", "-common", "-client", "-server", "-general"] {
            if let Some(owner) = owners.get(normalized.trim_end_matches(suffix)) {
                return (*owner).clone();
            }
        }
    }
    String::new()
}

fn normalize(value: &str) -> String {
    value.to_ascii_lowercase().replace('_', "-")
}

fn strip_extension(path: &str) -> &str {
    path.rsplit_once('.').map_or(path, |(stem, _)| stem)
}

fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_datapack_and_config_ownership() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("data/example/functions")).unwrap();
        fs::write(
            root.path().join("data/example/functions/start.mcfunction"),
            "say hi",
        )
        .unwrap();
        let registry = build_registry(root.path(), RegistryKind::Datapack).unwrap();
        assert_eq!(registry.entries[0].id, "example:start");
        assert_eq!(registry.entries[0].kind, "function");

        fs::create_dir_all(root.path().join("config")).unwrap();
        fs::create_dir_all(root.path().join("mods")).unwrap();
        fs::write(root.path().join("mods/example-mod.pw.toml"), "").unwrap();
        fs::write(root.path().join("config/example-mod-client.toml"), "").unwrap();
        let registry = build_registry(root.path(), RegistryKind::Config).unwrap();
        assert_eq!(registry.entries[0].owner, "example-mod");
    }
}
