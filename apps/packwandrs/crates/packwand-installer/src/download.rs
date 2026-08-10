use std::fs;
use std::path::{Component, Path};

use packwand_providers::{HttpRequest, Transport};

use crate::InstallerError;
use crate::cache::DownloadCache;
use crate::index::verify;
use crate::plan::{InstallPlan, OverwriteMode, PlanAction};

/// Applies a resolved plan with verified, staged file replacement.
pub fn apply(
	plan: &InstallPlan,
	instance: &Path,
	transport: &dyn Transport,
) -> Result<(), InstallerError> {
	let cache = DownloadCache::new(instance);
	let previous = read_manifest(instance)?;
	let mut installed = Vec::new();
	for action in &plan.actions {
		match action {
			PlanAction::Remove { target } => {
				if target.is_file() {
					fs::remove_file(target)?;
				}
			}
			PlanAction::Download {
				url,
				target,
				hash_format,
				hash,
				overwrite,
			} => {
				let relative = target
					.strip_prefix(instance)
					.map_err(|_| InstallerError::InvalidPath(target.display().to_string()))?;
				installed.push(relative.to_path_buf());
				if *overwrite == OverwriteMode::Preserve && target.exists() {
					continue;
				}
				let bytes = match cache.get(hash_format, hash)? {
					Some(bytes) => bytes,
					None => {
						let bytes = transport
							.get_large(HttpRequest::get(url))
							.map_err(|error| InstallerError::Transport(error.to_string()))?;
						verify(&target.display().to_string(), hash_format, hash, &bytes)?;
						cache.put(hash_format, hash, &bytes)?;
						bytes
					}
				};
				if let Some(parent) = target.parent() {
					fs::create_dir_all(parent)?;
				}
				let staging = target.with_extension("pw-part");
				fs::write(&staging, bytes)?;
				if target.exists() {
					fs::remove_file(target)?;
				}
				fs::rename(staging, target)?;
			}
		}
	}
	for relative in previous {
		if !installed.contains(&relative) {
			let stale = instance.join(&relative);
			if stale.is_file() {
				fs::remove_file(stale)?;
			}
		}
	}
	write_manifest(instance, &installed)?;
	Ok(())
}

fn read_manifest(instance: &Path) -> Result<Vec<std::path::PathBuf>, InstallerError> {
	match fs::read(manifest_path(instance)) {
		Ok(bytes) => {
			let paths: Vec<std::path::PathBuf> = serde_json::from_slice(&bytes)
				.map_err(|error| InstallerError::Decode(error.to_string()))?;
			for path in &paths {
				if path.as_os_str().is_empty()
					|| path
						.components()
						.any(|component| !matches!(component, Component::Normal(_)))
				{
					return Err(InstallerError::InvalidPath(path.display().to_string()));
				}
			}
			Ok(paths)
		}
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
		Err(error) => Err(error.into()),
	}
}

fn write_manifest(instance: &Path, installed: &[std::path::PathBuf]) -> Result<(), InstallerError> {
	let path = manifest_path(instance);
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	let bytes = serde_json::to_vec_pretty(installed)
		.map_err(|error| InstallerError::Decode(error.to_string()))?;
	let staging = path.with_extension("pw-part");
	fs::write(&staging, bytes)?;
	if path.exists() {
		fs::remove_file(&path)?;
	}
	fs::rename(staging, path)?;
	Ok(())
}

fn manifest_path(instance: &Path) -> std::path::PathBuf {
	instance.join(".packwand-installer").join("manifest.json")
}

#[cfg(test)]
mod tests {
	use super::apply;
	use crate::plan::{InstallPlan, OverwriteMode, PlanAction};
	use packwand_pack::{HashFormat, hash_bytes};
	use packwand_providers::{HttpRequest, Transport, TransportError};

	struct MemoryTransport(Vec<u8>);

	impl Transport for MemoryTransport {
		fn get(&self, _request: HttpRequest) -> Result<Vec<u8>, TransportError> {
			Ok(self.0.clone())
		}
	}

	#[test]
	fn verifies_and_installs_every_supported_hash_format() {
		let root = tempfile::tempdir().unwrap();
		let bytes = b"native installer".to_vec();
		for format in [
			HashFormat::Sha1,
			HashFormat::Sha256,
			HashFormat::Sha512,
			HashFormat::Md5,
			HashFormat::Murmur2,
			HashFormat::LengthBytes,
		] {
			let target = root
				.path()
				.join("mods")
				.join(format!("{}.jar", format.as_str()));
			let plan = InstallPlan {
				actions: vec![PlanAction::Download {
					url: "https://example.invalid/mod".into(),
					target: target.clone(),
					hash_format: format.as_str().into(),
					hash: hash_bytes(format, &bytes),
					overwrite: OverwriteMode::Replace,
				}],
				manual: Vec::new(),
			};
			apply(&plan, root.path(), &MemoryTransport(bytes.clone())).unwrap();
			assert_eq!(std::fs::read(target).unwrap(), bytes);
		}
	}

	#[test]
	fn removes_files_dropped_by_the_next_plan() {
		let root = tempfile::tempdir().unwrap();
		let bytes = b"old mod".to_vec();
		let target = root.path().join("mods/old.jar");
		let plan = InstallPlan {
			actions: vec![PlanAction::Download {
				url: "https://example.invalid/old".into(),
				target: target.clone(),
				hash_format: "sha256".into(),
				hash: hash_bytes(HashFormat::Sha256, &bytes),
				overwrite: OverwriteMode::Replace,
			}],
			manual: Vec::new(),
		};
		apply(&plan, root.path(), &MemoryTransport(bytes)).unwrap();
		apply(
			&InstallPlan::default(),
			root.path(),
			&MemoryTransport(Vec::new()),
		)
		.unwrap();
		assert!(!target.exists());
	}

	#[test]
	fn rejects_manifest_paths_that_escape_the_instance() {
		let root = tempfile::tempdir().unwrap();
		let state = root.path().join(".packwand-installer");
		std::fs::create_dir_all(&state).unwrap();
		std::fs::write(state.join("manifest.json"), b"[\"../outside.jar\"]").unwrap();
		let error = apply(
			&InstallPlan::default(),
			root.path(),
			&MemoryTransport(Vec::new()),
		)
		.unwrap_err();
		assert!(matches!(error, crate::InstallerError::InvalidPath(_)));
	}
}
