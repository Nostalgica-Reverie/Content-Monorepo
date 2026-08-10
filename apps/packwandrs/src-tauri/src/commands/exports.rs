use packwand_build::{
	ExportFormat, ExportOptions, ExportPlan, PublishMatrixEntry, PublishTarget, export_pack,
	list_publish_targets, plan_export,
};
use tauri::{AppHandle, State};

use crate::commands::jobs::JobRecord;
use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

#[tauri::command]
pub async fn exports_publish_plan(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<ExportPlan> {
	let root = pack_root(&state.workspace()?, &id)?;
	// `plan_export` reads the index and every metadata file it references.
	crate::commands::off_thread(move || Ok(plan_export(root)?)).await
}

#[tauri::command]
pub fn exports_publish_targets(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<Vec<PublishMatrixEntry>> {
	let project = packwand_workspace::find(state.workspace()?, &id)?;
	list_publish_targets([project.root.join("manifest.json")])
		.map_err(|error| SerializableError::new("publish", error.to_string()))
}

#[tauri::command]
pub fn exports_publish_inspect(
	id: String,
	variant: Option<String>,
	state: State<'_, AppState>,
) -> CommandResult<PublishTarget> {
	let project = packwand_workspace::find(state.workspace()?, &id)?;
	packwand_build::resolve_publish_target(project.root.join("manifest.json"), variant.as_deref())
		.map_err(|error| SerializableError::new("publish", error.to_string()))
}

#[tauri::command]
pub async fn exports_publish_build(
	id: String,
	variant: Option<String>,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let project = packwand_workspace::find(state.workspace()?, &id)?;
	let manifest = project.root.join("manifest.json");
	Ok(state
		.jobs
		.spawn(
			app,
			"publish.build",
			format!("Build publish target {id}"),
			move |context| async move {
				context
					.log("Building the selected publish matrix entry")
					.await;
				let target = tokio::task::spawn_blocking(move || {
					packwand_build::build_publish_target(manifest, variant.as_deref())
				})
				.await
				.map_err(|error| SerializableError::new("task", error.to_string()))?
				.map_err(|error| SerializableError::new("publish", error.to_string()))?;
				for artifact in target.artifacts.iter().filter(|artifact| artifact.exists) {
					context
						.log(format!(
							"{}: {} ({} bytes)",
							artifact.platform,
							artifact.path.display(),
							artifact.bytes
						))
						.await;
				}
				context
					.progress(1.0, Some("Publish artifacts built".into()))
					.await;
				Ok(())
			},
		)
		.await)
}

#[tauri::command]
pub async fn exports_publish_upload(
	id: String,
	variant: Option<String>,
	live: bool,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let project = packwand_workspace::find(state.workspace()?, &id)?;
	let manifest = project.root.join("manifest.json");
	Ok(state
		.jobs
		.spawn(
			app,
			"publish.upload",
			format!(
				"{} publish target {id}",
				if live { "Upload" } else { "Dry-run" }
			),
			move |context| async move {
				context
					.log(if live {
						"Uploading release artifacts"
					} else {
						"Validating upload without contacting release APIs"
					})
					.await;
				// Credentials come from the OS keychain the user connected in
				// Settings, falling back to the environment. Resolved here
				// rather than inside the library so the CLI keeps its own,
				// environment-only path.
				let credentials = crate::commands::accounts::publish_credentials();
				let report = tokio::task::spawn_blocking(move || {
					packwand_build::upload_publish_target_with(
						manifest,
						variant.as_deref(),
						live,
						None,
						&credentials,
					)
				})
				.await
				.map_err(|error| SerializableError::new("task", error.to_string()))?
				.map_err(|error| SerializableError::new("publish", error.to_string()))?;
				context
					.log(format!(
						"attempted: {}; uploaded/would upload: {}; skipped: {}",
						report.attempted.join(", "),
						report.uploaded.join(", "),
						report.skipped.join(", ")
					))
					.await;
				context
					.progress(
						1.0,
						Some(
							if live {
								"Upload complete"
							} else {
								"Dry-run complete"
							}
							.into(),
						),
					)
					.await;
				Ok(())
			},
		)
		.await)
}

#[tauri::command]
pub async fn exports_publish_verify(
	id: String,
	variant: Option<String>,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let project = packwand_workspace::find(state.workspace()?, &id)?;
	let manifest = project.root.join("manifest.json");
	Ok(state
		.jobs
		.spawn(
			app,
			"publish.verify",
			format!("Verify published target {id}"),
			move |context| async move {
				context
					.log("Checking the public Modrinth version API")
					.await;
				let found = tokio::task::spawn_blocking(move || {
					packwand_build::verify_publish_target(
						manifest,
						variant.as_deref(),
						8,
						std::time::Duration::from_secs(15),
					)
				})
				.await
				.map_err(|error| SerializableError::new("task", error.to_string()))?
				.map_err(|error| SerializableError::new("publish", error.to_string()))?;
				if !found {
					return Err(SerializableError::new(
						"publish_verify",
						"release did not become visible before verification timed out",
					));
				}
				context.log("Release is visible").await;
				context
					.progress(1.0, Some("Verification complete".into()))
					.await;
				Ok(())
			},
		)
		.await)
}

#[tauri::command]
pub async fn exports_build(
	id: String,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let root = pack_root(&state.workspace()?, &id)?;
	Ok(state
		.jobs
		.spawn(
			app,
			"exports.build",
			"Build pack export",
			move |context| async move {
				context
					.log("Reading pack metadata and building archive")
					.await;
				let format = if root
					.file_name()
					.and_then(|name| name.to_str())
					.is_some_and(|name| name.ends_with("-cf"))
				{
					ExportFormat::CurseForge
				} else {
					ExportFormat::Modrinth
				};
				let artifact = tokio::task::spawn_blocking(move || {
					export_pack(
						&root,
						format,
						None::<&std::path::Path>,
						ExportOptions::default(),
					)
				})
				.await
				.map_err(|error| SerializableError::new("task", error.to_string()))??;
				context
					.log(format!(
						"Wrote {} files to {} ({} bytes)",
						artifact.files,
						artifact.path.display(),
						artifact.bytes
					))
					.await;
				context.progress(1.0, Some("Export complete".into())).await;
				Ok(())
			},
		)
		.await)
}
