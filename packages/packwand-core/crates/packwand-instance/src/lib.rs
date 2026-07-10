//! Versioned instance records and a filesystem-backed repository.
//!
//! Part of the `packwand-rs` shared core (see `packwandrs.md`). This crate
//! must stay free of Tauri, clap, and axum dependencies: the Tauri adapter,
//! the probe CLI, and tests all consume the same library API.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Highest instance-record schema version this build can read and write.
pub const SCHEMA_VERSION: u32 = 1;

/// JVM memory limits in mebibytes. Absent values mean "let the JVM decide".
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryLimits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_mb: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_mb: Option<u32>,
}

/// Input for creating an instance (the `--spec` fixture file).
///
/// Unknown fields are rejected so fixture typos fail loudly instead of
/// silently producing a different instance.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstanceSpec {
    pub id: String,
    pub name: String,
    pub java_executable: PathBuf,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    pub main_class: String,
    #[serde(default)]
    pub classpath: Vec<PathBuf>,
    #[serde(default)]
    pub game_args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub memory: MemoryLimits,
    /// Names of account/session values that a future auth subsystem will
    /// resolve at launch time. Only names are ever stored, never values.
    #[serde(default)]
    pub session_placeholders: Vec<String>,
}

/// A stored, versioned instance record.
///
/// Unknown fields are tolerated on read so that same-version records written
/// by a slightly newer build do not become unreadable; genuinely newer
/// schemas are rejected via `schema_version` before full decoding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstanceRecord {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub java_executable: PathBuf,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    pub main_class: String,
    #[serde(default)]
    pub classpath: Vec<PathBuf>,
    #[serde(default)]
    pub game_args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub memory: MemoryLimits,
    #[serde(default)]
    pub session_placeholders: Vec<String>,
}

impl InstanceRecord {
    /// Builds the record `create` would persist for `spec`, without
    /// persisting it — lets callers upsert (fall back to `update` on
    /// `AlreadyExists`) without duplicating this field mapping.
    pub fn from_spec(spec: &InstanceSpec) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: spec.id.clone(),
            name: spec.name.clone(),
            java_executable: spec.java_executable.clone(),
            jvm_args: spec.jvm_args.clone(),
            main_class: spec.main_class.clone(),
            classpath: spec.classpath.clone(),
            game_args: spec.game_args.clone(),
            env: spec.env.clone(),
            memory: spec.memory.clone(),
            session_placeholders: spec.session_placeholders.clone(),
        }
    }
}

/// Filesystem locations belonging to one instance, derived from the
/// repository root. `assets` and `libraries` are shared across instances,
/// matching common launcher layouts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstancePaths {
    pub game_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub natives_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub libraries_dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum InstanceError {
    #[error("instance id {0:?} is invalid: ids must be non-empty and use only ASCII letters, digits, '.', '-' and '_'")]
    InvalidId(String),
    #[error("instance {0:?} already exists")]
    AlreadyExists(String),
    #[error("instance {0:?} was not found")]
    NotFound(String),
    #[error("instance record {path} is corrupt: {source}")]
    Corrupt {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("instance record {path} has schema version {found}, but this build supports up to {supported}")]
    UnsupportedSchemaVersion {
        path: PathBuf,
        found: u32,
        supported: u32,
    },
    #[error("I/O error at {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
}

/// One `list` result. Corrupt or future-version records are reported as
/// error entries instead of failing the whole listing.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ListEntry {
    Ok {
        id: String,
        record: Box<InstanceRecord>,
    },
    Error {
        id: String,
        error: String,
    },
}

impl ListEntry {
    /// Returns the instance ID for this entry, whether OK or Error.
    pub fn id(&self) -> &str {
        match self {
            ListEntry::Ok { id, .. } | ListEntry::Error { id, .. } => id,
        }
    }
}

/// Storage interface for instance records.
pub trait InstanceRepository {
    fn create(&self, spec: &InstanceSpec) -> Result<InstanceRecord, InstanceError>;
    fn get(&self, id: &str) -> Result<InstanceRecord, InstanceError>;
    fn list(&self) -> Result<Vec<ListEntry>, InstanceError>;
    /// Overwrites an existing instance record in place. Unlike `create`, this
    /// does not fail if the record already exists — it's for re-baking
    /// identity-bound fields (e.g. a different signed-in account) onto an
    /// already-installed instance without re-running installation.
    fn update(&self, id: &str, record: &InstanceRecord) -> Result<(), InstanceError>;
}

/// Repository storing each instance as `<root>/instances/<id>/instance.json`.
#[derive(Debug, Clone)]
pub struct FsInstanceRepository {
    root: PathBuf,
}

