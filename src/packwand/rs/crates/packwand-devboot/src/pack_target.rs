//! Reads just enough of a packwiz `pack.toml` to know what to launch.
//!
//! This intentionally does not understand the rest of the packwiz format
//! (mods, index files, hashing) — that remains exclusively Go/packwiz-owned.
//! All this needs is the `[versions]` table.

use std::fs;
use std::path::Path;

use serde::Deserialize;

const KNOWN_LOADERS: [&str; 4] = ["fabric", "quilt", "forge", "neoforge"];

#[derive(Debug, thiserror::Error)]
pub enum PackTargetError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("{path} has no [versions] table")]
    MissingVersions { path: String },
    #[error("{path} has no [versions].minecraft")]
    MissingMinecraft { path: String },
}

/// The Minecraft version and (optional) loader a pack subdir targets, per
/// its packwiz `pack.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackTarget {
    pub minecraft: String,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
}

#[derive(Deserialize)]
struct PackToml {
    #[serde(default)]
    versions: Option<toml::Table>,
}

/// Resolves the launch target from a packwiz pack subdir's `pack.toml`.
pub fn resolve_pack_target(pack_toml_path: &Path) -> Result<PackTarget, PackTargetError> {
    let path_str = pack_toml_path.display().to_string();
    let contents = fs::read_to_string(pack_toml_path).map_err(|source| PackTargetError::Read {
        path: path_str.clone(),
        source,
    })?;
    let parsed: PackToml = toml::from_str(&contents).map_err(|source| PackTargetError::Parse {
        path: path_str.clone(),
        source,
    })?;
    let versions = parsed
        .versions
        .ok_or_else(|| PackTargetError::MissingVersions {
            path: path_str.clone(),
        })?;

    let minecraft = versions
        .get("minecraft")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PackTargetError::MissingMinecraft {
            path: path_str.clone(),
        })?
        .to_string();

    let mut loader = None;
    let mut loader_version = None;
    for name in KNOWN_LOADERS {
        if let Some(v) = versions.get(name).and_then(|v| v.as_str()) {
            loader = Some(name.to_string());
            loader_version = Some(v.to_string());
            break;
        }
    }

    Ok(PackTarget {
        minecraft,
        loader,
        loader_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fabric_pack() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pack.toml");
        std::fs::write(
            &path,
            r#"
name = "Test Pack"
pack-format = "packwiz:1.1.0"

[versions]
minecraft = "1.20.1"
fabric = "0.15.11"
"#,
        )
        .unwrap();
        let target = resolve_pack_target(&path).unwrap();
        assert_eq!(target.minecraft, "1.20.1");
        assert_eq!(target.loader.as_deref(), Some("fabric"));
        assert_eq!(target.loader_version.as_deref(), Some("0.15.11"));
    }

    #[test]
    fn parses_vanilla_pack() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pack.toml");
        std::fs::write(
            &path,
            r#"
[versions]
minecraft = "1.20.1"
"#,
        )
        .unwrap();
        let target = resolve_pack_target(&path).unwrap();
        assert_eq!(target.minecraft, "1.20.1");
        assert_eq!(target.loader, None);
    }

    #[test]
    fn rejects_missing_versions_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pack.toml");
        std::fs::write(&path, "name = \"Test Pack\"\n").unwrap();
        assert!(matches!(
            resolve_pack_target(&path),
            Err(PackTargetError::MissingVersions { .. })
        ));
    }
}
