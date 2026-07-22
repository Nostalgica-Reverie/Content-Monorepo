use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

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
}

const fn default_memory() -> u32 {
    4096
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            workspace_path: None,
            java_defaults: BTreeMap::new(),
            memory_mb: default_memory(),
            msa_client_id: None,
        }
    }
}

pub struct AppState {
    settings_path: PathBuf,
    settings: RwLock<AppSettings>,
    pub jobs: JobManager,
    pub instances: InstanceRegistry,
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
            settings_path,
            settings: RwLock::new(settings),
            jobs: JobManager::default(),
            instances: InstanceRegistry::default(),
        })
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

    pub fn workspace(&self) -> CommandResult<PathBuf> {
        let settings = self.settings()?;
        let path = settings.workspace_path.ok_or_else(|| {
            SerializableError::new("workspace_not_configured", "select a workspace first")
        })?;
        Ok(PathBuf::from(path))
    }
}
