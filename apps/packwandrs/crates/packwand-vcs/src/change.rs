use std::path::Path;

use jj_lib::config::StackedConfig;
use jj_lib::settings::UserSettings;
use jj_lib::workspace::Workspace;
use pollster::FutureExt as _;

use crate::{Repository, StackEntry, VcsError, stack_log, with_repo};

/// Initializes Jujutsu alongside an existing Git repository.
pub fn enable_colocated(workspace_root: &Path) -> Result<(), VcsError> {
	if workspace_root.join(".jj").is_dir() {
		return Ok(());
	}
	if !workspace_root.join(".git").exists() {
		return Err(VcsError::InvalidInput(
			"colocated initialization requires an existing .git repository".into(),
		));
	}
	let settings = UserSettings::from_config(StackedConfig::with_defaults())
		.map_err(|error| VcsError::Library(error.to_string()))?;
	Workspace::init_external_git(&settings, workspace_root, &workspace_root.join(".git"))
		.block_on()
		.map_err(|error| VcsError::Library(error.to_string()))?;
	Ok(())
}

/// Creates and edits a new working-copy change.
pub fn new_change(workspace_root: &Path, parent: Option<&str>) -> Result<StackEntry, VcsError> {
	with_repo(workspace_root, |repository| {
		let mut args = vec!["new"];
		if let Some(parent) = parent.filter(|value| !value.trim().is_empty()) {
			ensure_unambiguous(repository, parent)?;
			args.push(parent);
		}
		repository.command(&args)?;
		stack_log(workspace_root)?
			.into_iter()
			.find(|entry| entry.is_working_copy)
			.ok_or_else(|| VcsError::InvalidOutput("new working-copy change was not found".into()))
	})
}

/// Replaces one change's description.
pub fn describe(workspace_root: &Path, change_id: &str, message: &str) -> Result<(), VcsError> {
	if message.trim().is_empty() {
		return Err(VcsError::InvalidInput("description cannot be empty".into()));
	}
	with_repo(workspace_root, |repository| {
		ensure_unambiguous(repository, change_id)?;
		repository.command(&["describe", "-r", change_id, "-m", message])?;
		Ok(())
	})
}

/// Squashes one change into its first parent.
pub fn squash(workspace_root: &Path, change_id: &str, _into_parent: bool) -> Result<(), VcsError> {
	with_repo(workspace_root, |repository| {
		ensure_unambiguous(repository, change_id)?;
		repository.command(&["squash", "-r", change_id])?;
		Ok(())
	})
}

fn ensure_unambiguous(repository: &Repository, change_id: &str) -> Result<(), VcsError> {
	if change_id.trim().is_empty() || change_id.starts_with('-') {
		return Err(VcsError::InvalidInput("invalid change ID".into()));
	}
	let output = repository.command(&[
		"log",
		"--no-graph",
		"-r",
		&format!("change_id({change_id})"),
		"-T",
		"commit_id ++ \"\\n\"",
	])?;
	let candidates = String::from_utf8_lossy(&output.stdout)
		.lines()
		.filter(|line| !line.is_empty())
		.map(str::to_owned)
		.collect::<Vec<_>>();
	match candidates.len() {
		0 => Err(VcsError::InvalidInput(format!(
			"change {change_id} was not found"
		))),
		1 => Ok(()),
		_ => Err(VcsError::Divergent {
			change_id: change_id.to_owned(),
			candidates,
		}),
	}
}
