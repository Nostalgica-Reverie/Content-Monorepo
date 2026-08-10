use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use tauri::{AppHandle, State};

use crate::commands::jobs::JobRecord;
use crate::commands::off_thread;
use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::events::emit_packs_changed;
use crate::fsutil::safe_join;
use crate::state::AppState;

/// One discovered `packeater.json` marker, relative to the workspace.
#[derive(Debug, serde::Serialize)]
pub struct PackeaterMarker {
	pub path: String,
	pub directory: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackeaterPreview {
	pub path: String,
	pub directory: String,
	pub enabled: bool,
	pub output: String,
	pub file_count: usize,
	pub input_bytes: u64,
}

/// Lists the Packeater markers under a pack, so the UI can show what would be
/// compressed before anyone commits to a run.
#[tauri::command]
pub async fn packeater_markers(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<Vec<PackeaterMarker>> {
	let workspace = state.workspace()?;
	let root = pack_root(&workspace, &id)?;
	off_thread(move || markers_inner(&workspace, &root)).await
}

fn markers_inner(workspace: &Path, root: &Path) -> CommandResult<Vec<PackeaterMarker>> {
	let markers = packwand_build::discover_packeater_markers(root)
		.map_err(|error| SerializableError::new("packeater", error.to_string()))?;
	Ok(markers
		.into_iter()
		.map(|marker| {
			let directory = marker
				.parent()
				.map(|parent| display_relative(workspace, parent))
				.unwrap_or_default();
			PackeaterMarker {
				path: display_relative(workspace, &marker),
				directory,
			}
		})
		.collect())
}

/// Reads and validates marker JSON and reports the exact source footprint that
/// a Packeater run will consume. This is intentionally read-only.
#[tauri::command]
pub async fn packeater_preview(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<Vec<PackeaterPreview>> {
	let workspace = state.workspace()?;
	let root = pack_root(&workspace, &id)?;
	// `source_footprint` walks and stats every file under each marker.
	off_thread(move || preview_inner(&workspace, &root, &id)).await
}

fn preview_inner(workspace: &Path, root: &Path, id: &str) -> CommandResult<Vec<PackeaterPreview>> {
	let destination = workspace.join("build/packeater").join(id);
	let markers = packwand_build::discover_packeater_markers(root)
		.map_err(|error| SerializableError::new("packeater", error.to_string()))?;
	markers
		.into_iter()
		.map(|marker| {
			let source = fs::read_to_string(&marker)
				.map_err(|error| SerializableError::new("packeater_config", error.to_string()))?;
			let config: serde_json::Value = serde_json::from_str(&source).map_err(|error| {
				SerializableError::new("packeater_config", format!("{}: {error}", marker.display()))
			})?;
			let object = config.as_object().ok_or_else(|| {
				SerializableError::new(
					"packeater_config",
					format!("{} must contain a JSON object", marker.display()),
				)
			})?;
			if object
				.get("version")
				.and_then(serde_json::Value::as_u64)
				.is_some_and(|version| version != 1)
			{
				return Err(SerializableError::new(
					"packeater_config",
					format!("{} uses an unsupported version", marker.display()),
				));
			}
			let directory = marker.parent().unwrap_or(root);
			let (file_count, input_bytes) = source_footprint(directory)?;
			let name = directory
				.file_name()
				.and_then(|value| value.to_str())
				.unwrap_or("pack");
			Ok(PackeaterPreview {
				path: display_relative(workspace, &marker),
				directory: display_relative(workspace, directory),
				enabled: object
					.get("enabled")
					.and_then(serde_json::Value::as_bool)
					.unwrap_or(true),
				output: display_relative(workspace, &destination.join(format!("{name}.zip"))),
				file_count,
				input_bytes,
			})
		})
		.collect()
}

/// Adds a conservative default marker to the selected pack. `create_new`
/// guarantees an existing, hand-authored optimizer configuration is never
/// overwritten.
#[tauri::command]
pub async fn packeater_initialize(
	id: String,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<PackeaterMarker> {
	let workspace = state.workspace()?;
	let root = pack_root(&workspace, &id)?;
	let created = off_thread(move || initialize_inner(&workspace, &root)).await?;
	emit_packs_changed(&app)?;
	Ok(created)
}

fn initialize_inner(workspace: &Path, root: &Path) -> CommandResult<PackeaterMarker> {
	let marker = root.join(packwand_build::PACKEATER_MARKER);
	let mut file = OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(&marker)
		.map_err(|error| {
			SerializableError::new(
				"packeater_config",
				if error.kind() == std::io::ErrorKind::AlreadyExists {
					"packeater.json already exists".into()
				} else {
					error.to_string()
				},
			)
		})?;
	file.write_all(br#"{
  "$schema": "https://raw.githubusercontent.com/Lasting-Legacy/Lasting-Legacy-Monorepo/main/apps/packwandrs/docs/packeater/packeater.schema.json",
  "version": 1,
  "enabled": true
}
"#)
        .map_err(|error| SerializableError::new("packeater_config", error.to_string()))?;
	Ok(PackeaterMarker {
		path: display_relative(workspace, &marker),
		directory: display_relative(workspace, root),
	})
}

fn source_footprint(root: &Path) -> CommandResult<(usize, u64)> {
	let mut files = 0usize;
	let mut bytes = 0u64;
	for entry in walkdir::WalkDir::new(root).follow_links(false) {
		let entry = entry
			.map_err(|error| SerializableError::new("packeater_preview", error.to_string()))?;
		if entry.file_type().is_file()
			&& entry.file_name().to_str() != Some(packwand_build::PACKEATER_MARKER)
		{
			let metadata = entry
				.metadata()
				.map_err(|error| SerializableError::new("packeater_preview", error.to_string()))?;
			files += 1;
			bytes = bytes.saturating_add(metadata.len());
		}
	}
	Ok((files, bytes))
}

/// Compresses every Packeater marker under a pack. Runs as a job because a large
/// resource pack takes long enough that a blocking command would freeze the UI.
#[tauri::command]
// `output` is deliberately one word: every other command in this crate takes
// single-word parameters, so there is no in-repo precedent for how a multi-word
// name is cased across the IPC boundary.
pub async fn packeater_run(
	id: String,
	output: Option<String>,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let workspace = state.workspace()?;
	let root = pack_root(&workspace, &id)?;
	let destination = output
		.filter(|value| !value.trim().is_empty())
		.map(|value| safe_join(&workspace, &value))
		.transpose()?
		.unwrap_or_else(|| workspace.join("build/packeater").join(&id));
	if destination.starts_with(&root) {
		return Err(SerializableError::new(
			"packeater_output",
			"Packeater output must be outside the source pack",
		));
	}
	let label = format!("Packeater {id}");
	Ok(state
		.jobs
		.spawn(app, "packeater.run", label, move |context| async move {
			let markers = tokio::task::spawn_blocking({
				let root = root.clone();
				move || packwand_build::discover_packeater_markers(&root)
			})
			.await
			.map_err(|error| SerializableError::new("task", error.to_string()))?
			.map_err(|error| SerializableError::new("packeater", error.to_string()))?;

			if markers.is_empty() {
				context
					.log("No packeater.json markers found; nothing to compress")
					.await;
				context.progress(1.0, Some("Nothing to do".into())).await;
				return Ok(());
			}

			let total = markers.len();
			context.log(format!("Compressing {total} marker(s)")).await;
			for (index, marker) in markers.into_iter().enumerate() {
				let enabled = fs::read_to_string(&marker)
					.ok()
					.and_then(|source| serde_json::from_str::<serde_json::Value>(&source).ok())
					.and_then(|config| config.get("enabled").and_then(serde_json::Value::as_bool))
					.unwrap_or(true);
				if !enabled {
					context
						.log(format!("Skipping disabled marker {}", marker.display()))
						.await;
					context
						.progress(
							(index + 1) as f64 / total as f64,
							Some(format!("Skipped disabled {}/{total}", index + 1)),
						)
						.await;
					continue;
				}
				let name = marker
					.parent()
					.and_then(|parent| parent.file_name())
					.map(|name| name.to_string_lossy().into_owned())
					.unwrap_or_else(|| "pack".into());
				let output = destination.join(format!("{name}.zip"));
				context.log(format!("packeater {name}")).await;
				let bytes = tokio::task::spawn_blocking({
					let marker = marker.clone();
					let output = output.clone();
					move || packwand_build::run_packeater(marker, output)
				})
				.await
				.map_err(|error| SerializableError::new("task", error.to_string()))?
				.map_err(|error| SerializableError::new("packeater", error.to_string()))?;
				context
					.log(format!("{name}: {bytes} byte(s) -> {}", output.display()))
					.await;
				context
					.progress(
						(index + 1) as f64 / total as f64,
						Some(format!("Compressed {}/{total}", index + 1)),
					)
					.await;
			}
			Ok(())
		})
		.await)
}

/// Workspace-relative, forward-slashed path for display.
fn display_relative(workspace: &std::path::Path, path: &std::path::Path) -> String {
	path.strip_prefix(workspace)
		.unwrap_or(path)
		.to_string_lossy()
		.replace('\\', "/")
}

#[cfg(test)]
mod tests {
	use super::source_footprint;

	#[test]
	fn preview_footprint_excludes_the_marker() {
		let root = tempfile::tempdir().unwrap();
		std::fs::write(root.path().join("packeater.json"), "{}").unwrap();
		std::fs::write(root.path().join("pack.mcmeta"), "1234").unwrap();
		std::fs::create_dir(root.path().join("assets")).unwrap();
		std::fs::write(root.path().join("assets/texture.png"), "123456").unwrap();
		assert_eq!(source_footprint(root.path()).unwrap(), (2, 10));
	}
}
