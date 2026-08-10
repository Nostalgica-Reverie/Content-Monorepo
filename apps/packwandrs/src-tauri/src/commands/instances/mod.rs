//! Tauri surface for persistent, user-owned Minecraft instances.
//!
//! The work lives in `packwand-orchestrator`; these modules resolve paths and
//! settings out of `AppState`, hand owned values to a blocking thread, and
//! translate the result. Anything with real logic in it belongs on the other
//! side of that seam, where it can be tested without a webview.

// Public because `tauri::generate_handler!` resolves each command's hidden
// companion macro alongside the function, which a re-export does not carry.
pub mod art;
pub mod content;
pub mod crud;
pub mod files;
pub mod io;
pub mod run;

use std::collections::HashMap;
use std::sync::Arc;

use packwand_instance::FsUserInstanceRepository;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

use crate::error::CommandResult;

/// The live phase of one instance, mirrored to the frontend over
/// `instance:status`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatusPayload {
	pub id: String,
	pub phase: String,
	pub message: Option<String>,
	pub job_id: Option<String>,
	pub exit_code: Option<i32>,
}

/// In-memory phase map, so a window opened mid-launch can catch up.
#[derive(Clone, Default)]
pub struct InstanceRegistry {
	entries: Arc<RwLock<HashMap<String, InstanceStatusPayload>>>,
}

impl InstanceRegistry {
	pub async fn set(&self, payload: InstanceStatusPayload) {
		self.entries
			.write()
			.await
			.insert(payload.id.clone(), payload);
	}

	pub async fn list(&self) -> Vec<InstanceStatusPayload> {
		self.entries.read().await.values().cloned().collect()
	}

	/// The job driving this instance, if one is still starting or running.
	pub async fn job_id_for(&self, id: &str) -> Option<String> {
		self.entries
			.read()
			.await
			.get(id)
			.filter(|entry| matches!(entry.phase.as_str(), "starting" | "running"))
			.and_then(|entry| entry.job_id.clone())
	}
}

/// The instance repository rooted at the app's data directory.
fn repository(app: &AppHandle) -> CommandResult<FsUserInstanceRepository> {
	let root = app
		.path()
		.app_data_dir()
		.map_err(|error| crate::error::SerializableError::new("path", error.to_string()))?;
	Ok(FsUserInstanceRepository::new(root))
}
