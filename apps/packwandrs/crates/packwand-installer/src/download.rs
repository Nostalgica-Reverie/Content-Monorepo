use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use packwand_parallel::Jobs;
use packwand_providers::{HttpRequest, Transport};

use crate::InstallerError;
use crate::cache::DownloadCache;
use crate::index::verify;
use crate::plan::{InstallPlan, OverwriteMode, PlanAction};

/// Applies a resolved plan with verified, staged file replacement.
///
/// Removals run first and in order, because a later action may write to a
/// path an earlier one clears. Everything that needs bytes then runs
/// concurrently — a modpack is a hundred-odd independent fetches, and doing
/// them one at a time was the single largest cost of installing one.
pub fn apply(
	plan: &InstallPlan,
	instance: &Path,
	transport: &dyn Transport,
) -> Result<(), InstallerError> {
	apply_with_jobs(plan, instance, transport, packwand_parallel::configured())
}

/// [`apply`] with an explicit worker count, for callers that expose one.
pub fn apply_with_jobs(
	plan: &InstallPlan,
	instance: &Path,
	transport: &dyn Transport,
	jobs: Jobs,
) -> Result<(), InstallerError> {
	let cache = DownloadCache::new(instance);
	let previous = read_manifest(instance)?;
	let mut installed = Vec::new();
	let mut pending = Vec::new();

	for action in &plan.actions {
		if let PlanAction::Remove { target } = action {
			if target.is_file() {
				fs::remove_file(target)?;
			}
			continue;
		}
		let target = action.target();
		let relative = target
			.strip_prefix(instance)
			.map_err(|_| InstallerError::InvalidPath(target.display().to_string()))?;
		// Recorded in plan order whether or not it needs work, so the manifest
		// this run writes does not depend on which fetches finished first.
		installed.push(relative.to_path_buf());
		if action.overwrite() == Some(OverwriteMode::Preserve) && target.exists() {
			continue;
		}
		pending.push(action);
	}

	for result in packwand_parallel::try_map(&pending, jobs, |action| {
		materialize(action, &cache, transport)
	}) {
		result?;
	}

	let kept: HashSet<&PathBuf> = installed.iter().collect();
	for relative in &previous {
		if !kept.contains(relative) {
			let stale = instance.join(relative);
			if stale.is_file() {
				fs::remove_file(stale)?;
			}
		}
	}
	write_manifest(instance, &installed)?;
	Ok(())
}

/// Produces one action's bytes — from the cache, the network, or the pack
/// directory — verifies them, and commits them to the target.
fn materialize(
	action: &PlanAction,
	cache: &DownloadCache,
	transport: &dyn Transport,
) -> Result<(), InstallerError> {
	match action {
		PlanAction::Download {
			url,
			target,
			hash_format,
			hash,
			..
		} => {
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
			commit(target, &bytes)
		}
		PlanAction::Copy {
			source,
			target,
			hash_format,
			hash,
			..
		} => {
			// Not cached: the source is already a local file, so a cache entry
			// would only be a third copy of the same bytes.
			let bytes = fs::read(source)?;
			verify(&target.display().to_string(), hash_format, hash, &bytes)?;
			commit(target, &bytes)
		}
		PlanAction::Remove { .. } => Ok(()),
	}
}

/// A staging path for `target` that no concurrent write can collide with.
///
/// Two things make the obvious `with_extension("pw-part")` wrong now that
/// these run in parallel: it *replaces* the extension, so `x.json` and
/// `x.toml` stage to the same `x.pw-part`; and one content hash can be
/// referenced by two entries in a single plan. Both produce a silent
/// wrong-bytes result, because verification happens in memory before the
/// rename and the file is never read back. Appending, plus a per-process
/// counter, removes the collision by construction.
pub(crate) fn staging_path(target: &Path) -> PathBuf {
	static NEXT: AtomicU64 = AtomicU64::new(0);
	let ticket = NEXT.fetch_add(1, Ordering::Relaxed);
	let name = target
		.file_name()
		.map(|name| name.to_string_lossy().into_owned())
		.unwrap_or_default();
	target.with_file_name(format!("{name}.{ticket}.pw-part"))
}

/// Writes `bytes` to `target` through a staging file, so a target is either
/// its old contents or the new ones and never a half-written mix.
fn commit(target: &Path, bytes: &[u8]) -> Result<(), InstallerError> {
	if let Some(parent) = target.parent() {
		fs::create_dir_all(parent)?;
	}
	let staging = staging_path(target);
	fs::write(&staging, bytes)?;
	if target.exists() {
		fs::remove_file(target)?;
	}
	fs::rename(staging, target)?;
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
	let staging = staging_path(&path);
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

	/// Serves each URL its own bytes, so a staging-path collision shows up as
	/// a file holding some *other* action's content rather than as an error.
	struct PerUrlTransport;

	impl Transport for PerUrlTransport {
		fn get(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError> {
			Ok(request
				.url
				.rsplit('/')
				.next()
				.unwrap_or("")
				.as_bytes()
				.to_vec())
		}
	}

	#[test]
	fn targets_sharing_a_stem_do_not_share_a_staging_file() {
		let root = tempfile::tempdir().unwrap();
		// `with_extension("pw-part")` would stage both of these to
		// `config/shared.pw-part`; in parallel that silently swaps their bytes.
		let names = ["config/shared.json", "config/shared.toml", "config/shared"];
		let actions: Vec<_> = names
			.iter()
			.map(|name| {
				let payload = name.rsplit('/').next().unwrap().as_bytes().to_vec();
				PlanAction::Download {
					url: format!(
						"https://example.invalid/{}",
						name.rsplit('/').next().unwrap()
					),
					target: root.path().join(name),
					hash_format: "sha256".into(),
					hash: hash_bytes(HashFormat::Sha256, &payload),
					overwrite: OverwriteMode::Replace,
				}
			})
			.collect();

		apply(
			&InstallPlan {
				actions,
				manual: Vec::new(),
			},
			root.path(),
			&PerUrlTransport,
		)
		.unwrap();

		for name in names {
			let expected = name.rsplit('/').next().unwrap();
			assert_eq!(
				std::fs::read(root.path().join(name)).unwrap(),
				expected.as_bytes(),
				"{name} holds another action's bytes"
			);
		}
	}

	#[test]
	fn the_manifest_follows_plan_order_not_completion_order() {
		let root = tempfile::tempdir().unwrap();
		let bytes = b"payload".to_vec();
		let hash = hash_bytes(HashFormat::Sha256, &bytes);
		// Enough entries that workers genuinely interleave; the next run diffs
		// against this manifest, so a nondeterministic order would make stale
		// -file detection depend on scheduling.
		let actions: Vec<_> = (0..64)
			.map(|index| PlanAction::Download {
				url: format!("https://example.invalid/{index}"),
				target: root.path().join("mods").join(format!("{index:02}.jar")),
				hash_format: "sha256".into(),
				hash: hash.clone(),
				overwrite: OverwriteMode::Replace,
			})
			.collect();
		let expected: Vec<_> = actions
			.iter()
			.map(|action| {
				action
					.target()
					.strip_prefix(root.path())
					.unwrap()
					.to_path_buf()
			})
			.collect();
		let plan = InstallPlan {
			actions,
			manual: Vec::new(),
		};

		apply(&plan, root.path(), &MemoryTransport(bytes)).unwrap();

		let manifest: Vec<std::path::PathBuf> =
			serde_json::from_slice(&std::fs::read(super::manifest_path(root.path())).unwrap())
				.unwrap();
		assert_eq!(manifest, expected);
		for relative in &expected {
			assert!(root.path().join(relative).is_file());
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
