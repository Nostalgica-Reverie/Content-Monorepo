use std::fs;
use std::path::{Path, PathBuf};

use packwand_pack::{HashFormat, hash_bytes};

use crate::InstallerError;

/// Content-addressed cache shared by repeated installer runs in one instance.
pub struct DownloadCache {
	root: PathBuf,
}

impl DownloadCache {
	pub fn new(instance: &Path) -> Self {
		Self {
			root: instance.join(".packwand-installer").join("cache"),
		}
	}

	pub fn get(&self, format: &str, hash: &str) -> Result<Option<Vec<u8>>, InstallerError> {
		if hash.is_empty() {
			return Ok(None);
		}
		let path = self.path(format, hash)?;
		let Ok(bytes) = fs::read(&path) else {
			return Ok(None);
		};
		let format = format
			.parse::<HashFormat>()
			.map_err(|error| InstallerError::Decode(error.to_string()))?;
		if hash_bytes(format, &bytes).eq_ignore_ascii_case(hash) {
			Ok(Some(bytes))
		} else {
			let _ = fs::remove_file(path);
			Ok(None)
		}
	}

	pub fn put(&self, format: &str, hash: &str, bytes: &[u8]) -> Result<(), InstallerError> {
		if hash.is_empty() {
			return Ok(());
		}
		let path = self.path(format, hash)?;
		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
		}
		let staging = path.with_extension("pw-part");
		fs::write(&staging, bytes)?;
		fs::rename(staging, path)?;
		Ok(())
	}

	fn path(&self, format: &str, hash: &str) -> Result<PathBuf, InstallerError> {
		if !format
			.chars()
			.all(|character| character.is_ascii_alphanumeric() || character == '-')
			|| !hash
				.chars()
				.all(|character| character.is_ascii_alphanumeric())
		{
			return Err(InstallerError::InvalidPath("invalid cache key".into()));
		}
		Ok(self.root.join(format).join(hash))
	}
}
