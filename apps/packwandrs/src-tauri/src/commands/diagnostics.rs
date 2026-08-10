use packwand_diagnostics::{ContentRegistry, ValidationReport, VariantParityReport};
use tauri::{AppHandle, Manager, State};

use crate::commands::jobs::JobRecord;
use crate::commands::off_thread;
use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

#[tauri::command]
pub async fn diagnostics_lint(state: State<'_, AppState>) -> CommandResult<ValidationReport> {
	let root = state.workspace()?;
	off_thread(move || Ok(packwand_diagnostics::lint_workspace(root))).await
}

#[tauri::command]
pub async fn diagnostics_validate(state: State<'_, AppState>) -> CommandResult<ValidationReport> {
	let root = state.workspace()?;
	off_thread(move || Ok(packwand_diagnostics::validate_projects(root)?)).await
}

#[tauri::command]
pub async fn diagnostics_parity(
	state: State<'_, AppState>,
) -> CommandResult<Vec<VariantParityReport>> {
	let root = state.workspace()?;
	off_thread(move || Ok(packwand_diagnostics::parity_workspace(root)?)).await
}

#[tauri::command]
pub async fn diagnostics_content_lint(
	state: State<'_, AppState>,
) -> CommandResult<ValidationReport> {
	let root = state.workspace()?;
	off_thread(move || Ok(packwand_diagnostics::content_lint(root))).await
}

#[tauri::command]
pub async fn diagnostics_preflight(state: State<'_, AppState>) -> CommandResult<ValidationReport> {
	let root = state.workspace()?;
	off_thread(move || {
		let mut report = packwand_diagnostics::validate_projects(&root)?;
		let lint = packwand_diagnostics::lint_workspace(&root);
		report.checked += lint.checked;
		report.issues.extend(lint.issues);
		let content = packwand_diagnostics::content_lint(&root);
		report.checked += content.checked;
		report.issues.extend(content.issues);
		Ok(report)
	})
	.await
}

#[tauri::command]
pub async fn diagnostics_registries(
	state: State<'_, AppState>,
) -> CommandResult<Vec<ContentRegistry>> {
	let root = state.workspace()?;
	off_thread(move || {
		packwand_diagnostics::build_all_registries(root)
			.map_err(|error| SerializableError::new("registry", error.to_string()))
	})
	.await
}

#[tauri::command]
pub async fn diagnostics_installer_test(
	id: String,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let pack = pack_root(&state.workspace()?, &id)?;
	let instance = app
		.path()
		.app_cache_dir()
		.map_err(|error| SerializableError::new("path", error.to_string()))?
		.join("installer-test")
		.join(&id);
	Ok(state
		.jobs
		.spawn(
			app,
			"diagnostics.test",
			format!("Installer test {id}"),
			move |context| async move {
				context.log("Installing pack content").await;
				let report = tokio::task::spawn_blocking(move || {
					packwand_build::test_with_installer(pack, instance)
				})
				.await
				.map_err(|error| SerializableError::new("task", error.to_string()))?
				.map_err(|error| SerializableError::new("installer_test", error.to_string()))?;
				context
					.log(format!(
						"validated {} into {} ({} actions)",
						report.pack.display(),
						report.instance.display(),
						report.actions
					))
					.await;
				context
					.progress(1.0, Some("Installer validation passed".into()))
					.await;
				Ok(())
			},
		)
		.await)
}
