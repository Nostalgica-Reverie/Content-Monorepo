use packwand_orchestrator::{ImageKind, art};
use tauri::AppHandle;

use super::repository;
use crate::error::CommandResult;

async fn read(id: String, kind: ImageKind, app: AppHandle) -> CommandResult<Option<Vec<u8>>> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(art::read(&repo, &id, kind)?)).await
}

#[tauri::command]
pub async fn instances_icon(id: String, app: AppHandle) -> CommandResult<Option<Vec<u8>>> {
	read(id, ImageKind::Icon, app).await
}

#[tauri::command]
pub async fn instances_image(
	id: String,
	kind: ImageKind,
	app: AppHandle,
) -> CommandResult<Option<Vec<u8>>> {
	read(id, kind, app).await
}
