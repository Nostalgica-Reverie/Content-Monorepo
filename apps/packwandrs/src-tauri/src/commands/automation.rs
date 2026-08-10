use std::process::Command;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, State};

use crate::commands::jobs::JobRecord;
use crate::error::{CommandResult, SerializableError};
use crate::events::emit_packs_changed;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationPlan {
	pub id: String,
	pub enabled: bool,
	pub version: String,
	pub next_version: String,
	pub tests: Vec<String>,
	pub subdirs: Vec<String>,
	pub steps: Vec<&'static str>,
}

fn full_auto(value: Option<&Value>) -> (bool, Vec<String>) {
	let enabled = value
		.and_then(|value| value.get("enabled"))
		.and_then(Value::as_bool)
		.unwrap_or(false);
	let tests = value
		.and_then(|value| value.get("tests"))
		.and_then(Value::as_array)
		.into_iter()
		.flatten()
		.filter_map(Value::as_str)
		.map(str::to_owned)
		.collect();
	(enabled, tests)
}

fn next_calver(current: &str) -> String {
	// Avoid a clock dependency in the frontend contract: UTC date is supplied
	// by the standard library through UNIX days and converted here.
	let seconds = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap_or_default()
		.as_secs() as i64;
	let z = seconds.div_euclid(86_400) + 719_468;
	let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
	let doe = z - era * 146_097;
	let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
	let mut year = yoe + era * 400;
	let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
	let mp = (5 * doy + 2) / 153;
	let month = mp + if mp < 10 { 3 } else { -9 };
	year += i64::from(month <= 2);
	let cycle = format!("{:02}.{month:02}", year % 100);
	let mut parts = current.split('.');
	let old_cycle = match (parts.next(), parts.next()) {
		(Some(year), Some(month)) => format!("{year}.{month}"),
		_ => String::new(),
	};
	if old_cycle != cycle {
		return cycle;
	}
	let patch = parts
		.next()
		.and_then(|value| value.parse::<u32>().ok())
		.unwrap_or(0)
		+ 1;
	format!("{cycle}.{patch}")
}

#[tauri::command]
pub fn automation_plan(id: String, state: State<'_, AppState>) -> CommandResult<AutomationPlan> {
	let project = packwand_workspace::find(state.workspace()?, &id)?;
	let (enabled, tests) = full_auto(
		project
			.manifest
			.automation
			.as_ref()
			.and_then(|value| value.full_auto.as_ref()),
	);
	Ok(AutomationPlan {
		id,
		enabled,
		next_version: next_calver(&project.manifest.version),
		version: project.manifest.version,
		tests,
		subdirs: project
			.subdirs
			.iter()
			.map(|path| path.to_string_lossy().into_owned())
			.collect(),
		steps: vec!["validate", "update", "sync", "refresh", "tests", "bump"],
	})
}

#[tauri::command]
pub async fn automation_run(
	id: String,
	dry_run: bool,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let root = state.workspace()?;
	let project = packwand_workspace::find(&root, &id)?;
	let plan = automation_plan(id.clone(), state.clone())?;
	if !plan.enabled {
		return Err(SerializableError::new(
			"automation_disabled",
			"automation.full_auto.enabled is not true",
		));
	}
	let changed_app = app.clone();
	Ok(state
		.jobs
		.spawn(
			app,
			"automation.run",
			format!(
				"{} automation for {id}",
				if dry_run { "Dry-run" } else { "Run" }
			),
			move |context| async move {
				context
					.log("Validating manifests before release preparation")
					.await;
				let validation = packwand_diagnostics::validate_projects(&root)?;
				if !validation.valid() {
					return Err(SerializableError::new(
						"validation",
						"manifest validation failed",
					));
				}
				context
					.progress(0.12, Some("Manifest validation passed".into()))
					.await;

				for (index, subdir) in project.subdirs.iter().enumerate() {
					if context.is_cancelled() {
						return Err(context.cancelled_error());
					}
					let subdir = subdir.clone();
					let records = tokio::task::spawn_blocking(move || {
						packwand_ops::update_latest(subdir, None, true, dry_run)
					})
					.await
					.map_err(|error| SerializableError::new("task", error.to_string()))??;
					let changed = records.iter().filter(|record| record.changed).count();
					let failed = records
						.iter()
						.filter(|record| {
							record
								.error
								.as_deref()
								.is_some_and(|error| error != "pinned")
						})
						.count();
					context
						.log(format!(
							"{}: {changed} update(s), {failed} failure(s)",
							project.subdirs[index].display()
						))
						.await;
					if failed > 0 {
						return Err(SerializableError::new(
							"update",
							format!(
								"provider updates failed in {}",
								project.subdirs[index].display()
							),
						));
					}
				}
				context
					.progress(0.48, Some("Provider updates resolved".into()))
					.await;

				let sync = packwand_workspace::sync_performance_bases(&root, dry_run)?;
				context
					.log(format!(
						"sync: {} copied, {} deleted{}",
						sync.copied,
						sync.deleted,
						if dry_run { " (dry-run)" } else { "" }
					))
					.await;
				context
					.progress(0.62, Some("Workspace synchronization complete".into()))
					.await;

				if !dry_run {
					for subdir in &project.subdirs {
						packwand_ops::Workspace::open(subdir.clone())?.refresh_metadata_index()?;
					}
					for test in &plan.tests {
						if context.is_cancelled() {
							return Err(context.cancelled_error());
						}
						context.log(format!("test: {test}")).await;
						#[cfg(windows)]
						let status = Command::new("cmd")
							.args(["/C", test])
							.current_dir(&project.root)
							.status()?;
						#[cfg(not(windows))]
						let status = Command::new("sh")
							.args(["-c", test])
							.current_dir(&project.root)
							.status()?;
						if !status.success() {
							return Err(SerializableError::new(
								"automation_test",
								format!("{test:?} failed with {status}"),
							));
						}
					}
					packwand_workspace::bump(&root, &id, &plan.next_version)?;
					emit_packs_changed(&changed_app)?;
				} else {
					context
						.log(format!(
							"would bump {} -> {}",
							plan.version, plan.next_version
						))
						.await;
				}
				context
					.progress(
						1.0,
						Some(
							if dry_run {
								"Automation dry-run passed"
							} else {
								"Ready to publish"
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
