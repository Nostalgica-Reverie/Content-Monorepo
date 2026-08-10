//! pw4shell command-console integration.
//!
//! Rust owns the constrained grammar and built-ins. Unknown verbs are sent
//! only to Packwand's bundled CLI; no operating-system shell is involved.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellLine {
	pub text: String,
	pub tone: String,
}

impl ShellLine {
	fn info(text: impl Into<String>) -> Self {
		Self {
			text: text.into(),
			tone: "info".into(),
		}
	}
	fn error(text: impl Into<String>) -> Self {
		Self {
			text: text.into(),
			tone: "error".into(),
		}
	}
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellResult {
	pub lines: Vec<ShellLine>,
	pub handled: bool,
}

#[tauri::command]
pub async fn shell_exec(
	app: AppHandle,
	state: State<'_, AppState>,
	line: String,
	cwd: Option<String>,
) -> CommandResult<ShellResult> {
	match packwand_platform::shell_exec(&line) {
		Ok(packwand_platform::ShellOutcome::Handled(lines)) => Ok(ShellResult {
			lines: lines.into_iter().map(ShellLine::info).collect(),
			handled: true,
		}),
		Ok(packwand_platform::ShellOutcome::Empty) => Ok(ShellResult {
			lines: Vec::new(),
			handled: true,
		}),
		Ok(packwand_platform::ShellOutcome::ForHost(words)) => {
			let workspace = state.workspace()?;
			let cwd = validated_cwd(&workspace, cwd.as_deref())?;
			let executable = bundled_cli(&app)?;
			tauri::async_runtime::spawn_blocking(move || dispatch_cli(&executable, &cwd, &words))
				.await
				.map_err(|error| SerializableError::new("shell_join", error.to_string()))?
		}
		Err(error) => Ok(ShellResult {
			lines: vec![ShellLine::error(error.to_string())],
			handled: false,
		}),
	}
}

#[tauri::command]
pub fn shell_parse(line: String) -> CommandResult<Vec<String>> {
	packwand_platform::shell_parse(&line)
		.map_err(|error| SerializableError::new("shell", error.to_string()))
}

fn bundled_cli(app: &AppHandle) -> CommandResult<PathBuf> {
	let name = if cfg!(windows) {
		"packwand.exe"
	} else {
		"packwand"
	};
	let mut candidates = Vec::new();
	if let Ok(root) = app.path().resource_dir() {
		candidates.push(root.join(name));
		candidates.push(root.join("resources").join(name));
	}
	if let Ok(current) = std::env::current_exe()
		&& let Some(parent) = current.parent()
	{
		candidates.push(parent.join(name));
	}
	candidates
		.into_iter()
		.find(|path| path.is_file())
		.ok_or_else(|| {
			SerializableError::new("cli_unavailable", "the bundled Packwand CLI was not found")
		})
}

fn validated_cwd(workspace: &Path, requested: Option<&str>) -> CommandResult<PathBuf> {
	let workspace = workspace
		.canonicalize()
		.map_err(|error| SerializableError::new("workspace_unavailable", error.to_string()))?;
	let requested = requested
		.map(PathBuf::from)
		.map(|path| {
			if path.is_absolute() {
				path
			} else {
				workspace.join(path)
			}
		})
		.unwrap_or_else(|| workspace.clone());
	let requested = requested
		.canonicalize()
		.map_err(|error| SerializableError::new("folder_unavailable", error.to_string()))?;
	if !requested.starts_with(&workspace) {
		return Err(SerializableError::new(
			"folder_outside_workspace",
			"terminal folder is outside the workspace",
		));
	}
	Ok(requested)
}

fn dispatch_cli(executable: &Path, cwd: &Path, words: &[String]) -> CommandResult<ShellResult> {
	let args = match words.first().map(String::as_str) {
		Some("packwand") => &words[1..],
		_ => words,
	};
	let output = Command::new(executable)
		.args(args)
		.current_dir(cwd)
		.output()
		.map_err(|error| SerializableError::new("cli_start", error.to_string()))?;
	let mut lines = text_lines(&output.stdout, "info");
	lines.extend(text_lines(&output.stderr, "error"));
	if lines.is_empty() {
		lines.push(if output.status.success() {
			ShellLine {
				text: "command completed".into(),
				tone: "success".into(),
			}
		} else {
			ShellLine::error(format!("packwand exited with {}", output.status))
		});
	}
	Ok(ShellResult {
		lines,
		handled: output.status.success(),
	})
}

fn text_lines(bytes: &[u8], tone: &str) -> Vec<ShellLine> {
	String::from_utf8_lossy(bytes)
		.lines()
		.map(|text| ShellLine {
			text: text.to_owned(),
			tone: tone.to_owned(),
		})
		.collect()
}

#[cfg(test)]
mod tests {
	use super::{text_lines, validated_cwd};

	#[test]
	fn terminal_folder_must_stay_inside_workspace() {
		let workspace = tempfile::tempdir().unwrap();
		let project = workspace.path().join("modpacks").join("example");
		std::fs::create_dir_all(&project).unwrap();
		assert_eq!(
			validated_cwd(workspace.path(), Some("modpacks/example")).unwrap(),
			project.canonicalize().unwrap()
		);
		let outside = tempfile::tempdir().unwrap();
		assert!(validated_cwd(workspace.path(), outside.path().to_str()).is_err());
	}

	#[test]
	fn cli_output_is_split_into_terminal_lines() {
		assert_eq!(text_lines(b"one\ntwo\n", "info").len(), 2);
	}
}
