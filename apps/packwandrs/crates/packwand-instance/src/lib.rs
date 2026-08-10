//! Versioned instance records and a filesystem-backed repository.
//!
//! Part of the shared Packwand core. This crate
//! must stay free of Tauri, clap, and axum dependencies: the Tauri adapter,
//! the probe CLI, and tests all consume the same library API.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Highest instance-record schema version this build can read and write.
/// Bumped to 2 when account identity stopped being baked into `game_args`.
/// A version-1 record carries one account's name inside arguments that a
/// shared install hands to every account, so it must be rebuilt rather than
/// reused.
pub const SCHEMA_VERSION: u32 = 2;

/// Highest user-owned instance schema this build understands.
pub const USER_INSTANCE_SCHEMA_VERSION: u32 = 1;

/// A user instance either follows a workspace pack or owns a hidden pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
	tag = "kind",
	rename_all = "snake_case",
	rename_all_fields = "camelCase"
)]
pub enum InstanceSource {
	Linked { pack_dir: PathBuf },
	Owned,
}

/// Installation state surfaced by the launcher UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum InstallStage {
	#[default]
	NotInstalled,
	Installing,
	Ready,
	Failed {
		message: String,
	},
}

/// Editable per-instance launch settings. `None` always means inherit.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSettings {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub java_path: Option<PathBuf>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub memory_min_mb: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub memory_max_mb: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub extra_jvm_args: Option<Vec<String>>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub extra_game_args: Option<Vec<String>>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub env: Option<BTreeMap<String, String>>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub window_width: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub window_height: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub fullscreen: Option<bool>,
	/// How many files to fetch at once while installing this instance.
	/// `0` means "decide from the machine"; the useful reason to set it is a
	/// metered or shaky connection, where fewer concurrent transfers is the
	/// point rather than a performance tradeoff.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub download_jobs: Option<usize>,
}

impl InstanceSettings {
	/// Resolve inherited values without losing which values were explicitly set.
	pub fn merged(&self, defaults: &Self) -> Self {
		Self {
			java_path: self
				.java_path
				.clone()
				.or_else(|| defaults.java_path.clone()),
			memory_min_mb: self.memory_min_mb.or(defaults.memory_min_mb),
			memory_max_mb: self.memory_max_mb.or(defaults.memory_max_mb),
			extra_jvm_args: self
				.extra_jvm_args
				.clone()
				.or_else(|| defaults.extra_jvm_args.clone()),
			extra_game_args: self
				.extra_game_args
				.clone()
				.or_else(|| defaults.extra_game_args.clone()),
			env: self.env.clone().or_else(|| defaults.env.clone()),
			window_width: self.window_width.or(defaults.window_width),
			window_height: self.window_height.or(defaults.window_height),
			fullscreen: self.fullscreen.or(defaults.fullscreen),
			download_jobs: self.download_jobs.or(defaults.download_jobs),
		}
	}
}

/// The durable document for an instance the user owns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
	pub schema_version: u32,
	pub id: String,
	pub name: String,
	pub source: InstanceSource,
	pub game_version: String,
	pub loader: String,
	pub loader_version: Option<String>,
	#[serde(default)]
	pub stage: InstallStage,
	#[serde(default)]
	pub settings: InstanceSettings,
	pub created_ms: u64,
	pub last_played_ms: Option<u64>,
	pub icon: Option<String>,
	pub group: Option<String>,
}

impl Instance {
	pub fn new(
		id: String,
		name: String,
		source: InstanceSource,
		game_version: String,
		loader: String,
		loader_version: Option<String>,
		created_ms: u64,
	) -> Self {
		Self {
			schema_version: USER_INSTANCE_SCHEMA_VERSION,
			id,
			name,
			source,
			game_version,
			loader,
			loader_version,
			stage: InstallStage::NotInstalled,
			settings: InstanceSettings::default(),
			created_ms,
			last_played_ms: None,
			icon: None,
			group: None,
		}
	}
}

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
	/// Names of secret account values resolved at launch time. Only names are
	/// ever stored, never values.
	#[serde(default)]
	pub session_placeholders: Vec<String>,
	/// Names of non-secret account values — player name, uuid, user type —
	/// also resolved at launch. Kept out of the record so one managed install
	/// serves every account on that version.
	#[serde(default)]
	pub identity_placeholders: Vec<String>,
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
	#[serde(default)]
	pub identity_placeholders: Vec<String>,
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
			identity_placeholders: spec.identity_placeholders.clone(),
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
	#[error(
		"instance id {0:?} is invalid: ids must be non-empty and use only ASCII letters, digits, '.', '-' and '_'"
	)]
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
	#[error(
		"instance record {path} has schema version {found}, but this build supports up to {supported}"
	)]
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
				return Err(InstanceError::NotFound(id.to_string()));
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

/// Repository for user-owned instances at `<app-data>/instances/<id>`.
#[derive(Debug, Clone)]
pub struct FsUserInstanceRepository {
	root: PathBuf,
}

impl FsUserInstanceRepository {
	pub fn new(root: PathBuf) -> Self {
		Self { root }
	}

	pub fn root(&self) -> &Path {
		&self.root
	}

	pub fn instances_dir(&self) -> PathBuf {
		self.root.join("instances")
	}

	pub fn instance_dir(&self, id: &str) -> Result<PathBuf, InstanceError> {
		validate_id(id)?;
		Ok(self.instances_dir().join(id))
	}

	pub fn owned_pack_dir(&self, id: &str) -> Result<PathBuf, InstanceError> {
		Ok(self.instance_dir(id)?.join(".pack"))
	}

	fn record_path(&self, id: &str) -> Result<PathBuf, InstanceError> {
		Ok(self.instance_dir(id)?.join("instance.json"))
	}

