use std::fs;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::BuildError;

pub const PACKEATER_MARKER: &str = "packeater.json";

fn descend(entry: &DirEntry) -> bool {
	entry.depth() == 0
		|| !matches!(
			entry.file_name().to_str(),
			Some(".git" | ".hg" | ".svn" | "node_modules" | "target")
		)
}

/// Find every Packeater marker below a folder in deterministic order.
pub fn discover_packeater_markers(root: impl AsRef<Path>) -> Result<Vec<PathBuf>, BuildError> {
	let root = root.as_ref();
	if !root.is_dir() {
		return Err(BuildError::InvalidPack(format!(
			"{} is not a directory",
			root.display()
		)));
	}
	let mut markers = WalkDir::new(root)
		.follow_links(false)
		.into_iter()
		.filter_entry(descend)
		.filter_map(|entry| match entry {
			Ok(entry)
				if entry.file_type().is_file()
					&& entry.file_name().to_str() == Some(PACKEATER_MARKER) =>
			{
				Some(Ok(entry.into_path()))
			}
			Ok(_) => None,
			Err(source) => Some(Err(BuildError::InvalidPack(format!(
				"Packeater folder discovery failed: {source}"
			)))),
		})
		.collect::<Result<Vec<_>, _>>()?;
	markers.sort();
	Ok(markers)
}

/// Run Packeater for one marker, forcing the artifact destination selected by Packwand.
pub fn run_packeater(
	marker: impl AsRef<Path>,
	output: impl AsRef<Path>,
) -> Result<u64, BuildError> {
	let marker = marker.as_ref();
	let output = output.as_ref();
	let parent = output.parent().unwrap_or_else(|| Path::new("."));
	fs::create_dir_all(parent).map_err(|source| BuildError::Io {
		path: parent.to_path_buf(),
		source,
	})?;
	packeater_cli::run_marker(marker, Some(output)).map_err(BuildError::Optimizer)
}

/// Archive a content folder, selecting Packeater whenever it opts in with a marker.
pub fn archive_content_directory(
	root: impl AsRef<Path>,
	output: impl AsRef<Path>,
) -> Result<u64, BuildError> {
	let root = root.as_ref();
	let marker = root.join(PACKEATER_MARKER);
	if marker.is_file() {
		run_packeater(marker, output)
	} else {
		crate::archive_directory(root, output)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn discovers_markers_and_skips_build_folders() {
		let root = tempfile::tempdir().unwrap();
		let marker = root.path().join("resourcepacks/example/packeater.json");
		let skipped = root.path().join("target/example/packeater.json");
		fs::create_dir_all(marker.parent().unwrap()).unwrap();
		fs::create_dir_all(skipped.parent().unwrap()).unwrap();
		fs::write(&marker, "{}").unwrap();
		fs::write(skipped, "{}").unwrap();
		assert_eq!(
			discover_packeater_markers(root.path()).unwrap(),
			vec![marker]
		);
	}
}
