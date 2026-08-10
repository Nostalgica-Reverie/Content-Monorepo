use std::path::{Path, PathBuf};

use packwand_instance::{FsUserInstanceRepository, InstallStage, Instance};
use serde::Serialize;
use walkdir::WalkDir;

use crate::error::{OrchestratorError, Result};
use crate::paths::{backing_pack, normalized, safe_content_path};

/// A CurseForge file the author has excluded from third-party distribution —
/// the install still finishes, but this one mod needs a human.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingManualDownload {
	pub name: String,
	/// Instance-relative, matching every other content path.
	pub target: String,
	pub page_url: Option<String>,
}

/// Every `.disabled` file currently under `root`.
pub fn disabled_files(root: &Path) -> Vec<PathBuf> {
	WalkDir::new(root)
		.follow_links(false)
		.into_iter()
		.filter_map(std::result::Result::ok)
		.filter(|entry| entry.file_type().is_file())
		.map(|entry| entry.into_path())
		.filter(|path| {
			path.extension()
				.is_some_and(|extension| extension == "disabled")
		})
		.collect()
}

/// Re-applies the disabled state a reinstall just undid.
///
/// The installer sees a disabled `foo.jar.disabled` as `foo.jar` missing and
/// downloads it again, leaving both files present and the mod switched back
/// on. Without this, "disable a mod, then reinstall" silently re-enables it,
/// which reads as the disable button being broken.
pub fn restore_disabled(root: &Path, disabled: &[PathBuf]) -> Result<()> {
	for path in disabled {
		if !path.starts_with(root) || !path.is_file() {
			continue;
		}
		let enabled = path.with_extension("");
		if enabled.is_file() {
			std::fs::remove_file(enabled)?;
		}
	}
	Ok(())
}

/// Installs an instance's backing pack into its game directory, at
/// `default_jobs` concurrent downloads unless the instance overrides it.
pub fn install(repo: &FsUserInstanceRepository, id: &str, default_jobs: usize) -> Result<Instance> {
	let requested = repo
		.get(id)
		.ok()
		.and_then(|instance| instance.settings.download_jobs)
		.unwrap_or(default_jobs);
	let jobs = packwand_parallel::Jobs::new(requested);
	install_with(repo, id, |pack_dir, game_dir| {
		packwand_build::install_with_native_installer(pack_dir, game_dir, jobs)
			.map(|_| ())
			.map_err(|error| OrchestratorError::new("installer", error))
	})
}

/// [`install`] with the installation step supplied, so tests can drive the
/// stage transitions without a network.
pub fn install_with(
	repo: &FsUserInstanceRepository,
	id: &str,
	install: impl FnOnce(&Path, &Path) -> Result<()>,
) -> Result<Instance> {
	let mut instance = repo.get(id)?;
	instance.stage = InstallStage::Installing;
	repo.write(&instance)?;
	let game_dir = repo.instance_dir(id)?;
	let pack_dir = backing_pack(repo, &instance)?;
	let disabled = disabled_files(&game_dir);
	let result =
		install(&pack_dir, &game_dir).and_then(|()| restore_disabled(&game_dir, &disabled));
	match result {
		Ok(()) => instance.stage = InstallStage::Ready,
		Err(error) => {
			instance.stage = InstallStage::Failed {
				message: error.message.clone(),
			};
			// Best effort: the install error is what the caller needs, and a
			// failure to record it must not mask that.
			let _ = repo.write(&instance);
			return Err(error);
		}
	}
	repo.write(&instance)?;
	Ok(instance)
}

/// Mods the last install could not fetch, with instance-relative targets.
pub fn manual_pending(
	repo: &FsUserInstanceRepository,
	id: &str,
) -> Result<Vec<PendingManualDownload>> {
	repo.get(id)?;
	let game_dir = repo.instance_dir(id)?;
	let pending = packwand_build::manual_pending(&game_dir)
		.map_err(|error| OrchestratorError::new("installer", error))?;
	Ok(pending
		.into_iter()
		.map(|entry| PendingManualDownload {
			name: entry.name,
			target: normalized(
				entry
					.target
					.strip_prefix(&game_dir)
					.unwrap_or(&entry.target),
			),
			page_url: entry.page_url,
		})
		.collect())
}

/// Accepts a file the user downloaded by hand for one pending mod, after
/// checking it is really that mod.
pub fn provide_manual(
	repo: &FsUserInstanceRepository,
	id: &str,
	target: &str,
	source: &Path,
) -> Result<()> {
	let game_dir = repo.instance_dir(id)?;
	safe_content_path(&game_dir, target)?;
	let pending = packwand_build::manual_pending(&game_dir)
		.map_err(|error| OrchestratorError::new("installer", error))?;
	let entry = pending
		.into_iter()
		.find(|entry| {
			normalized(
				entry
					.target
					.strip_prefix(&game_dir)
					.unwrap_or(&entry.target),
			) == target
		})
		.ok_or_else(|| {
			OrchestratorError::new("not_found", "no pending manual download at that path")
		})?;
	packwand_build::provide_manual_download(source, &entry)
		.map_err(|error| OrchestratorError::new("installer", error))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::lifecycle::{CreateSource, CreateSpec, create};

	fn owned_instance(repo: &FsUserInstanceRepository, name: &str) -> Instance {
		create(
			repo,
			CreateSpec {
				name: name.into(),
				source: CreateSource::Owned,
				pack_id: None,
				game_version: Some("1.21.1".into()),
				loader: Some("fabric".into()),
				loader_version: None,
			},
			|_| unreachable!(),
		)
		.unwrap()
	}

	#[test]
	fn disabled_content_survives_a_reinstall() {
		let root = tempfile::tempdir().unwrap();
		let disabled = root.path().join("mods/example.jar.disabled");
		std::fs::create_dir_all(disabled.parent().unwrap()).unwrap();
		std::fs::write(&disabled, b"disabled").unwrap();
		// What a reinstall does: sees `example.jar` missing and fetches it.
		std::fs::write(root.path().join("mods/example.jar"), b"fresh").unwrap();

		restore_disabled(root.path(), std::slice::from_ref(&disabled)).unwrap();
		assert!(disabled.is_file());
		assert!(!root.path().join("mods/example.jar").exists());
	}

	#[test]
	fn a_failed_install_records_why_and_a_retry_can_recover() {
		let repo = FsUserInstanceRepository::new(tempfile::tempdir().unwrap().keep());
		let instance = owned_instance(&repo, "Recoverable");

		let error = install_with(&repo, &instance.id, |_, _| {
			Err(OrchestratorError::new(
				"installer",
				"native installer failed",
			))
		})
		.unwrap_err();
		assert_eq!(error.kind, "installer");
		assert_eq!(
			repo.get(&instance.id).unwrap().stage,
			InstallStage::Failed {
				message: "native installer failed".into()
			}
		);

		let recovered = install_with(&repo, &instance.id, |_, _| Ok(())).unwrap();
		assert_eq!(recovered.stage, InstallStage::Ready);
	}
}
