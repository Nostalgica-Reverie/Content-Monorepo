use packwand_workspace::{Manifest, NewProject, Project};
use tauri::{AppHandle, State};

use crate::commands::off_thread;
use crate::error::CommandResult;
use crate::events::emit_packs_changed;
use crate::state::AppState;

#[tauri::command]
pub async fn projects_list(state: State<'_, AppState>) -> CommandResult<Vec<Project>> {
    // `discover` walks every content root and parses every manifest.
    let root = state.workspace()?;
    off_thread(move || Ok(packwand_workspace::discover(root)?)).await
}

#[tauri::command]
pub async fn projects_get(id: String, state: State<'_, AppState>) -> CommandResult<Project> {
    let root = state.workspace()?;
    off_thread(move || Ok(packwand_workspace::find(root, &id)?)).await
}

#[tauri::command]
pub async fn projects_create(
    request: NewProject,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Project> {
    let root = state.workspace()?;
    // Scaffolds a whole pack directory tree.
    let project =
        off_thread(move || Ok(packwand_workspace::create_project(root, &request)?)).await?;
    emit_packs_changed(&app)?;
    Ok(project)
}

#[tauri::command]
pub async fn projects_manifest_update(
    id: String,
    manifest: Manifest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Manifest> {
    let root = state.workspace()?;
    let manifest = off_thread(move || {
        let project = packwand_workspace::find(root, &id)?;
        packwand_workspace::write_manifest(project.root, &manifest)?;
        Ok(manifest)
    })
    .await?;
    emit_packs_changed(&app)?;
    Ok(manifest)
}

#[tauri::command]
pub async fn projects_bump(
    id: String,
    version: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<(String, String)> {
    let root = state.workspace()?;
    let outcome = off_thread(move || Ok(packwand_workspace::bump(root, &id, &version)?)).await?;
    emit_packs_changed(&app)?;
    Ok(outcome)
}

#[tauri::command]
pub async fn projects_freeze(
    id: String,
    subdir: String,
    slugs: Vec<String>,
    frozen: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Vec<String>> {
    let root = state.workspace()?;
    let changed = off_thread(move || {
        Ok(packwand_workspace::set_frozen(
            root, &id, &subdir, &slugs, frozen,
        )?)
    })
    .await?;
    if !changed.is_empty() {
        emit_packs_changed(&app)?;
    }
    Ok(changed)
}
