use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub struct FileMutation {
	target: PathBuf,
	contents: Option<Vec<u8>>,
}

impl FileMutation {
	pub fn write(target: PathBuf, contents: Vec<u8>) -> Self {
		Self {
			target,
			contents: Some(contents),
		}
	}

	pub fn remove(target: PathBuf) -> Self {
		Self {
			target,
			contents: None,
		}
	}
}

pub struct FileTransaction {
	mutations: Vec<FileMutation>,
}

impl FileTransaction {
	pub fn new(mutations: Vec<FileMutation>) -> Self {
		Self { mutations }
	}

	pub fn commit(self) -> Result<(), TransactionError> {
		let mut targets = HashSet::new();
		for mutation in &self.mutations {
			if !targets.insert(mutation.target.clone()) {
				return Err(TransactionError::DuplicateTarget(mutation.target.clone()));
			}
		}

		// Stage every write before moving any existing file.
		let mut staged = Vec::with_capacity(self.mutations.len());
		for mutation in self.mutations {
			match stage(mutation) {
				Ok(mutation) => staged.push(mutation),
				Err(error) => {
					rollback(&mut staged);
					return Err(error);
				}
			}
		}

		for index in 0..staged.len() {
			let result = install(&mut staged[index]);
			if let Err(error) = result {
				rollback(&mut staged);
				return Err(error);
			}
		}
		for mutation in &staged {
			if let Some(backup) = &mutation.backup {
				let _ = fs::remove_file(backup);
			}
		}
		Ok(())
	}
}

fn stage(mutation: FileMutation) -> Result<StagedMutation, TransactionError> {
	let temp = match mutation.contents {
		Some(contents) => {
			let parent = mutation
				.target
				.parent()
				.ok_or_else(|| TransactionError::NoParent(mutation.target.clone()))?;
			fs::create_dir_all(parent).map_err(|source| TransactionError::Io {
				path: parent.to_path_buf(),
				operation: "create parent directory",
				source,
			})?;
			let temp = sibling_path(&mutation.target, "tmp");
			if let Err(source) = fs::write(&temp, contents) {
				let _ = fs::remove_file(&temp);
				return Err(TransactionError::Io {
					path: temp,
					operation: "stage write",
					source,
				});
			}
			Some(temp)
		}
		None => None,
	};
	Ok(StagedMutation {
		target: mutation.target,
		temp,
		backup: None,
		installed: false,
	})
}

struct StagedMutation {
	target: PathBuf,
	temp: Option<PathBuf>,
	backup: Option<PathBuf>,
	installed: bool,
}

fn install(mutation: &mut StagedMutation) -> Result<(), TransactionError> {
	if mutation.target.exists() {
		let backup = sibling_path(&mutation.target, "backup");
		fs::rename(&mutation.target, &backup).map_err(|source| TransactionError::Io {
			path: mutation.target.clone(),
			operation: "move existing file to rollback backup",
			source,
		})?;
		mutation.backup = Some(backup);
	}
	if let Some(temp) = &mutation.temp {
		fs::rename(temp, &mutation.target).map_err(|source| TransactionError::Io {
			path: mutation.target.clone(),
			operation: "install staged file",
			source,
		})?;
		mutation.installed = true;
	}
	Ok(())
}

fn rollback(staged: &mut [StagedMutation]) {
	for mutation in staged.iter_mut().rev() {
		if mutation.installed {
			let _ = fs::remove_file(&mutation.target);
		}
		if let Some(backup) = mutation.backup.take() {
			let _ = fs::rename(backup, &mutation.target);
		}
		if let Some(temp) = &mutation.temp {
			let _ = fs::remove_file(temp);
		}
	}
}

fn sibling_path(target: &Path, suffix: &str) -> PathBuf {
	let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
	let name = target
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("packwand");
	target.with_file_name(format!(
		".{name}.packwand-{}-{id}.{suffix}",
		std::process::id()
	))
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
	#[error("transaction contains duplicate target {0}")]
	DuplicateTarget(PathBuf),
	#[error("transaction target has no parent: {0}")]
	NoParent(PathBuf),
	#[error("failed to {operation} at {path}: {source}")]
	Io {
		path: PathBuf,
		operation: &'static str,
		source: std::io::Error,
	},
}

#[cfg(test)]
mod tests {
	use super::{FileMutation, FileTransaction};

	#[test]
	fn staging_failure_leaves_existing_targets_untouched() {
		let directory = tempfile::tempdir().unwrap();
		let original = directory.path().join("original.txt");
		std::fs::write(&original, b"before").unwrap();
		let blocked_parent = directory.path().join("not-a-directory");
		std::fs::write(&blocked_parent, b"block").unwrap();

		let transaction = FileTransaction::new(vec![
			FileMutation::write(original.clone(), b"after".to_vec()),
			FileMutation::write(blocked_parent.join("child"), b"never".to_vec()),
		]);
		assert!(transaction.commit().is_err());
		assert_eq!(std::fs::read(original).unwrap(), b"before");
	}
}
