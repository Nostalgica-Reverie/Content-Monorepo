use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::error::{CommandResult, SerializableError};

pub fn safe_join(root: &Path, relative: &str) -> CommandResult<PathBuf> {
    packwand_platform::validate_relative_path(relative).map_err(|error| {
        SerializableError::new(
            "unsafe_path",
            format!("path {relative:?} escapes its configured root: {error}"),
        )
    })?;
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
    packwand_platform::atomic_write(path, bytes)
        .map_err(|error| SerializableError::new("io", error.to_string()))
}
