use packwand_instance::Instance;
use packwand_orchestrator::{CreateSpec, InstancePatch, lifecycle};
use tauri::{AppHandle, State};

use super::repository;
use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

#[tauri::command]
pub async fn instances_list(app: AppHandle) -> CommandResult<Vec<Instance>> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(repo.list()?)).await
}

#[tauri::command]
pub async fn instances_get(id: String, app: AppHandle) -> CommandResult<Instance> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(repo.get(&id)?)).await
}

#[tauri::command]
pub async fn instances_create(
	spec: CreateSpec,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<Instance> {
	let repo = repository(&app)?;
	// `State` is not `'static`, so the workspace is resolved before the move.
	let workspace = state.workspace()?;
	crate::commands::off_thread(move || {
		Ok(lifecycle::create(&repo, spec, |pack_id| {
			pack_root(&workspace, pack_id).map_err(|error| {
				packwand_orchestrator::OrchestratorError::new(error.kind, error.message)
			})
		})?)
	})
	.await
}

#[tauri::command]
pub async fn instances_edit(
	id: String,
	patch: InstancePatch,
	app: AppHandle,
) -> CommandResult<Instance> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(lifecycle::edit(&repo, &id, patch)?)).await
}

#[tauri::command]
pub async fn instances_delete(
	id: String,
	delete_files: bool,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	if state.instances.job_id_for(&id).await.is_some() {
		return Err(SerializableError::new(
			"instance_running",
			"stop the instance before deleting it",
		));
	}
	let repo = repository(&app)?;
	crate::commands::off_thread(move || Ok(repo.delete(&id, delete_files)?)).await
}
