use std::fs;
use std::time::UNIX_EPOCH;

use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::error::{CommandResult, SerializableError};
use crate::fsutil::atomic_write;
use crate::state::AppState;

const SUFFIX: &str = ".packwand-theme.json";
const MAX_THEME_BYTES: u64 = 256 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredTheme {
	pub file_name: String,
	pub modified_ms: u64,
	pub theme: Option<Value>,
	pub error: Option<String>,
}

fn valid_user_id(id: &str) -> bool {
	let Some(rest) = id.strip_prefix("user.") else {
		return false;
	};
	!rest.is_empty()
		&& rest.bytes().all(|byte| {
			byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'-'
		}) && !rest.starts_with(['.', '-'])
		&& !rest.ends_with(['.', '-'])
		&& !rest.contains("..")
}

fn validate_envelope(theme: &Value) -> CommandResult<&str> {
	let object = theme
		.as_object()
		.ok_or_else(|| SerializableError::new("invalid_theme", "theme must be a JSON object"))?;
	if object.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
		return Err(SerializableError::new(
			"invalid_theme",
			"theme schemaVersion must be 1",
		));
	}
	let id = object
		.get("id")
		.and_then(Value::as_str)
		.filter(|id| valid_user_id(id))
		.ok_or_else(|| {
			SerializableError::new("invalid_theme", "custom theme id must be a user.* slug")
		})?;
	if object
		.get("name")
		.and_then(Value::as_str)
		.is_none_or(|name| name.trim().is_empty())
	{
		return Err(SerializableError::new(
			"invalid_theme",
			"theme name is required",
		));
	}
	if !matches!(
		object.get("appearance").and_then(Value::as_str),
		Some("light" | "dark" | "high-contrast")
	) {
		return Err(SerializableError::new(
			"invalid_theme",
			"theme appearance is invalid",
		));
	}
	Ok(id)
}

fn themes_dir(state: &AppState) -> std::path::PathBuf {
	state.config_dir().join("themes")
}

#[tauri::command]
pub fn themes_list(state: State<'_, AppState>) -> CommandResult<Vec<StoredTheme>> {
	let directory = themes_dir(&state);
	fs::create_dir_all(&directory)?;
	let mut records = Vec::new();
	for entry in fs::read_dir(directory)? {
		let entry = entry?;
		let path = entry.path();
		let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
			continue;
		};
		if !entry.file_type()?.is_file() || !file_name.ends_with(SUFFIX) {
			continue;
		}
		let metadata = entry.metadata()?;
		let modified_ms = metadata
			.modified()
			.ok()
			.and_then(|value| value.duration_since(UNIX_EPOCH).ok())
			.map(|value| value.as_millis().try_into().unwrap_or(u64::MAX))
			.unwrap_or(0);
		let loaded = if metadata.len() > MAX_THEME_BYTES {
			Err("theme exceeds the 256 KiB limit".to_owned())
		} else {
			fs::read(&path)
				.map_err(|error| error.to_string())
				.and_then(|bytes| {
					serde_json::from_slice::<Value>(&bytes).map_err(|error| error.to_string())
				})
				.and_then(|theme| match validate_envelope(&theme) {
					Ok(_) => Ok(theme),
					Err(error) => Err(error.message),
				})
		};
		let (theme, error) = match loaded {
			Ok(theme) => (Some(theme), None),
			Err(error) => (None, Some(error)),
		};
		records.push(StoredTheme {
			file_name: file_name.to_owned(),
			modified_ms,
			theme,
			error,
		});
	}
	records.sort_by(|left, right| left.file_name.cmp(&right.file_name));
	Ok(records)
}

#[tauri::command]
pub fn themes_save(theme: Value, state: State<'_, AppState>) -> CommandResult<Value> {
	let id = validate_envelope(&theme)?.to_owned();
	let bytes = serde_json::to_vec_pretty(&theme)?;
	if bytes.len() as u64 > MAX_THEME_BYTES {
		return Err(SerializableError::new(
			"invalid_theme",
			"theme exceeds the 256 KiB limit",
		));
	}
	atomic_write(&themes_dir(&state).join(format!("{id}{SUFFIX}")), &bytes)?;
	Ok(theme)
}

#[tauri::command]
pub fn themes_delete(id: String, state: State<'_, AppState>) -> CommandResult<()> {
	if !valid_user_id(&id) {
		return Err(SerializableError::new(
			"invalid_theme",
			"only custom user.* themes can be deleted",
		));
	}
	match fs::remove_file(themes_dir(&state).join(format!("{id}{SUFFIX}"))) {
		Ok(()) => Ok(()),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
		Err(error) => Err(error.into()),
	}
}

#[cfg(test)]
mod tests {
	use super::valid_user_id;

	#[test]
	fn custom_theme_ids_are_path_safe() {
		assert!(valid_user_id("user.my-theme"));
		assert!(valid_user_id("user.group.theme-2"));
		assert!(!valid_user_id("builtin.packwand-dark"));
		assert!(!valid_user_id("user../escape"));
		assert!(!valid_user_id("user.UPPER"));
	}
}
