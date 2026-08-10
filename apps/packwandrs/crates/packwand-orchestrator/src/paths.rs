use std::path::{Component, Path, PathBuf};

use packwand_instance::{FsUserInstanceRepository, Instance, InstanceSource};

use crate::error::{OrchestratorError, Result};

/// A path in the form the UI and the pack index both use: forward slashes,
/// instance-relative.
pub fn normalized(path: &Path) -> String {
	path.to_string_lossy().replace('\\', "/")
}

/// The pack an instance's content comes from — a workspace directory for a
/// linked instance, its own private `.pack/` for a standalone one.
pub fn backing_pack(repo: &FsUserInstanceRepository, instance: &Instance) -> Result<PathBuf> {
	match &instance.source {
		InstanceSource::Linked { pack_dir } => Ok(pack_dir.clone()),
		InstanceSource::Owned => Ok(repo.owned_pack_dir(&instance.id)?),
	}
}

/// Resolves a caller-supplied relative path inside `root`, refusing anything
/// that could escape it.
///
/// The check is on components rather than on the joined result: `..` that
/// cancels out still indicates a caller doing something unintended, and
/// `starts_with` alone would accept it.
pub fn safe_content_path(root: &Path, relative: &str) -> Result<PathBuf> {
	let relative = Path::new(relative);
	if relative.as_os_str().is_empty()
		|| relative.is_absolute()
		|| relative
			.components()
			.any(|part| !matches!(part, Component::Normal(_)))
	{
		return Err(OrchestratorError::new(
			"unsafe_path",
			"content path must be instance-relative",
		));
	}
	let target = root.join(relative);
	if !target.starts_with(root) {
		return Err(OrchestratorError::new(
			"unsafe_path",
			"content path leaves the instance",
		));
	}
	Ok(target)
}

/// [`safe_content_path`], additionally refusing the two paths that are the
/// launcher's own state rather than game files a user should hand-edit.
pub fn safe_instance_file(root: &Path, relative: &str) -> Result<PathBuf> {
	let target = safe_content_path(root, relative)?;
	let first = Path::new(relative).components().next();
	if relative == "instance.json"
		|| first.is_some_and(|component| component.as_os_str() == ".pack")
	{
		return Err(OrchestratorError::new(
			"protected_path",
			"instance metadata and the backing pack are not editable game files",
		));
	}
	Ok(target)
}

/// Milliseconds since the Unix epoch, for the record's timestamps.
pub fn now_ms() -> u64 {
	std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap_or_default()
		.as_millis() as u64
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn content_paths_reject_traversal_and_absolute_paths() {
		let root = Path::new("instance");
		assert!(safe_content_path(root, "mods/a.jar").is_ok());
		assert!(safe_content_path(root, "../a.jar").is_err());
		assert!(safe_content_path(root, "C:\\a.jar").is_err());
		assert!(safe_content_path(root, "").is_err());
	}

	#[test]
	fn launcher_state_is_not_an_editable_game_file() {
		let root = Path::new("instance");
		assert!(safe_instance_file(root, "config/a.toml").is_ok());
		assert!(safe_instance_file(root, "instance.json").is_err());
		assert!(safe_instance_file(root, ".pack/pack.toml").is_err());
	}
}
