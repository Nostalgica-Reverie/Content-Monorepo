use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use walkdir::{DirEntry, WalkDir};

use crate::BuildError;

pub const PACKEATER_MARKER: &str = "packeater.json";

fn descend(entry: &DirEntry) -> bool {
    entry.depth() == 0
        || !matches!(
            entry.file_name().to_str(),
            Some(".git" | ".hg" | ".svn" | "node_modules" | "target")
        )
}

/// Find every Packeater marker below a folder in deterministic order.
pub fn discover_packeater_markers(root: impl AsRef<Path>) -> Result<Vec<PathBuf>, BuildError> {
    let root = root.as_ref();
    if !root.is_dir() {
        return Err(BuildError::InvalidPack(format!(
            "{} is not a directory",
            root.display()
        )));
    }
    let mut markers = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(descend)
        .filter_map(|entry| match entry {
            Ok(entry)
                if entry.file_type().is_file()
                    && entry.file_name().to_str() == Some(PACKEATER_MARKER) =>
            {
                Some(Ok(entry.into_path()))
            }
            Ok(_) => None,
            Err(source) => Some(Err(BuildError::InvalidPack(format!(
                "Packeater folder discovery failed: {source}"
            )))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    markers.sort();
    Ok(markers)
}

/// Run Packeater for one marker, forcing the artifact destination selected by Packwand.
pub fn run_packeater(
    marker: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<u64, BuildError> {
    let marker = marker.as_ref();
    let output = output.as_ref();
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| BuildError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let program = packeater_program();
    let result = Command::new(&program)
        .arg(marker)
        .arg("--output")
        .arg(output)
        .output()
        .map_err(|source| BuildError::ExternalTool {
            program: program.clone(),
            message: if source.kind() == std::io::ErrorKind::NotFound {
                "not found; set PACKEATER_BIN or add packeater to PATH".into()
            } else {
                source.to_string()
            },
        })?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&result.stdout).trim().to_owned();
        return Err(BuildError::ExternalTool {
            program,
            message: if stderr.is_empty() { stdout } else { stderr },
        });
    }
    fs::metadata(output)
        .map(|metadata| metadata.len())
        .map_err(|source| BuildError::Io {
            path: output.to_path_buf(),
            source,
        })
}

fn packeater_program() -> PathBuf {
    if let Some(configured) = env::var_os("PACKEATER_BIN").filter(|value| !value.is_empty()) {
        return configured.into();
    }
    let executable_name = if cfg!(windows) {
        "packeater.exe"
    } else {
        "packeater"
    };
    if let Ok(current_executable) = env::current_exe()
        && let Some(directory) = current_executable.parent()
    {
        let bundled = directory.join(executable_name);
        if bundled.is_file() {
            return bundled;
        }
    }
    if let Ok(current_directory) = env::current_dir() {
        for relative in [
            "packeater/target/release",
            "packeater/target/debug",
            "apps/packwandrs/packeater/target/release",
            "apps/packwandrs/packeater/target/debug",
        ] {
            let in_tree = current_directory.join(relative).join(executable_name);
            if in_tree.is_file() {
                return in_tree;
            }
        }
    }
    executable_name.into()
}

/// Archive a content folder, selecting Packeater whenever it opts in with a marker.
pub fn archive_content_directory(
    root: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<u64, BuildError> {
    let root = root.as_ref();
    let marker = root.join(PACKEATER_MARKER);
    if marker.is_file() {
        run_packeater(marker, output)
    } else {
        crate::archive_directory(root, output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_markers_and_skips_build_folders() {
        let root = tempfile::tempdir().unwrap();
        let marker = root.path().join("resourcepacks/example/packeater.json");
        let skipped = root.path().join("target/example/packeater.json");
        fs::create_dir_all(marker.parent().unwrap()).unwrap();
        fs::create_dir_all(skipped.parent().unwrap()).unwrap();
        fs::write(&marker, "{}").unwrap();
        fs::write(skipped, "{}").unwrap();
        assert_eq!(
            discover_packeater_markers(root.path()).unwrap(),
            vec![marker]
        );
    }
}
