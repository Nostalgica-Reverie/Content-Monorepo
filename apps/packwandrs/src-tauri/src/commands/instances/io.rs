use std::path::PathBuf;

use packwand_instance::Instance;
use packwand_orchestrator::archive as archives;
use packwand_orchestrator::{ArchiveFormat, ExportResult};
use tauri::{AppHandle, State};

use super::repository;
use crate::error::CommandResult;
use crate::state::AppState;

#[tauri::command]
pub async fn instances_export(
	id: String,
	format: ArchiveFormat,
	output: Option<PathBuf>,
	app: AppHandle,
) -> CommandResult<ExportResult> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(archives::export(&repo, &id, format, output)?)).await
}

#[tauri::command]
pub async fn instances_import(
	archive: PathBuf,
	format: ArchiveFormat,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<Instance> {
	let repo = repository(&app)?;
	let default_jobs = state.settings()?.download_jobs;
	crate::commands::off_thread(move || {
		Ok(archives::import(&repo, &archive, format, default_jobs)?)
	})
	.await
}
