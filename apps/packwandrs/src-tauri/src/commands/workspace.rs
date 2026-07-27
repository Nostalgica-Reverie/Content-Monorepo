use std::path::PathBuf;

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::commands::jobs::JobRecord;
use crate::error::{CommandResult, SerializableError};
use crate::events::{emit_packs_changed, emit_settings_changed};
use crate::state::AppState;

fn set_workspace(path: PathBuf, app: &AppHandle, state: &AppState) -> CommandResult<String> {
    if !path.is_dir() {
        return Err(SerializableError::new(
            "invalid_workspace",
            format!("{} is not a directory", path.display()),
        ));
    }
    let canonical = path.canonicalize()?;
    let display = canonical.to_string_lossy().into_owned();
    let mut settings = state.settings()?;
    settings.workspace_path = Some(display.clone());
    let settings = state.update_settings(settings)?;
    state.restart_watch(app, &canonical)?;
    emit_settings_changed(app, settings)?;
    emit_packs_changed(app)?;
    Ok(display)
}

#[tauri::command]
pub fn workspace_get(state: State<'_, AppState>) -> CommandResult<Option<String>> {
    Ok(state.settings()?.workspace_path)
}

#[tauri::command]
pub fn workspace_set(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<String> {
    set_workspace(PathBuf::from(path), &app, &state)
}

#[tauri::command]
pub async fn workspace_select(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Option<String>> {
    let selected = app.dialog().file().blocking_pick_folder();
    selected
        .map(|path| {
            path.into_path()
                .map_err(|error| SerializableError::new("invalid_workspace", error.to_string()))
                .and_then(|path| set_workspace(path, &app, &state))
        })
        .transpose()
}

#[tauri::command]
pub async fn select_workspace(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Option<String>> {
    workspace_select(app, state).await
}

/// Compatibility alias for the removed loopback backend. It is always absent.
#[tauri::command]
pub const fn backend_url() -> Option<String> {
    None
}

#[tauri::command]
pub fn workspace_sync_preview(
    state: State<'_, AppState>,
) -> CommandResult<packwand_workspace::SyncReport> {
    Ok(packwand_workspace::sync_performance_bases(
        state.workspace()?,
        true,
    )?)
}

#[tauri::command]
pub async fn workspace_sync(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
    let root = state.workspace()?;
    let changed_app = app.clone();
    Ok(state
        .jobs
        .spawn(
            app,
            "workspace.sync",
            "Synchronize performance bases",
            move |context| async move {
                context
                    .log("Applying validated base-to-consumer mappings")
                    .await;
                let report = tokio::task::spawn_blocking(move || {
                    packwand_workspace::sync_performance_bases(root, false)
                })
                .await
                .map_err(|error| SerializableError::new("task", error.to_string()))??;
                context
                    .log(format!(
                        "copied {}, deleted {}, {} mapping(s)",
                        report.copied,
                        report.deleted,
                        report.jobs.len()
                    ))
                    .await;
                context
                    .progress(1.0, Some("Workspace synchronization complete".into()))
                    .await;
                emit_packs_changed(&changed_app)?;
                Ok(())
            },
        )
        .await)
}
