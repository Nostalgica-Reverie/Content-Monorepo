use std::fs;
use std::path::{Component, Path, PathBuf};

use uuid::Uuid;

use crate::error::{CommandResult, SerializableError};

pub fn safe_join(root: &Path, relative: &str) -> CommandResult<PathBuf> {
    let path = Path::new(relative);
    if path.as_os_str().is_empty() {
        return Ok(root.to_path_buf());
    }
    let mut joined = root.to_path_buf();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                joined.push(part);
                if fs::symlink_metadata(&joined)
                    .map(|metadata| metadata.file_type().is_symlink())
                    .unwrap_or(false)
                {
                    return Err(SerializableError::new(
                        "unsafe_path",
                        format!("path {relative:?} traverses a symbolic link"),
                    ));
                }
            }
            _ => {
                return Err(SerializableError::new(
                    "unsafe_path",
                    format!("path {relative:?} escapes its configured root"),
                ));
            }
        }
    }
    Ok(joined)
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> CommandResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| SerializableError::new("invalid_path", "file has no parent directory"))?;
    fs::create_dir_all(parent)?;
    let id = Uuid::new_v4();
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("packwand");
    let temporary = parent.join(format!(".{filename}.{id}.tmp"));
    let backup = parent.join(format!(".{filename}.{id}.backup"));
    fs::write(&temporary, bytes)?;
    let had_target = path.exists();
    if had_target && let Err(error) = fs::rename(path, &backup) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if had_target {
            let _ = fs::rename(&backup, path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    if had_target {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}
