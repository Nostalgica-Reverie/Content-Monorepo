use packwand_orchestrator::{InstanceFileEntry, files};
use tauri::AppHandle;

use super::repository;
use crate::error::CommandResult;

#[tauri::command]
pub async fn instances_files_list(
	id: String,
	app: AppHandle,
) -> CommandResult<Vec<InstanceFileEntry>> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(files::list(&repo, &id)?)).await
}

#[tauri::command]
pub async fn instances_file_read(
	id: String,
	path: String,
	app: AppHandle,
) -> CommandResult<String> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(files::read(&repo, &id, &path)?)).await
}

#[tauri::command]
pub async fn instances_file_write(
	id: String,
	path: String,
	content: String,
	app: AppHandle,
) -> CommandResult<()> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(files::write(&repo, &id, &path, &content)?)).await
}
