use packwand_workspace::{Manifest, NewProject, Project};
use tauri::{AppHandle, State};

use crate::error::CommandResult;
use crate::events::emit_packs_changed;
use crate::state::AppState;

#[tauri::command]
pub fn projects_list(state: State<'_, AppState>) -> CommandResult<Vec<Project>> {
    Ok(packwand_workspace::discover(state.workspace()?)?)
}

#[tauri::command]
pub fn projects_get(id: String, state: State<'_, AppState>) -> CommandResult<Project> {
    Ok(packwand_workspace::find(state.workspace()?, &id)?)
}

#[tauri::command]
pub fn projects_create(
    request: NewProject,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Project> {
    let project = packwand_workspace::create_project(state.workspace()?, &request)?;
    emit_packs_changed(&app)?;
    Ok(project)
}

#[tauri::command]
pub fn projects_manifest_update(
    id: String,
    manifest: Manifest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Manifest> {
    let project = packwand_workspace::find(state.workspace()?, &id)?;
    packwand_workspace::write_manifest(project.root, &manifest)?;
    emit_packs_changed(&app)?;
    Ok(manifest)
}

#[tauri::command]
pub fn projects_bump(
    id: String,
    version: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<(String, String)> {
    let outcome = packwand_workspace::bump(state.workspace()?, &id, &version)?;
    emit_packs_changed(&app)?;
    Ok(outcome)
}

#[tauri::command]
pub fn projects_freeze(
    id: String,
    subdir: String,
    slugs: Vec<String>,
    frozen: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Vec<String>> {
    let changed = packwand_workspace::set_frozen(state.workspace()?, &id, &subdir, &slugs, frozen)?;
    if !changed.is_empty() {
        emit_packs_changed(&app)?;
    }
    Ok(changed)
}
