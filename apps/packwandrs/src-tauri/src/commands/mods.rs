use std::fs;

use packwand_ops::Workspace;
use packwand_pack::Mod;
use packwand_providers::ResolvedProject;
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::commands::jobs::JobRecord;
use crate::commands::off_thread;
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
pub async fn mods_list(id: String, state: State<'_, AppState>) -> CommandResult<Vec<ModSummary>> {
    let root = pack_root(&state.workspace()?, &id)?;
    tokio::task::spawn_blocking(move || list_mods(&root))
        .await
        .map_err(|error| SerializableError::new("task", error.to_string()))?
}

/// One `.pw.json` read and TOML parse per mod — a large pack has hundreds, so
/// the reads run concurrently. Results are sorted by name afterwards, so the
/// order the workers finish in never reaches the frontend.
fn list_mods(root: &std::path::Path) -> CommandResult<Vec<ModSummary>> {
    let workspace = Workspace::open(root)?;
    let entries: Vec<_> = workspace
        .index()
        .files
        .iter()
        .filter(|entry| entry.metafile && entry.alias.is_none())
        .collect();
    let parsed = packwand_parallel::try_map(
        &entries,
        packwand_parallel::configured(),
        |entry| -> CommandResult<ModSummary> {
            let path = safe_join(root, &entry.file)?;
            let metadata: Mod = serde_json::from_str(&fs::read_to_string(path)?)?;
            Ok(ModSummary {
                metadata_path: entry.file.clone(),
                name: metadata.name,
                filename: metadata.filename,
                side: metadata.side,
                pinned: metadata.pin,
                providers: metadata.update.keys().cloned().collect(),
            })
        },
    );
    let mut mods = parsed.into_iter().collect::<CommandResult<Vec<_>>>()?;
    mods.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(mods)
}

#[tauri::command]
pub async fn mods_add(
    id: String,
    metadata_path: String,
    metadata: Mod,
    replace: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    // `Workspace::open` parses the whole index and `add_metadata` rewrites it.
    off_thread(move || {
        Workspace::open(root)?.add_metadata(&metadata_path, metadata, replace)?;
        Ok(())
    })
    .await?;
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
                context.log("Scanning .pw.json metadata").await;
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
    let mut metadata: Mod = serde_json::from_str(&fs::read_to_string(&path)?)?;
    edit(&mut metadata);
    Workspace::open(root)?.add_metadata(metadata_path, metadata, true)?;
    Ok(())
}

#[tauri::command]
pub async fn mods_pin(
    id: String,
    metadata_path: String,
    pinned: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let root = pack_root(&state.workspace()?, &id)?;
    // `edit_metadata` opens the `Workspace`, which parses the pack's whole
    // index — tens of thousands of lines on a large pack.
    off_thread(move || edit_metadata(&root, &metadata_path, |metadata| metadata.pin = pinned))
        .await?;
    emit_packs_changed(&app)
}

#[tauri::command]
pub async fn mods_side_get(
    id: String,
    metadata_path: String,
    state: State<'_, AppState>,
) -> CommandResult<String> {
    let root = pack_root(&state.workspace()?, &id)?;
    off_thread(move || {
        let metadata: Mod =
            toml::from_str(&fs::read_to_string(safe_join(&root, &metadata_path)?)?)?;
        Ok(metadata.side)
    })
    .await
}

#[tauri::command]
pub async fn mods_side_set(
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
    off_thread(move || edit_metadata(&root, &metadata_path, |metadata| metadata.side = side))
        .await?;
    emit_packs_changed(&app)
}
