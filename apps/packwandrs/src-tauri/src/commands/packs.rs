use std::fs;
use std::path::{Component, Path, PathBuf};

use packwand_pack::{Index, Pack};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, State};
use walkdir::{DirEntry, WalkDir};

use crate::commands::off_thread;
use crate::error::{CommandResult, SerializableError};
use crate::events::emit_packs_changed;
use crate::fsutil::atomic_write;
use crate::state::AppState;

const PACK_ROOTS: [&str; 4] = ["mods", "modpacks", "datapacks", "resourcepacks"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackSummary {
    pub id: String,
    pub name: String,
    pub path: String,
    pub pack_format: String,
    pub version: String,
    pub minecraft_version: Option<String>,
    pub loaders: Vec<String>,
    pub indexed_files: usize,
    pub metadata_files: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackDetail {
    pub summary: PackSummary,
    pub pack: Pack,
    pub index: Index,
}

fn should_descend(entry: &DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return true;
    }
    !matches!(
        entry.file_name().to_str(),
        Some(".git" | "target" | "node_modules" | ".packwand-launcher")
    )
}

pub(crate) fn discover_packs(workspace: &Path) -> CommandResult<Vec<PackSummary>> {
    let mut packs = Vec::new();
    if workspace.join("pack.toml").is_file()
        && let Ok(pack) = read_pack_summary(workspace, workspace)
    {
        packs.push(pack);
    }
    for category in PACK_ROOTS {
        let category_root = workspace.join(category);
        if !category_root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(category_root)
            .follow_links(false)
            .into_iter()
            .filter_entry(should_descend)
        {
            let entry = entry.map_err(|error| SerializableError::new("walk", error.to_string()))?;
            if !entry.file_type().is_file() || entry.file_name() != "pack.toml" {
                continue;
            }
            let root = entry
                .path()
                .parent()
                .ok_or_else(|| SerializableError::new("invalid_pack", "pack.toml has no parent"))?;
            if let Ok(pack) = read_pack_summary(workspace, root) {
                packs.push(pack);
            }
        }
    }
    packs.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
    Ok(packs)
}

fn read_pack_summary(workspace: &Path, root: &Path) -> CommandResult<PackSummary> {
    let source = fs::read_to_string(root.join("pack.toml"))?;
    let pack: Pack = toml::from_str(&source)?;
    pack.format()
        .map_err(|error| SerializableError::new("invalid_pack", error.to_string()))?;
    let index: Index = match fs::read_to_string(root.join(&pack.index.file)) {
        // Generated state may be temporarily malformed while being refreshed.
        Ok(source) => serde_json::from_str(&source).unwrap_or_default(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Index::default(),
        Err(error) => return Err(error.into()),
    };
    summarize(workspace, root, &pack, &index)
}

/// Builds a summary from documents the caller has already parsed.
///
/// An index file for a large pack is tens of thousands of lines, so parsing
/// it twice to serve one request is worth avoiding — see [`packs_get`].
fn summarize(
    workspace: &Path,
    root: &Path,
    pack: &Pack,
    index: &Index,
) -> CommandResult<PackSummary> {
    let relative = root
        .strip_prefix(workspace)
        .map_err(|error| SerializableError::new("invalid_pack", error.to_string()))?;
    let id = if relative.as_os_str().is_empty() {
        ".".into()
    } else {
        relative
            .components()
            .filter_map(|component| match component {
                Component::Normal(part) => Some(part.to_string_lossy()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("/")
    };
    let minecraft_version = pack.versions.get("minecraft").cloned();
    let loaders = pack
        .versions
        .keys()
        .filter(|key| key.as_str() != "minecraft")
        .cloned()
        .collect();
    Ok(PackSummary {
        id,
        name: if pack.name.is_empty() {
            root.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unnamed pack")
                .to_string()
        } else {
            pack.name.clone()
        },
        path: root.to_string_lossy().into_owned(),
        pack_format: if pack.pack_format.is_empty() {
            "packwiz:1.1.0".into()
        } else {
            pack.pack_format.clone()
        },
        version: pack.version.clone(),
        minecraft_version,
        loaders,
        indexed_files: index.files.len(),
        metadata_files: index.files.iter().filter(|file| file.metafile).count(),
    })
}

pub(crate) fn pack_root(workspace: &Path, id: &str) -> CommandResult<PathBuf> {
    let root = if id == "." {
        workspace.to_path_buf()
    } else {
        crate::fsutil::safe_join(workspace, id)?
    };
    if !root.join("pack.toml").is_file() {
        return Err(SerializableError::new(
            "pack_not_found",
            format!("pack {id:?} was not found"),
        ));
    }
    Ok(root)
}

#[tauri::command]
pub async fn packs_list(state: State<'_, AppState>) -> CommandResult<Vec<PackSummary>> {
    let workspace = state.workspace()?;
    off_thread(move || discover_packs(&workspace)).await
}

#[tauri::command]
pub async fn packs_get(id: String, state: State<'_, AppState>) -> CommandResult<PackDetail> {
    let workspace = state.workspace()?;
    let root = pack_root(&workspace, &id)?;
    off_thread(move || {
        let pack: Pack = toml::from_str(&fs::read_to_string(root.join("pack.toml"))?)?;
        let index: Index = serde_json::from_str(&fs::read_to_string(root.join(&pack.index.file))?)?;
        // Both documents are already in hand; summarizing from them avoids
        // reading and parsing pack.toml and the index a second time.
        let summary = summarize(&workspace, &root, &pack, &index)?;
        Ok(PackDetail {
            summary,
            pack,
            index,
        })
    })
    .await
}

#[tauri::command]
pub fn packs_manifest_get(id: String, state: State<'_, AppState>) -> CommandResult<Option<Value>> {
    let root = pack_root(&state.workspace()?, &id)?;
    match fs::read_to_string(root.join("manifest.json")) {
        Ok(source) => Ok(Some(serde_json::from_str(&source)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

#[tauri::command]
pub fn packs_manifest_put(
    id: String,
    manifest: Value,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    if !manifest.is_object() {
        return Err(SerializableError::new(
            "invalid_manifest",
            "manifest must be a JSON object",
        ));
    }
    let root = pack_root(&state.workspace()?, &id)?;
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    atomic_write(&root.join("manifest.json"), &bytes)?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn packs_changelog_get(id: String, state: State<'_, AppState>) -> CommandResult<String> {
    let root = pack_root(&state.workspace()?, &id)?;
    match fs::read_to_string(root.join("changelog.md")) {
        Ok(source) => Ok(source),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error.into()),
    }
}

#[tauri::command]
pub fn packs_changelog_put(
    id: String,
    content: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    atomic_write(&root.join("changelog.md"), content.as_bytes())?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub async fn packs_icon(id: String, state: State<'_, AppState>) -> CommandResult<Option<Vec<u8>>> {
    let root = pack_root(&state.workspace()?, &id)?;
    off_thread(move || {
        for filename in ["icon.png", "icon.jpg", "icon.webp"] {
            match fs::read(root.join(filename)) {
                Ok(bytes) => return Ok(Some(bytes)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(None)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::discover_packs;

    #[test]
    fn discovers_nested_pack_roots() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("modpacks/example/1.21");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("pack.toml"),
            "name = \"Example\"\npack-format = \"packwand:26\"\n[index]\nfile = \"index.json\"\n[versions]\nminecraft = \"1.21\"\nfabric = \"0.16\"\n",
        )
        .unwrap();
        std::fs::write(root.join("index.json"), "hash-format = \"sha512\"\n").unwrap();
        let packs = discover_packs(directory.path()).unwrap();
        assert_eq!(packs.len(), 1);
        std::fs::write(root.join("index.json"), "broken").unwrap();
        assert_eq!(
            discover_packs(directory.path()).unwrap()[0].indexed_files,
            0
        );
        assert_eq!(packs[0].id, "modpacks/example/1.21");
        assert_eq!(packs[0].minecraft_version.as_deref(), Some("1.21"));
    }
}
