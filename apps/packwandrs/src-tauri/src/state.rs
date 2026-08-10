use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::commands::instances::InstanceRegistry;
use crate::commands::jobs::JobManager;
use crate::error::{CommandResult, SerializableError};
use crate::fsutil::atomic_write;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
	pub workspace_path: Option<String>,
	#[serde(default)]
	pub java_defaults: BTreeMap<String, String>,
	#[serde(default = "default_memory")]
	pub memory_mb: u32,
	pub msa_client_id: Option<String>,
	#[serde(default)]
	pub raw_input_enabled: bool,
	#[serde(default = "default_theme_id")]
	pub theme_id: String,
	/// Collapse every UI transition regardless of the OS preference.
	///
	/// Separate from `prefers-reduced-motion` on purpose: wanting a still
	/// editor is not the same as wanting a still desktop, and the OS signal
	/// cannot be overridden per-application.
	#[serde(default)]
	pub reduce_motion: bool,
	/// The user's shell arrangement, when they have opted into rearranging it.
	///
	/// Stored opaquely: the set of regions and how they nest is the
	/// frontend's business, and pinning a schema here would mean a Rust change
	/// every time a panel is added. `None` means "the default layout", which
	/// is also what a broken or unreadable value falls back to.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub layout: Option<serde_json::Value>,
	/// Whether the shell may be rearranged at all. Off by default; a custom
	/// arrangement is explicitly unsupported.
	#[serde(default)]
	pub layout_editing: bool,
}

const fn default_memory() -> u32 {
	4096
}

fn default_theme_id() -> String {
	"builtin.packwand-dark".to_owned()
}

impl Default for AppSettings {
	fn default() -> Self {
		Self {
			workspace_path: None,
			java_defaults: BTreeMap::new(),
			memory_mb: default_memory(),
			msa_client_id: None,
			raw_input_enabled: false,
			theme_id: default_theme_id(),
			reduce_motion: false,
			layout: None,
			layout_editing: false,
		}
	}
}

pub struct AppState {
	config_dir: PathBuf,
	settings_path: PathBuf,
	settings: RwLock<AppSettings>,
	pub jobs: JobManager,
	pub instances: InstanceRegistry,
	watch: Mutex<Option<packwand_platform::WorkspaceWatchCanceller>>,
	pub collab: Mutex<Option<crate::commands::collab::CollabHandle>>,
	pub collab_identity: Mutex<crate::commands::collab::CollabIdentity>,
}

impl AppState {
	pub fn load(app: &AppHandle) -> CommandResult<Self> {
		let config_dir = app
			.path()
			.app_config_dir()
			.map_err(|error| SerializableError::new("path", error.to_string()))?;
		fs::create_dir_all(&config_dir)?;
		let settings_path = config_dir.join("settings.json");
		let settings = match fs::read_to_string(&settings_path) {
			Ok(source) => serde_json::from_str(&source)?,
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => AppSettings::default(),
			Err(error) => return Err(error.into()),
		};
		Ok(Self {
			config_dir,
			settings_path,
			settings: RwLock::new(settings),
			jobs: JobManager::default(),
			instances: InstanceRegistry::default(),
			watch: Mutex::new(None),
			collab: Mutex::new(None),
			collab_identity: Mutex::new(crate::commands::collab::CollabIdentity::default()),
		})
	}

	pub fn config_dir(&self) -> &std::path::Path {
		&self.config_dir
	}

	pub fn settings(&self) -> CommandResult<AppSettings> {
		self.settings
			.read()
			.map(|settings| settings.clone())
			.map_err(|_| SerializableError::new("state", "settings lock was poisoned"))
	}

	pub fn update_settings(&self, settings: AppSettings) -> CommandResult<AppSettings> {
		atomic_write(&self.settings_path, &serde_json::to_vec_pretty(&settings)?)?;
		*self
			.settings
			.write()
			.map_err(|_| SerializableError::new("state", "settings lock was poisoned"))? = settings.clone();
		Ok(settings)
	}

	pub fn restart_watch(&self, app: &AppHandle, root: &std::path::Path) -> CommandResult<()> {
		let watch = packwand_platform::WorkspaceWatcher::open(root)
			.map_err(|error| SerializableError::new("native_watch", error.to_string()))?;
		let canceller = watch.canceller();
		let mut active = self
			.watch
			.lock()
			.map_err(|_| SerializableError::new("state", "watch lock was poisoned"))?;
		if let Some(previous) = active.take() {
			previous.cancel();
		}
		*active = Some(canceller);
		let changed_app = app.clone();
		let workspace_root = root.to_path_buf();
		std::thread::spawn(move || {
			while let Ok(paths) = watch.read_changes() {
				if paths.is_empty() {
					continue;
				}
				std::thread::sleep(Duration::from_millis(75));
				let relative_paths = paths
					.iter()
					.map(|path| path.to_string_lossy().replace('\\', "/"))
					.collect::<Vec<_>>();
				let absolute_paths = paths
					.into_iter()
					.map(|path| {
						workspace_root
							.join(path)
							.to_string_lossy()
							.replace('\\', "/")
					})
					.collect();
				let _ = crate::events::emit_workspace_files_changed(&changed_app, absolute_paths);
				crate::commands::collab::broadcast_fs_changes(&changed_app, &relative_paths);
				let _ = crate::events::emit_packs_changed(&changed_app);
			}
		});
		Ok(())
	}
	pub fn workspace(&self) -> CommandResult<PathBuf> {
		let settings = self.settings()?;
		let path = settings.workspace_path.ok_or_else(|| {
			SerializableError::new("workspace_not_configured", "select a workspace first")
		})?;
		Ok(PathBuf::from(path))
	}

	pub fn tool_root(&self) -> PathBuf {
		self.config_dir.clone()
	}
}
