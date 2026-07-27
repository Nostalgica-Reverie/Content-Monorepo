use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, State};
use walkdir::{DirEntry, WalkDir};

use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::events::emit_packs_changed;
use crate::fsutil::safe_join;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeEntry {
    pub path: String,
    pub name: String,
    pub directory: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorFileStat {
    pub file_type: u8,
    pub size: u64,
    pub ctime: u64,
    pub mtime: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorDirectoryEntry {
    pub name: String,
    pub file_type: u8,
}

fn timestamp(value: std::io::Result<SystemTime>) -> u64 {
    value
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn file_type(metadata: &fs::Metadata) -> u8 {
    if metadata.is_dir() {
        2
    } else if metadata.is_file() {
        1
    } else {
        0
    }
}

fn native_root(root: &std::path::Path) -> CommandResult<String> {
    root.to_str()
        .map(str::to_owned)
        .ok_or_else(|| SerializableError::new("invalid_path", "pack root is not valid UTF-8"))
}

fn native_error(operation: &str, error: packwandc::Error) -> SerializableError {
    SerializableError::new("native_fs", format!("{operation}: {error}"))
}
fn io_error(operation: &str, error: std::io::Error) -> SerializableError {
    let kind = match error.kind() {
        std::io::ErrorKind::NotFound => "not_found",
        std::io::ErrorKind::AlreadyExists => "already_exists",
        std::io::ErrorKind::PermissionDenied => "permission_denied",
        std::io::ErrorKind::InvalidInput | std::io::ErrorKind::InvalidData => "invalid_data",
        _ => "io",
    };
    SerializableError::new(kind, format!("{operation}: {error}"))
}

fn remove_path(path: &std::path::Path, recursive: bool) -> CommandResult<()> {
    let metadata = fs::symlink_metadata(path).map_err(|error| io_error("delete", error))?;
    if metadata.is_dir() {
        if recursive {
            fs::remove_dir_all(path).map_err(|error| io_error("delete", error))
        } else {
            fs::remove_dir(path).map_err(|error| io_error("delete", error))
        }
    } else {
        fs::remove_file(path).map_err(|error| io_error("delete", error))
    }
}

fn should_descend(entry: &DirEntry) -> bool {
    entry.depth() == 0
        || !entry.file_type().is_dir()
        || !matches!(entry.file_name().to_str(), Some(".git" | "target"))
}

#[tauri::command]
pub fn editor_tree(id: String, state: State<'_, AppState>) -> CommandResult<Vec<TreeEntry>> {
    let root = pack_root(&state.workspace()?, &id)?;
    let mut entries = Vec::new();
    for entry in WalkDir::new(&root)
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_entry(should_descend)
    {
        let entry = entry.map_err(|error| SerializableError::new("walk", error.to_string()))?;
        let relative = entry
            .path()
            .strip_prefix(&root)
            .map_err(|error| SerializableError::new("unsafe_path", error.to_string()))?
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        entries.push(TreeEntry {
            path: relative,
            name: entry.file_name().to_string_lossy().into_owned(),
            directory: entry.file_type().is_dir(),
            size: entry.metadata().map(|metadata| metadata.len()).unwrap_or(0),
        });
    }
    entries.sort_by(|left, right| {
        right
            .directory
            .cmp(&left.directory)
            .then(left.path.cmp(&right.path))
    });
    Ok(entries)
}

#[tauri::command]
pub fn editor_file_read(
    id: String,
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<String> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    let bytes = fs::read(&target)?;
    String::from_utf8(bytes).map_err(|_| {
        SerializableError::new(
            "binary_file",
            format!(
                "{} is binary and cannot be opened as text",
                target.display()
            ),
        )
    })
}

#[tauri::command]
pub fn editor_file_write(
    id: String,
    path: String,
    content: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    if !target.is_file() {
        return Err(SerializableError::new(
            "not_found",
            format!("{} is not an existing file", target.display()),
        ));
    }
    packwandc::fs_atomic_write(&native_root(&root)?, &path, content.as_bytes())
        .map_err(|error| native_error("write file", error))?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn editor_create(
    id: String,
    path: String,
    directory: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    if target.exists() {
        return Err(SerializableError::new(
            "already_exists",
            format!("{} already exists", target.display()),
        ));
    }
    if directory {
        fs::create_dir_all(target)?;
    } else {
        packwandc::fs_atomic_write(&native_root(&root)?, &path, b"")
            .map_err(|error| native_error("create file", error))?;
    }
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn editor_fs_stat(
    id: String,
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<EditorFileStat> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    let metadata = fs::metadata(&target).map_err(|error| io_error("stat", error))?;
    Ok(EditorFileStat {
        file_type: file_type(&metadata),
        size: metadata.len(),
        ctime: timestamp(metadata.created()),
        mtime: timestamp(metadata.modified()),
    })
}

#[tauri::command]
pub fn editor_fs_read_dir(
    id: String,
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<EditorDirectoryEntry>> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    let mut entries = fs::read_dir(&target)
        .map_err(|error| io_error("read directory", error))?
        .map(|entry| {
            let entry = entry.map_err(|error| io_error("read directory", error))?;
            let metadata = entry
                .metadata()
                .map_err(|error| io_error("read directory", error))?;
            Ok(EditorDirectoryEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                file_type: file_type(&metadata),
            })
        })
        .collect::<CommandResult<Vec<_>>>()?;
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(entries)
}

#[tauri::command]
pub fn editor_fs_read_file(
    id: String,
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<u8>> {
    let root = pack_root(&state.workspace()?, &id)?;
    safe_join(&root, &path)?;
    packwandc::fs_read(&native_root(&root)?, &path)
        .map_err(|error| native_error("read file", error))
}

#[tauri::command]
pub fn editor_fs_write_file(
    id: String,
    path: String,
    content: Vec<u8>,
    create: bool,
    overwrite: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    let exists = target.exists();
    if exists && target.is_dir() {
        return Err(SerializableError::new(
            "is_directory",
            format!("{} is a directory", target.display()),
        ));
    }
    if !exists && !create {
        return Err(SerializableError::new(
            "not_found",
            format!("{} does not exist", target.display()),
        ));
    }
    if exists && !overwrite {
        return Err(SerializableError::new(
            "already_exists",
            format!("{} already exists", target.display()),
        ));
    }
    packwandc::fs_atomic_write(&native_root(&root)?, &path, &content)
        .map_err(|error| native_error("write file", error))?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn editor_fs_create_dir(
    id: String,
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    fs::create_dir(&target).map_err(|error| io_error("create directory", error))?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn editor_fs_delete(
    id: String,
    path: String,
    recursive: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    if path.is_empty() {
        return Err(SerializableError::new(
            "unsafe_path",
            "the pack root cannot be deleted",
        ));
    }
    let root = pack_root(&state.workspace()?, &id)?;
    let target = safe_join(&root, &path)?;
    remove_path(&target, recursive)?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn editor_fs_rename(
    id: String,
    from: String,
    to: String,
    overwrite: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    if from.is_empty() || to.is_empty() {
        return Err(SerializableError::new(
            "unsafe_path",
            "the pack root cannot be renamed or replaced",
        ));
    }
    let root = pack_root(&state.workspace()?, &id)?;
    let source = safe_join(&root, &from)?;
    let target = safe_join(&root, &to)?;
    if target.exists() {
        if !overwrite {
            return Err(SerializableError::new(
                "already_exists",
                format!("{} already exists", target.display()),
            ));
        }
        remove_path(&target, true)?;
    }
    fs::rename(&source, &target).map_err(|error| io_error("rename", error))?;
    emit_packs_changed(&app)
}
