use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use packwand_instance::FsUserInstanceRepository;
use serde::Serialize;
use walkdir::WalkDir;

use crate::error::{OrchestratorError, Result};
use crate::paths::{backing_pack, normalized, safe_content_path};

/// The content directories an instance's mod list covers.
const CONTENT_DIRECTORIES: [&str; 4] = ["mods", "resourcepacks", "shaderpacks", "datapacks"];

/// One file in an instance's content directories.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceContent {
	pub path: String,
	pub name: String,
	pub enabled: bool,
	/// Whether the backing pack lists it. A hand-added jar is not exportable
	/// and is not removed by a reinstall, so the distinction has to be visible.
	pub pack_sourced: bool,
	pub bytes: u64,
}

/// Every path the backing pack installs, as the instance sees them.
///
/// A metafile entry names the metadata document, not the jar, so each one is
/// read to recover the filename the mod actually lands under.
pub fn pack_content_paths(pack_dir: &Path) -> Result<BTreeSet<String>> {
	let pack: packwand_pack::Pack =
		toml::from_str(&std::fs::read_to_string(pack_dir.join("pack.toml"))?)?;
	let index: packwand_pack::Index =
		serde_json::from_slice(&std::fs::read(pack_dir.join(&pack.index.file))?)?;
	let mut paths = BTreeSet::new();
	for entry in index.files {
		if entry.metafile && packwand_pack::metafile::is_metafile(&entry.file) {
			let metadata: packwand_pack::Mod =
				serde_json::from_slice(&std::fs::read(pack_dir.join(&entry.file))?)?;
			let parent = Path::new(&entry.file)
				.parent()
				.unwrap_or_else(|| Path::new(""));
			paths.insert(normalized(&parent.join(metadata.filename)));
		} else {
			paths.insert(normalized(Path::new(&entry.file)));
		}
	}
	Ok(paths)
}

/// Lists an instance's content, marking each item pack-sourced or hand-added.
pub fn list(repo: &FsUserInstanceRepository, id: &str) -> Result<Vec<InstanceContent>> {
	let instance = repo.get(id)?;
	let root = repo.instance_dir(id)?;
	let sourced = pack_content_paths(&backing_pack(repo, &instance)?)?;
	let mut items = Vec::new();
	for directory in CONTENT_DIRECTORIES {
		let start = root.join(directory);
		if !start.is_dir() {
			continue;
		}
		for entry in WalkDir::new(start)
			.follow_links(false)
			.into_iter()
			.filter_map(std::result::Result::ok)
		{
			if !entry.file_type().is_file() {
				continue;
			}
			let relative = entry.path().strip_prefix(&root).unwrap_or(entry.path());
			let raw = normalized(relative);
			let enabled = !raw.ends_with(".disabled");
			let logical = raw.strip_suffix(".disabled").unwrap_or(&raw).to_owned();
			items.push(InstanceContent {
				path: raw,
				name: Path::new(&logical)
					.file_name()
					.unwrap_or_default()
					.to_string_lossy()
					.into_owned(),
				enabled,
				// Matched on the enabled name, so a disabled mod is still
				// recognised as belonging to the pack.
				pack_sourced: sourced.contains(&logical),
				bytes: entry.metadata().map(|metadata| metadata.len()).unwrap_or(0),
			});
		}
	}
	items.sort_by(|left, right| left.path.cmp(&right.path));
	Ok(items)
}

/// Flips one file between enabled and disabled, returning its new path.
///
/// The `.disabled` suffix is the convention every other launcher uses, which
/// is what makes an instance readable by them and theirs by us.
pub fn toggle(repo: &FsUserInstanceRepository, id: &str, path: &str) -> Result<String> {
	repo.get(id)?;
	let root = repo.instance_dir(id)?;
	let source = safe_content_path(&root, path)?;
	if !source.is_file() {
		return Err(OrchestratorError::new(
			"not_found",
			"content file was not found",
		));
	}
	let target = if source
		.extension()
		.is_some_and(|extension| extension == "disabled")
	{
		source.with_extension("")
	} else {
		PathBuf::from(format!("{}.disabled", source.display()))
	};
	std::fs::rename(&source, &target)?;
	Ok(normalized(target.strip_prefix(&root).unwrap_or(&target)))
}

/// Deletes one content file.
pub fn remove(repo: &FsUserInstanceRepository, id: &str, path: &str) -> Result<()> {
	repo.get(id)?;
	let root = repo.instance_dir(id)?;
	let target = safe_content_path(&root, path)?;
	if !target.is_file() {
		return Err(OrchestratorError::new(
			"not_found",
			"content file was not found",
		));
	}
	std::fs::remove_file(target)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::lifecycle::{CreateSource, CreateSpec, create};

	#[test]
	fn toggling_round_trips_and_keeps_the_pack_marking() {
		let repo = FsUserInstanceRepository::new(tempfile::tempdir().unwrap().keep());
		let instance = create(
			&repo,
			CreateSpec {
				name: "Toggler".into(),
				source: CreateSource::Owned,
				pack_id: None,
				game_version: Some("1.21.1".into()),
				loader: Some("fabric".into()),
				loader_version: None,
			},
			|_| unreachable!(),
		)
		.unwrap();
		let mods = repo.instance_dir(&instance.id).unwrap().join("mods");
		std::fs::create_dir_all(&mods).unwrap();
		std::fs::write(mods.join("hand-added.jar"), b"jar").unwrap();

		let items = list(&repo, &instance.id).unwrap();
		assert_eq!(items.len(), 1);
		assert!(items[0].enabled);
		assert!(!items[0].pack_sourced, "not in the empty pack index");

		let disabled = toggle(&repo, &instance.id, "mods/hand-added.jar").unwrap();
		assert_eq!(disabled, "mods/hand-added.jar.disabled");
		let items = list(&repo, &instance.id).unwrap();
		assert!(!items[0].enabled);
		assert_eq!(items[0].name, "hand-added.jar", "name drops the suffix");

		let enabled = toggle(&repo, &instance.id, &disabled).unwrap();
		assert_eq!(enabled, "mods/hand-added.jar");
	}
}
