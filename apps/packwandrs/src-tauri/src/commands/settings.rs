use tauri::{AppHandle, State};

use crate::error::CommandResult;
use crate::events::emit_settings_changed;
use crate::state::{AppSettings, AppState};

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> CommandResult<AppSettings> {
    state.settings()
}

#[tauri::command]
pub fn settings_update(
    settings: AppSettings,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<AppSettings> {
    let settings = state.update_settings(settings)?;
    emit_settings_changed(&app, settings.clone())?;
    Ok(settings)
}
