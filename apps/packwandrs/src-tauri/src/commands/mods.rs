use std::fs;

use packwand_ops::Workspace;
use packwand_pack::Mod;
use packwand_providers::ResolvedProject;
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::commands::jobs::JobRecord;
use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::events::emit_packs_changed;
use crate::fsutil::safe_join;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModSummary {
    pub metadata_path: String,
    pub name: String,
    pub filename: String,
    pub side: String,
    pub pinned: bool,
    pub providers: Vec<String>,
}

#[tauri::command]
pub fn mods_list(id: String, state: State<'_, AppState>) -> CommandResult<Vec<ModSummary>> {
    let root = pack_root(&state.workspace()?, &id)?;
    let workspace = Workspace::open(&root)?;
    let mut mods = Vec::new();
    for entry in workspace
        .index()
        .files
        .iter()
        .filter(|entry| entry.metafile && entry.alias.is_none())
    {
        let path = safe_join(&root, &entry.file)?;
        let metadata: Mod = toml::from_str(&fs::read_to_string(path)?)?;
        mods.push(ModSummary {
            metadata_path: entry.file.clone(),
            name: metadata.name,
            filename: metadata.filename,
            side: metadata.side,
            pinned: metadata.pin,
            providers: metadata.update.keys().cloned().collect(),
        });
    }
    mods.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(mods)
}

#[tauri::command]
pub fn mods_add(
    id: String,
    metadata_path: String,
    metadata: Mod,
    replace: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    Workspace::open(root)?.add_metadata(&metadata_path, metadata, replace)?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub async fn mods_remove(
    id: String,
    metadata_path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
    let root = pack_root(&state.workspace()?, &id)?;
    let changed_app = app.clone();
    Ok(state
        .jobs
        .spawn(
            app,
            "mods.remove",
            format!("Remove {metadata_path}"),
            move |context| async move {
                context.log(format!("Removing {metadata_path}")).await;
                let result = tokio::task::spawn_blocking(move || {
                    Workspace::open(root)?.remove_metadata(&metadata_path)
                })
                .await
                .map_err(|error| SerializableError::new("task", error.to_string()))?;
                result?;
                context.progress(1.0, Some("Metadata removed".into())).await;
                emit_packs_changed(&changed_app)?;
                Ok(())
            },
        )
        .await)
}

#[tauri::command]
pub async fn mods_update(
    id: String,
    metadata_path: String,
    resolved: ResolvedProject,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
    let root = pack_root(&state.workspace()?, &id)?;
    let changed_app = app.clone();
    Ok(state
        .jobs
        .spawn(
            app,
            "mods.update",
            format!("Update {metadata_path}"),
            move |context| async move {
                context.log(format!("Updating {metadata_path}")).await;
                let outcome = tokio::task::spawn_blocking(move || {
                    Workspace::open(root)?.update_resolved(&metadata_path, resolved)
                })
                .await
                .map_err(|error| SerializableError::new("task", error.to_string()))??;
                context
                    .log(if outcome.changed {
                        format!("{} -> {}", outcome.old_filename, outcome.new_filename)
                    } else {
                        "Already on the selected release".into()
                    })
                    .await;
                context.progress(1.0, Some("Update complete".into())).await;
                emit_packs_changed(&changed_app)?;
                Ok(())
            },
        )
        .await)
}

#[tauri::command]
pub async fn mods_refresh(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
    let root = pack_root(&state.workspace()?, &id)?;
    let changed_app = app.clone();
    Ok(state
        .jobs
        .spawn(
            app,
            "mods.refresh",
            "Refresh metadata index",
            move |context| async move {
                context.log("Scanning .pw.toml metadata").await;
                let report = tokio::task::spawn_blocking(move || {
                    Workspace::open(root)?.refresh_metadata_index()
                })
                .await
                .map_err(|error| SerializableError::new("task", error.to_string()))??;
                context
                    .log(format!(
                        "Added {}, updated {}, removed {} metadata entries",
                        report.added, report.updated, report.removed
                    ))
                    .await;
                context.progress(1.0, Some("Index refreshed".into())).await;
                emit_packs_changed(&changed_app)?;
                Ok(())
            },
        )
        .await)
}

fn edit_metadata(
    root: &std::path::Path,
    metadata_path: &str,
    edit: impl FnOnce(&mut Mod),
) -> CommandResult<()> {
    let path = safe_join(root, metadata_path)?;
    let mut metadata: Mod = toml::from_str(&fs::read_to_string(&path)?)?;
    edit(&mut metadata);
    Workspace::open(root)?.add_metadata(metadata_path, metadata, true)?;
    Ok(())
}

#[tauri::command]
pub fn mods_pin(
    id: String,
    metadata_path: String,
    pinned: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    edit_metadata(&root, &metadata_path, |metadata| metadata.pin = pinned)?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub fn mods_side_get(
    id: String,
    metadata_path: String,
    state: State<'_, AppState>,
) -> CommandResult<String> {
    let root = pack_root(&state.workspace()?, &id)?;
    let metadata: Mod = toml::from_str(&fs::read_to_string(safe_join(&root, &metadata_path)?)?)?;
    Ok(metadata.side)
}

#[tauri::command]
pub fn mods_side_set(
    id: String,
    metadata_path: String,
    side: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    if !matches!(side.as_str(), "both" | "client" | "server") {
        return Err(SerializableError::new(
            "invalid_side",
            "side must be both, client, or server",
        ));
    }
    let root = pack_root(&state.workspace()?, &id)?;
    edit_metadata(&root, &metadata_path, |metadata| metadata.side = side)?;
    emit_packs_changed(&app)
}
