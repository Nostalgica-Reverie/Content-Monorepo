use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{OnceLock, RwLock};

use jj_lib::config::StackedConfig;
use jj_lib::repo::StoreFactories;
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{Workspace, default_working_copy_factories};

/// Failures from opening or mutating a Jujutsu workspace.
#[derive(Debug, thiserror::Error)]
pub enum VcsError {
	#[error("{0} is not a Jujutsu workspace")]
	NotInitialized(PathBuf),
	#[error("Jujutsu executable was not found; set PACKWAND_JJ_BIN")]
	JjNotFound,
	#[error("Jujutsu repository changed concurrently: {0}")]
	Concurrent(String),
	#[error("change {change_id} is divergent: {candidates:?}")]
	Divergent {
		change_id: String,
		candidates: Vec<String>,
	},
	#[error("Jujutsu command failed: {0}")]
	Command(String),
	#[error("invalid Jujutsu output: {0}")]
	InvalidOutput(String),
	#[error("invalid change request: {0}")]
	InvalidInput(String),
	#[error("could not open Jujutsu through jj-lib: {0}")]
	Library(String),
	#[error(transparent)]
	Io(#[from] std::io::Error),
}

/// A short-lived workspace descriptor passed only inside [`with_repo`].
pub struct Repository {
	root: PathBuf,
	jj: PathBuf,
	_workspace: Option<Workspace>,
}

static CONFIGURED_JJ: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();

/// Configures the managed Jujutsu executable used by subsequent short-lived
/// operations without mutating the process environment.
pub fn configure_jj_binary(path: PathBuf) -> Result<(), VcsError> {
	if !path.is_file() {
		return Err(VcsError::JjNotFound);
	}
	*CONFIGURED_JJ
		.get_or_init(|| RwLock::new(None))
		.write()
		.map_err(|_| VcsError::Concurrent("configured tool lock was poisoned".into()))? = Some(path);
	Ok(())
}

impl Repository {
	pub(crate) fn open(root: &Path, require_initialized: bool) -> Result<Self, VcsError> {
		let root = root.canonicalize()?;
		if require_initialized && !root.join(".jj").is_dir() {
			return Err(VcsError::NotInitialized(root));
		}
		let workspace = if require_initialized {
			let settings = UserSettings::from_config(StackedConfig::with_defaults())
				.map_err(|error| VcsError::Library(error.to_string()))?;
			Some(
				Workspace::load(
					&settings,
					&root,
					&StoreFactories::default(),
					&default_working_copy_factories(),
				)
				.map_err(|error| classify_library_error(error.to_string()))?,
			)
		} else {
			None
		};
		Ok(Self {
			root,
			jj: find_jj()?,
			_workspace: workspace,
		})
	}

	pub(crate) fn command(&self, args: &[&str]) -> Result<Output, VcsError> {
		let output = Command::new(&self.jj)
			.args(["--no-pager", "--color=never"])
			.args(args)
			.current_dir(&self.root)
			.output()?;
		if output.status.success() {
			return Ok(output);
		}
		let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
		Err(classify_command_error(message))
	}
}

/// Opens a workspace for one operation and drops it immediately afterward.
pub fn with_repo<T>(
	root: &Path,
	operation: impl FnOnce(&Repository) -> Result<T, VcsError>,
) -> Result<T, VcsError> {
	let repository = Repository::open(root, true)?;
	operation(&repository)
}

fn find_jj() -> Result<PathBuf, VcsError> {
	if let Some(configured) = CONFIGURED_JJ
		.get_or_init(|| RwLock::new(None))
		.read()
		.map_err(|_| VcsError::Concurrent("configured tool lock was poisoned".into()))?
		.as_ref()
		.filter(|path| path.is_file())
	{
		return Ok(configured.clone());
	}
	if let Some(configured) = std::env::var_os("PACKWAND_JJ_BIN") {
		let path = PathBuf::from(configured);
		if path.is_file() {
			return Ok(path);
		}
	}
	let executable = if cfg!(windows) { "jj.exe" } else { "jj" };
	if let Some(paths) = std::env::var_os("PATH") {
		for directory in std::env::split_paths(&paths) {
			let candidate = directory.join(executable);
			if candidate.is_file() {
				return Ok(candidate);
			}
		}
	}
	Err(VcsError::JjNotFound)
}

fn classify_command_error(message: String) -> VcsError {
	if is_concurrency_error(&message) {
		VcsError::Concurrent(message)
	} else {
		VcsError::Command(message)
	}
}

fn classify_library_error(message: String) -> VcsError {
	if is_concurrency_error(&message) {
		VcsError::Concurrent(message)
	} else {
		VcsError::Library(message)
	}
}

fn is_concurrency_error(message: &str) -> bool {
	let lower = message.to_ascii_lowercase();
	lower.contains("concurrent")
		|| lower.contains("stale operation")
		|| lower.contains("operation was updated")
		|| lower.contains("operation head") && lower.contains("changed")
}

#[cfg(test)]
mod tests {
	use super::{VcsError, classify_command_error, classify_library_error};

	#[test]
	fn maps_cli_and_library_contention_to_typed_errors() {
		assert!(matches!(
			classify_command_error("stale operation: operation was updated".into()),
			VcsError::Concurrent(_)
		));
		assert!(matches!(
			classify_library_error("operation head changed concurrently".into()),
			VcsError::Concurrent(_)
		));
	}

	#[test]
	fn preserves_non_contention_error_sources() {
		assert!(matches!(
			classify_command_error("unknown revision".into()),
			VcsError::Command(_)
		));
		assert!(matches!(
			classify_library_error("unsupported repository".into()),
			VcsError::Library(_)
		));
	}
}