	pub fn create(&self, instance: &Instance) -> Result<(), InstanceError> {
		validate_id(&instance.id)?;
		let path = self.record_path(&instance.id)?;
		if path.exists() {
			return Err(InstanceError::AlreadyExists(instance.id.clone()));
		}
		let directory = self.instance_dir(&instance.id)?;
		fs::create_dir_all(&directory).map_err(|source| InstanceError::Io {
			path: directory,
			source,
		})?;
		self.write(instance)
	}

	pub fn write(&self, instance: &Instance) -> Result<(), InstanceError> {
		validate_id(&instance.id)?;
		let path = self.record_path(&instance.id)?;
		if !path.parent().is_some_and(Path::is_dir) {
			return Err(InstanceError::NotFound(instance.id.clone()));
		}
		let bytes =
			serde_json::to_vec_pretty(instance).map_err(|source| InstanceError::Corrupt {
				path: path.clone(),
				source,
			})?;
		write_atomic(&path, &bytes)
	}

	pub fn get(&self, id: &str) -> Result<Instance, InstanceError> {
		let path = self.record_path(id)?;
		let bytes = match fs::read(&path) {
			Ok(bytes) => bytes,
			Err(error) if error.kind() == io::ErrorKind::NotFound => {
				return Err(InstanceError::NotFound(id.to_owned()));
			}
			Err(source) => return Err(InstanceError::Io { path, source }),
		};
		#[derive(Deserialize)]
		struct VersionProbe {
			#[serde(rename = "schemaVersion")]
			schema_version: u32,
		}
		let version: VersionProbe =
			serde_json::from_slice(&bytes).map_err(|source| InstanceError::Corrupt {
				path: path.clone(),
				source,
			})?;
		if version.schema_version > USER_INSTANCE_SCHEMA_VERSION {
			return Err(InstanceError::UnsupportedSchemaVersion {
				path,
				found: version.schema_version,
				supported: USER_INSTANCE_SCHEMA_VERSION,
			});
		}
		serde_json::from_slice(&bytes).map_err(|source| InstanceError::Corrupt { path, source })
	}

	pub fn list(&self) -> Result<Vec<Instance>, InstanceError> {
		let directory = self.instances_dir();
		let entries = match fs::read_dir(&directory) {
			Ok(entries) => entries,
			Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
			Err(source) => {
				return Err(InstanceError::Io {
					path: directory,
					source,
				});
			}
		};
		let mut instances = Vec::new();
		for entry in entries {
			let entry = entry.map_err(|source| InstanceError::Io {
				path: directory.clone(),
				source,
			})?;
			if entry.path().join("instance.json").is_file() {
				let id = entry.file_name().to_string_lossy().into_owned();
				instances.push(self.get(&id)?);
			}
		}
		instances.sort_by(|left, right| {
			right
				.last_played_ms
				.cmp(&left.last_played_ms)
				.then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
		});
		Ok(instances)
	}

	/// Remove the record. Game files are only removed after explicit opt-in.
	pub fn delete(&self, id: &str, delete_files: bool) -> Result<(), InstanceError> {
		let record = self.record_path(id)?;
		if !record.is_file() {
			return Err(InstanceError::NotFound(id.to_owned()));
		}
		if delete_files {
			let directory = self.instance_dir(id)?;
			fs::remove_dir_all(&directory).map_err(|source| InstanceError::Io {
				path: directory,
				source,
			})
		} else {
			fs::remove_file(&record).map_err(|source| InstanceError::Io {
				path: record,
				source,
			})
		}
	}

	/// Make a stable slug and add a numeric suffix when it already exists.
	pub fn available_id(&self, name: &str) -> String {
		let mut base = name
			.trim()
			.to_ascii_lowercase()
			.chars()
			.map(|character| {
				if character.is_ascii_alphanumeric() {
					character
				} else {
					'-'
				}
			})
			.collect::<String>();
		while base.contains("--") {
			base = base.replace("--", "-");
		}
		base = base.trim_matches('-').to_owned();
		if base.is_empty() {
			base = "instance".to_owned();
		}
		let mut candidate = base.clone();
		let mut suffix = 2;
		while self.instances_dir().join(&candidate).exists() {
			candidate = format!("{base}-{suffix}");
			suffix += 1;
		}
		candidate
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn id_validation() {
		assert!(validate_id("vanilla-1.21_test").is_ok());
		for bad in ["", ".", "..", "a/b", "a\\b", "a:b", "sp ace", "ünï"] {
			assert!(validate_id(bad).is_err(), "expected {bad:?} to be rejected");
		}
	}

	#[test]
	fn settings_merge_preserves_overrides_and_inherits_unset_values() {
		let defaults = InstanceSettings {
			memory_max_mb: Some(4096),
			fullscreen: Some(false),
			..Default::default()
		};
		let explicit = InstanceSettings {
			fullscreen: Some(true),
			..Default::default()
		};
		let merged = explicit.merged(&defaults);
		assert_eq!(merged.memory_max_mb, Some(4096));
		assert_eq!(merged.fullscreen, Some(true));
	}

	#[test]
	fn slugging_handles_empty_names_and_collisions() {
		let root = tempfile::tempdir().unwrap();
		let repo = FsUserInstanceRepository::new(root.path().to_path_buf());
		assert_eq!(repo.available_id("Hello, World!"), "hello-world");
		let instance = Instance::new(
			"hello-world".into(),
			"Hello".into(),
			InstanceSource::Owned,
			"1.21.1".into(),
			"vanilla".into(),
			None,
			0,
		);
		repo.create(&instance).unwrap();
		assert_eq!(repo.available_id("Hello World"), "hello-world-2");
		assert_eq!(repo.available_id("---"), "instance");
	}
}
