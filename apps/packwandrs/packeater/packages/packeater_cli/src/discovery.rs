use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::config::MARKER_NAME;

fn descend(entry: &DirEntry) -> bool {
	entry.depth() == 0
		|| !matches!(
			entry.file_name().to_str(),
			Some(".git" | ".hg" | ".svn" | "node_modules" | "target")
		)
}

pub fn discover(root: &Path) -> Result<Vec<PathBuf>, String> {
	if !root.is_dir() {
		return Err(format!("{} is not a directory", root.display()));
	}
	let mut markers = WalkDir::new(root)
		.follow_links(false)
		.into_iter()
		.filter_entry(descend)
		.filter_map(|entry| match entry {
			Ok(entry)
				if entry.file_type().is_file() && entry.file_name().to_str() == Some(MARKER_NAME) =>
			{
				Some(Ok(entry.into_path()))
			}
			Ok(_) => None,
			Err(error) => Some(Err(format!("folder discovery failed: {error}")))
		})
		.collect::<Result<Vec<_>, _>>()?;
	markers.sort();
	Ok(markers)
}

#[cfg(test)]
mod tests {
	use std::fs;

	use super::*;

	#[test]
	fn finds_markers_but_skips_build_trees() {
		let root = tempfile::tempdir().unwrap();
		let wanted = root.path().join("packs/example/packeater.json");
		let skipped = root.path().join("target/example/packeater.json");
		fs::create_dir_all(wanted.parent().unwrap()).unwrap();
		fs::create_dir_all(skipped.parent().unwrap()).unwrap();
		fs::write(&wanted, "{}").unwrap();
		fs::write(skipped, "{}").unwrap();
		assert_eq!(discover(root.path()).unwrap(), vec![wanted]);
	}
}
