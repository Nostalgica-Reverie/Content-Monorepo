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
            .map_err(|_| SerializableError::new("state", "settings lock was poisoned"))? =
            settings.clone();
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
}
