use std::path::{Path, PathBuf};
use std::process::Stdio;

use tauri::{AppHandle, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::commands::jobs::JobRecord;
use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

/// Mirrors `packwand-cli`'s own `dispatch/somnus.rs` binary search: an
/// explicit override, then the release build next to this app, then the Go
/// module's own working tree. Kept independent rather than shared because the
/// CLI's copy is private to that crate — duplicating a dozen lines here is
/// cheaper than exporting it across a crate boundary for one caller.
fn find_binary(root: &Path) -> CommandResult<PathBuf> {
	let executable = if cfg!(windows) { "somnus.exe" } else { "somnus" };
	let candidates = std::env::var_os("PACKWAND_SOMNUS_BIN")
		.map(PathBuf::from)
		.into_iter()
		.chain([
			root.join(executable),
			root.join("apps/packwandrs/target/release").join(executable),
			root.join("apps/packwandrs/somnus").join(executable),
		]);
	for candidate in candidates {
		if candidate.is_file() {
			return Ok(candidate);
		}
	}
	Err(SerializableError::new(
		"somnus_not_found",
		"somnus was not found; build apps/packwandrs/somnus or set PACKWAND_SOMNUS_BIN",
	))
}

async fn repository_root() -> CommandResult<PathBuf> {
	let output = Command::new("git")
		.args(["rev-parse", "--show-toplevel"])
		.output()
		.await?;
	if !output.status.success() {
		return Err(SerializableError::new(
			"not_a_repository",
			"Somnus must run inside a Git repository",
		));
	}
	let path = String::from_utf8(output.stdout)
		.map_err(|error| SerializableError::new("encoding", error.to_string()))?;
	Ok(PathBuf::from(path.trim()))
}

/// Runs every `.tangled/workflows/*.yml` whose `when.paths` match the
/// repository's current uncommitted changes — "approximately the same CI
/// locally that will run upstream," streamed into the ordinary job/log
/// surface rather than a bespoke panel, so Somnus runs show up next to every
/// other background operation instead of needing their own view.
#[tauri::command]
pub async fn somnus_run(
	workflow: Option<String>,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let root = repository_root().await?;
	let binary = find_binary(&root)?;
	let changed = Command::new("git")
		.args(["diff", "--name-only", "HEAD"])
		.current_dir(&root)
		.output()
		.await
		.ok()
		.and_then(|output| String::from_utf8(output.stdout).ok())
		.unwrap_or_default()
		.replace('\n', ",");
	let label = workflow
		.clone()
		.unwrap_or_else(|| "every matching workflow".to_owned());
	Ok(state
		.jobs
		.spawn(app, "somnus", format!("Somnus: {label}"), move |context| async move {
			let mut command = Command::new(&binary);
			command
				.arg("run")
				.args(["--root", &root.to_string_lossy()])
				.args(["--changed-paths", &changed])
				.stdin(Stdio::null())
				.stdout(Stdio::piped())
				.stderr(Stdio::piped());
			if let Some(workflow) = &workflow {
				command.arg(workflow);
			}
			let mut child = command.spawn()?;
			let stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
			let stderr = BufReader::new(child.stderr.take().expect("piped stderr"));
			let context_out = context.clone();
			let out_task = tokio::spawn(async move {
				let mut lines = stdout.lines();
				while let Ok(Some(line)) = lines.next_line().await {
					context_out.log(line).await;
				}
			});
			let context_err = context.clone();
			let err_task = tokio::spawn(async move {
				let mut lines = stderr.lines();
				while let Ok(Some(line)) = lines.next_line().await {
					context_err.log(line).await;
				}
			});
			let _ = out_task.await;
			let _ = err_task.await;
			let status = child.wait().await?;
			if !status.success() {
				return Err(SerializableError::new(
					"somnus_failed",
					format!("somnus exited with {status}"),
				));
			}
			context.progress(1.0, Some("Somnus run complete".into())).await;
			Ok(())
		})
		.await)
}

/// Discovered workflows and whether each would fire against the current
/// uncommitted changes — a quick, non-job call so the GUI can show this
/// before the user commits to running anything.
#[tauri::command]
pub async fn somnus_list(state: State<'_, AppState>) -> CommandResult<serde_json::Value> {
	let root = state.workspace().or_else(|_| {
		std::env::current_dir().map_err(|error| SerializableError::new("io", error.to_string()))
	})?;
	let root = repository_root().await.unwrap_or(root);
	let binary = find_binary(&root)?;
	let changed = Command::new("git")
		.args(["diff", "--name-only", "HEAD"])
		.current_dir(&root)
		.output()
		.await
		.ok()
		.and_then(|output| String::from_utf8(output.stdout).ok())
		.unwrap_or_default()
		.replace('\n', ",");
	let output = Command::new(&binary)
		.arg("list")
		.args(["--root", &root.to_string_lossy()])
		.args(["--changed-paths", &changed])
		.arg("--json")
		.output()
		.await?;
	if !output.status.success() {
		return Err(SerializableError::new(
			"somnus_failed",
			String::from_utf8_lossy(&output.stderr).into_owned(),
		));
	}
	serde_json::from_slice(&output.stdout)
		.map_err(|error| SerializableError::new("json", error.to_string()))
}
