use std::path::PathBuf;

use packwand_instance::InstanceSettings;
use packwand_orchestrator::{LaunchRequest, LaunchSignal, install, launch};
use tauri::{AppHandle, State};

use super::{InstanceStatusPayload, repository};
use crate::commands::jobs::JobRecord;
use crate::error::{CommandResult, SerializableError};
use crate::events::emit_instance_status;
use crate::state::AppState;

/// Records a phase in the registry and mirrors it to the frontend.
async fn announce(
	app: &AppHandle,
	registry: &super::InstanceRegistry,
	payload: InstanceStatusPayload,
) {
	registry.set(payload.clone()).await;
	let _ = emit_instance_status(app, payload);
}

#[tauri::command]
pub async fn instances_status_list(
	state: State<'_, AppState>,
) -> CommandResult<Vec<InstanceStatusPayload>> {
	Ok(state.instances.list().await)
}

#[tauri::command]
pub async fn instances_stop(id: String, state: State<'_, AppState>) -> CommandResult<bool> {
	match state.instances.job_id_for(&id).await {
		Some(job) => Ok(state.jobs.cancel(&job).await),
		None => Ok(false),
	}
}

#[tauri::command]
pub async fn instances_install(
	id: String,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let repo = repository(&app)?;
	repo.get(&id)?;
	let default_jobs = state.settings()?.download_jobs;
	let registry = state.instances.clone();
	let install_app = app.clone();
	let install_id = id.clone();
	let job = state
		.jobs
		.spawn(
			app,
			"instance.install",
			format!("Install {id}"),
			move |context| async move {
				announce(
					&install_app,
					&registry,
					InstanceStatusPayload {
						id: install_id.clone(),
						phase: "starting".to_owned(),
						message: Some("Installing pack content".to_owned()),
						job_id: Some(context.id().to_owned()),
						exit_code: None,
					},
				)
				.await;
				context
					.progress(0.05, Some("Installing pack content".to_owned()))
					.await;

				let result =
					tokio::task::spawn_blocking(move || install::install(&repo, &id, default_jobs))
						.await
						.map_err(|error| SerializableError::new("task", error.to_string()))?;
				if let Err(error) = result {
					let error = SerializableError::from(error);
					announce(
						&install_app,
						&registry,
						InstanceStatusPayload {
							id: install_id,
							phase: "error".to_owned(),
							message: Some(error.message.clone()),
							job_id: Some(context.id().to_owned()),
							exit_code: None,
						},
					)
					.await;
					return Err(error);
				}
				context
					.progress(1.0, Some("Instance ready".to_owned()))
					.await;
				announce(
					&install_app,
					&registry,
					InstanceStatusPayload {
						id: install_id,
						phase: "stopped".to_owned(),
						message: Some("Ready".to_owned()),
						job_id: Some(context.id().to_owned()),
						exit_code: None,
					},
				)
				.await;
				Ok(())
			},
		)
		.await;
	Ok(job)
}

#[tauri::command]
pub async fn instances_launch(
	id: String,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let repo = repository(&app)?;
	let instance = repo.get(&id)?;
	let app_settings = state.settings()?;
	// App defaults are the inherited layer; anything the instance sets wins.
	let inherited = InstanceSettings {
		java_path: app_settings
			.java_defaults
			.get(&instance.game_version)
			.map(PathBuf::from),
		memory_max_mb: Some(app_settings.memory_mb),
		..Default::default()
	};
	let settings = instance.settings.merged(&inherited);
	let managed_root = launch::managed_root(repo.root());
	let default_jobs = app_settings.download_jobs;
	let msa_client_id = app_settings.msa_client_id.clone();
	let registry = state.instances.clone();
	let launch_app = app.clone();
	let launch_id = id.clone();

	let job = state
		.jobs
		.spawn(
			app,
			"instance.launch",
			format!("Launch {id}"),
			move |context| async move {
				let job_id = context.id().to_owned();
				// The supervisor is blocking and the job context is async, so
				// signals cross on a channel rather than by calling into the
				// runtime from a blocking thread.
				let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
				let cancel = context.clone();
				let blocking = tokio::task::spawn_blocking(move || {
					launch::run(
						&repo,
						&id,
						&LaunchRequest {
							managed_root: &managed_root,
							default_jobs,
							settings: &settings,
							msa_client_id: msa_client_id.clone(),
						},
						&|| cancel.is_cancelled(),
						&|signal| {
							let _ = tx.send(signal);
						},
					)
				});

				while let Some(signal) = rx.recv().await {
					match signal {
						LaunchSignal::Log(line) => context.log(line).await,
						LaunchSignal::Progress(value, message) => {
							context.progress(value, message).await;
						}
						LaunchSignal::Status(phase, message, exit_code) => {
							announce(
								&launch_app,
								&registry,
								InstanceStatusPayload {
									id: launch_id.clone(),
									phase: phase.to_owned(),
									message,
									job_id: Some(job_id.clone()),
									exit_code,
								},
							)
							.await;
						}
					}
				}

				let outcome = blocking
					.await
					.map_err(|error| SerializableError::new("task", error.to_string()))?;
				match outcome {
					Ok(()) => Ok(()),
					Err(error) => {
						// A cancelled launch is a stop, not a failure — the
						// user asked for it.
						let phase = if error.kind == "cancelled" {
							"stopped"
						} else {
							"error"
						};
						let error = SerializableError::from(error);
						announce(
							&launch_app,
							&registry,
							InstanceStatusPayload {
								id: launch_id,
								phase: phase.to_owned(),
								message: Some(error.message.clone()),
								job_id: Some(job_id),
								exit_code: None,
							},
						)
						.await;
						Err(error)
					}
				}
			},
		)
		.await;
	Ok(job)
}
