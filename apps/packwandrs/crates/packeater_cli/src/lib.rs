//! Trusted, embeddable PackEater optimizer used by Packwand and the CLI.

#![forbid(unsafe_code)]

mod config;

use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use config::PackeaterConfig;
use packsquash::{PackSquasher, vfs::os_fs::OsFilesystem};

/// Marker filename recognized by PackEater discovery.
pub const MARKER_NAME: &str = config::MARKER_NAME;

/// Optimize one marker into the host-selected destination without spawning a
/// process. The same PackSquash fork is compiled into Packwand and the CLI.
pub fn run_marker(marker: &Path, output: Option<&Path>) -> Result<u64, String> {
	if marker.file_name().and_then(|name| name.to_str()) != Some(MARKER_NAME) {
		return Err(format!(
			"JSON marker must be named {MARKER_NAME}: {}",
			marker.display()
		));
	}
	let pack_directory = marker
		.parent()
		.ok_or_else(|| format!("{} has no parent folder", marker.display()))?;
	let config = PackeaterConfig::read(marker)?;
	if !config.enabled {
		return Ok(0);
	}
	let output = output
		.map(Path::to_path_buf)
		.unwrap_or_else(|| config.output_path(pack_directory));
	let normalized_pack_directory = absolute_lexical(pack_directory)?;
	let output = absolute_lexical(&output)?;
	if output.starts_with(&normalized_pack_directory) {
		return Err(format!(
			"output {} must be outside the source pack folder {}",
			output.display(),
			pack_directory.display()
		));
	}
	if let Some(parent) = output.parent() {
		fs::create_dir_all(parent)
			.map_err(|error| format!("could not create {}: {error}", parent.display()))?;
	}
	let options = config.squash_options(&normalized_pack_directory, &output)?;
	PackSquasher::new()
		.run(OsFilesystem, options, None)
		.map_err(|error| error.to_string())?;
	fs::metadata(&output)
		.map(|metadata| metadata.len())
		.map_err(|error| format!("could not inspect {}: {error}", output.display()))
}

fn absolute_lexical(path: &Path) -> Result<PathBuf, String> {
	let path = if path.is_absolute() {
		path.to_path_buf()
	} else {
		env::current_dir()
			.map_err(|error| format!("could not resolve current directory: {error}"))?
			.join(path)
	};
	let mut normalized = PathBuf::new();
	for component in path.components() {
		match component {
			Component::CurDir => {}
			Component::ParentDir => {
				normalized.pop();
			}
			Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
				normalized.push(component.as_os_str())
			}
		}
	}
	Ok(normalized)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parent_output_is_normalized_outside_the_pack() {
		let pack = absolute_lexical(Path::new("packs/example")).unwrap();
		let output = absolute_lexical(Path::new("packs/example/../dist/result.zip")).unwrap();
		assert!(!output.starts_with(pack));
	}
}