impl FsInstanceRepository {
    /// Creates a new repository at the given root directory.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Returns the repository root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn instances_dir(&self) -> PathBuf {
        self.root.join("instances")
    }

    fn record_path(&self, id: &str) -> PathBuf {
        self.instances_dir().join(id).join("instance.json")
    }

    /// Filesystem locations for one instance. The id is not required to
    /// exist yet; paths are derived purely from the repository layout.
    pub fn instance_paths(&self, id: &str) -> InstancePaths {
        let game_dir = self.instances_dir().join(id);
        InstancePaths {
            logs_dir: game_dir.join("logs"),
            natives_dir: game_dir.join("natives"),
            assets_dir: self.root.join("assets"),
            libraries_dir: self.root.join("libraries"),
            game_dir,
        }
    }
}

fn validate_id(id: &str) -> Result<(), InstanceError> {
    let valid = !id.is_empty()
        && id != "."
        && id != ".."
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    if valid {
        Ok(())
    } else {
        Err(InstanceError::InvalidId(id.to_string()))
    }
}

/// Writes via a temp file in the same directory plus rename, so an
/// interrupted write never leaves a half-written `instance.json`.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), InstanceError> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes).map_err(|source| InstanceError::Io {
        path: tmp.clone(),
        source,
    })?;
    fs::rename(&tmp, path).map_err(|source| InstanceError::Io {
        path: path.to_path_buf(),
        source,
    })
}

impl InstanceRepository for FsInstanceRepository {
    fn create(&self, spec: &InstanceSpec) -> Result<InstanceRecord, InstanceError> {
        validate_id(&spec.id)?;
        let paths = self.instance_paths(&spec.id);
        fs::create_dir_all(&paths.game_dir).map_err(|source| InstanceError::Io {
            path: paths.game_dir.clone(),
            source,
        })?;
        let record_path = self.record_path(&spec.id);
        if record_path.exists() {
            return Err(InstanceError::AlreadyExists(spec.id.clone()));
        }
        let record = InstanceRecord::from_spec(spec);
        let bytes =
            serde_json::to_vec_pretty(&record).map_err(|source| InstanceError::Corrupt {
                path: record_path.clone(),
                source,
            })?;
        write_atomic(&record_path, &bytes)?;
        Ok(record)
    }

    fn update(&self, id: &str, record: &InstanceRecord) -> Result<(), InstanceError> {
        validate_id(id)?;
        let record_path = self.record_path(id);
        let bytes = serde_json::to_vec_pretty(record).map_err(|source| InstanceError::Corrupt {
            path: record_path.clone(),
            source,
        })?;
        write_atomic(&record_path, &bytes)
    }

    fn get(&self, id: &str) -> Result<InstanceRecord, InstanceError> {
        validate_id(id)?;
        let path = self.record_path(id);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(InstanceError::NotFound(id.to_string()))
            }
            Err(source) => return Err(InstanceError::Io { path, source }),
        };
        #[derive(Deserialize)]
        struct VersionProbe {
            schema_version: u32,
        }
        let probe: VersionProbe =
            serde_json::from_slice(&bytes).map_err(|source| InstanceError::Corrupt {
                path: path.clone(),
                source,
            })?;
        if probe.schema_version > SCHEMA_VERSION {
            return Err(InstanceError::UnsupportedSchemaVersion {
                path,
                found: probe.schema_version,
                supported: SCHEMA_VERSION,
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| InstanceError::Corrupt { path, source })
    }

    fn list(&self) -> Result<Vec<ListEntry>, InstanceError> {
        let dir = self.instances_dir();
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => return Err(InstanceError::Io { path: dir, source }),
        };
        let mut ids = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| InstanceError::Io {
                path: dir.clone(),
                source,
            })?;
            let is_dir = entry
                .file_type()
                .map_err(|source| InstanceError::Io {
                    path: entry.path(),
                    source,
                })?
                .is_dir();
            if is_dir {
                ids.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        ids.sort();
        Ok(ids
            .into_iter()
            .map(|id| match self.get(&id) {
                Ok(record) => ListEntry::Ok {
                    id,
                    record: Box::new(record),
                },
                Err(e) => ListEntry::Error {
                    id,
                    error: e.to_string(),
                },
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::validate_id;

    #[test]
    fn id_validation() {
        assert!(validate_id("vanilla-1.21_test").is_ok());
        for bad in ["", ".", "..", "a/b", "a\\b", "a:b", "sp ace", "ünï"] {
            assert!(validate_id(bad).is_err(), "expected {bad:?} to be rejected");
        }
    }
}
