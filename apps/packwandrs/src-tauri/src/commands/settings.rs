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
	let previous = state.settings()?.raw_input_enabled;
	crate::raw_input::set_enabled(&app, settings.raw_input_enabled)?;
	let settings = match state.update_settings(settings) {
		Ok(settings) => settings,
		Err(error) => {
			let _ = crate::raw_input::set_enabled(&app, previous);
			return Err(error);
		}
	};
	emit_settings_changed(&app, settings.clone())?;
	Ok(settings)
}
