use packwand_instance::FsUserInstanceRepository;
use serde::Serialize;
use walkdir::WalkDir;

use crate::error::{OrchestratorError, Result};
use crate::paths::{normalized, safe_instance_file};

/// Above this, the editor refuses rather than pulling the file into the
/// webview as one string.
const MAX_EDITABLE_BYTES: u64 = 4 * 1024 * 1024;

/// One entry in the instance's file tree.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceFileEntry {
	pub path: String,
	pub name: String,
	pub directory: bool,
	pub size: u64,
}

/// The instance's whole tree, directories first then paths, with the
/// launcher's own state hidden.
pub fn list(repo: &FsUserInstanceRepository, id: &str) -> Result<Vec<InstanceFileEntry>> {
	repo.get(id)?;
	let root = repo.instance_dir(id)?;
	let mut entries = Vec::new();
	for entry in WalkDir::new(&root)
		.min_depth(1)
		.follow_links(false)
		.into_iter()
		.filter_map(std::result::Result::ok)
	{
		let relative = normalized(entry.path().strip_prefix(&root).unwrap_or(entry.path()));
		if relative == "instance.json" || relative.starts_with(".pack/") {
			continue;
		}
		let metadata = entry.metadata().ok();
		entries.push(InstanceFileEntry {
			name: entry.file_name().to_string_lossy().into_owned(),
			path: relative,
			directory: entry.file_type().is_dir(),
			size: metadata.map(|value| value.len()).unwrap_or(0),
		});
	}
	entries.sort_by(|left, right| {
		right
			.directory
			.cmp(&left.directory)
			.then_with(|| left.path.cmp(&right.path))
	});
	Ok(entries)
}

/// Reads one editable file as text.
pub fn read(repo: &FsUserInstanceRepository, id: &str, path: &str) -> Result<String> {
	repo.get(id)?;
	let root = repo.instance_dir(id)?;
	let target = safe_instance_file(&root, path)?;
	let metadata = std::fs::metadata(&target)?;
	if metadata.len() > MAX_EDITABLE_BYTES {
		return Err(OrchestratorError::new(
			"file_too_large",
			"files larger than 4 MiB cannot be edited",
		));
	}
	Ok(std::fs::read_to_string(target)?)
}

/// Writes one editable file, creating parent directories.
pub fn write(repo: &FsUserInstanceRepository, id: &str, path: &str, content: &str) -> Result<()> {
	repo.get(id)?;
	let root = repo.instance_dir(id)?;
	let target = safe_instance_file(&root, path)?;
	if let Some(parent) = target.parent() {
		std::fs::create_dir_all(parent)?;
	}
	std::fs::write(target, content)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::lifecycle::{CreateSource, CreateSpec, create};

	#[test]
	fn the_tree_hides_launcher_state_and_the_backing_pack() {
		let repo = FsUserInstanceRepository::new(tempfile::tempdir().unwrap().keep());
		let instance = create(
			&repo,
			CreateSpec {
				name: "Files".into(),
				source: CreateSource::Owned,
				pack_id: None,
				game_version: Some("1.21.1".into()),
				loader: None,
				loader_version: None,
			},
			|_| unreachable!(),
		)
		.unwrap();
		write(&repo, &instance.id, "config/a.toml", "x = 1").unwrap();

		let listed: Vec<_> = list(&repo, &instance.id)
			.unwrap()
			.into_iter()
			.map(|entry| entry.path)
			.collect();
		assert!(listed.contains(&"config/a.toml".to_owned()));
		assert!(!listed.iter().any(|path| path == "instance.json"));
		assert!(!listed.iter().any(|path| path.starts_with(".pack/")));

		assert_eq!(read(&repo, &instance.id, "config/a.toml").unwrap(), "x = 1");
		assert_eq!(
			write(&repo, &instance.id, "instance.json", "{}")
				.unwrap_err()
				.kind,
			"protected_path"
		);
	}
}
