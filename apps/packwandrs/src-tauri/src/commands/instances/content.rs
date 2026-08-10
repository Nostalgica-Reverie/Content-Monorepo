use packwand_orchestrator::{InstanceContent, PendingManualDownload, content, install};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use super::repository;
use crate::error::{CommandResult, SerializableError};

#[tauri::command]
pub async fn instances_content_list(
	id: String,
	app: AppHandle,
) -> CommandResult<Vec<InstanceContent>> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(content::list(&repo, &id)?)).await
}

#[tauri::command]
pub async fn instances_content_toggle(
	id: String,
	path: String,
	app: AppHandle,
) -> CommandResult<String> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(content::toggle(&repo, &id, &path)?)).await
}

#[tauri::command]
pub async fn instances_content_remove(
	id: String,
	path: String,
	app: AppHandle,
) -> CommandResult<()> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(content::remove(&repo, &id, &path)?)).await
}

/// Mods left over from the last install that CurseForge's API will not serve
/// (third-party distribution disabled by the author). The install itself
/// already succeeded; these still need a human to place by hand.
#[tauri::command]
pub async fn instances_manual_pending(
	id: String,
	app: AppHandle,
) -> CommandResult<Vec<PendingManualDownload>> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(install::manual_pending(&repo, &id)?)).await
}

/// Prism-style "I already downloaded it": opens a native file picker, and if
/// the user selects a file, verifies it against the pending mod's expected
/// hash and copies it into place. Returns `false` if the dialog was cancelled
/// rather than erroring.
#[tauri::command]
pub async fn instances_manual_provide(
	id: String,
	target: String,
	app: AppHandle,
) -> CommandResult<bool> {
	let repo = repository(&app)?;
	let Some(selected) = app.dialog().file().blocking_pick_file() else {
		return Ok(false);
	};
	let source = selected
		.into_path()
		.map_err(|error| SerializableError::new("invalid_path", error.to_string()))?;
	crate::commands::off_thread(move || {
		install::provide_manual(&repo, &id, &target, &source)?;
		Ok(true)
	})
	.await
}
