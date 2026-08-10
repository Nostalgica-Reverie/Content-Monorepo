use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use packwand_installer::InstallSide;
use packwand_ops::Workspace;
use packwand_parallel::Jobs;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub use packwand_installer::ManualDownload;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerTestReport {
	pub pack: PathBuf,
	pub instance: PathBuf,
	/// How many filesystem actions the plan carried, so a caller can say
	/// "installed 121 files" rather than only "succeeded".
	pub actions: usize,
	pub manual: Vec<ManualDownload>,
	pub success: bool,
}

fn manual_pending_path(game_dir: &Path) -> PathBuf {
	game_dir
		.join(".packwand-installer")
		.join("manual-pending.json")
}

/// Reads the manual-download backlog the last install left behind
/// (CurseForge files an author has disabled third-party distribution for), if
/// any. The install itself still succeeds without these — they're reported so
/// a GUI can prompt for them instead of the instance silently missing content.
pub fn manual_pending(game_dir: impl AsRef<Path>) -> Result<Vec<ManualDownload>> {
	let bytes = match fs::read(manual_pending_path(game_dir.as_ref())) {
		Ok(bytes) => bytes,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
		Err(error) => return Err(error.into()),
	};
	serde_json::from_slice(&bytes).map_err(|error| error.into())
}

/// Places a user-selected file for one pending manual download — the same
/// "point us at the jar you already downloaded" flow Prism uses for
/// CurseForge files that forbid third-party downloads. Verifies the file
/// matches what the pack expects before accepting it.
pub fn provide_manual_download(source: impl AsRef<Path>, pending: &ManualDownload) -> Result<()> {
	let bytes = fs::read(source.as_ref())?;
	packwand_installer::index::verify(
		&pending.target.display().to_string(),
		&pending.hash_format,
		&pending.hash,
		&bytes,
	)
	.map_err(|error| Box::new(error) as Box<dyn Error + Send + Sync>)?;
	if let Some(parent) = pending.target.parent() {
		fs::create_dir_all(parent)?;
	}
	fs::write(&pending.target, bytes)?;
	Ok(())
}

/// Installs a workspace pack into a game directory.
///
/// The pack is read from disk. Refreshing its index first is not optional:
/// under `packwand:27` the index is generated, so a stale one would install
/// stale content and look exactly like an edit that never took effect.
pub fn install_with_native_installer(
	pack: impl AsRef<Path>,
	instance: impl AsRef<Path>,
	jobs: Jobs,
) -> Result<InstallerTestReport> {
	let pack = pack.as_ref().canonicalize()?;
	Workspace::open(pack.clone())?.refresh_metadata_index()?;
	let instance = instance.as_ref().to_path_buf();
	fs::create_dir_all(&instance)?;
	let plan =
		packwand_installer::install_local_with_jobs(&pack, &instance, InstallSide::Client, jobs)?;
	Ok(InstallerTestReport {
		pack,
		instance,
		actions: plan.actions.len(),
		manual: plan.manual,
		success: true,
	})
}

/// Compatibility name used by `packwand test`.
pub fn test_with_installer(
	pack: impl AsRef<Path>,
	instance: impl AsRef<Path>,
) -> Result<InstallerTestReport> {
	// `configured()`, not `Jobs::default()`: the CLI sets this from `--jobs`,
	// and the default would silently ignore the flag it advertises.
	install_with_native_installer(pack, instance, packwand_parallel::configured())
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;

	use super::install_with_native_installer;
	use packwand_ops::Workspace;
	use packwand_parallel::Jobs;

	fn write_pack(root: &std::path::Path) {
		let metadata = packwand_pack::Pack {
			name: "Built-in launcher fixture".into(),
			version: "1.0.0".into(),
			pack_format: packwand_pack::CURRENT_PACK_FORMAT.into(),
			versions: BTreeMap::from([("minecraft".into(), "1.21.1".into())]),
			..Default::default()
		};
		std::fs::write(
			root.join("pack.toml"),
			toml::to_string_pretty(&metadata).unwrap(),
		)
		.unwrap();
		std::fs::write(
			root.join(packwand_pack::metafile::INDEX_FILE),
			serde_json::to_vec_pretty(&packwand_pack::Index::default()).unwrap(),
		)
		.unwrap();
		Workspace::open(root.to_path_buf())
			.unwrap()
			.refresh_metadata_index()
			.unwrap();
	}

	#[test]
	fn pack_overrides_are_copied_into_the_instance() {
		let pack = tempfile::tempdir().unwrap();
		let instance = tempfile::tempdir().unwrap();
		let config = pack.path().join("config").join("fixture.txt");
		std::fs::create_dir_all(config.parent().unwrap()).unwrap();
		std::fs::write(&config, b"native launcher contract\n").unwrap();
		write_pack(pack.path());

		let report =
			install_with_native_installer(pack.path(), instance.path(), Jobs::default()).unwrap();
		assert!(report.success);
		assert_eq!(
			std::fs::read(instance.path().join("config").join("fixture.txt")).unwrap(),
			b"native launcher contract\n"
		);
	}

	#[test]
	fn a_pack_edited_since_the_last_refresh_still_installs_the_new_bytes() {
		let pack = tempfile::tempdir().unwrap();
		let instance = tempfile::tempdir().unwrap();
		let config = pack.path().join("config").join("fixture.txt");
		std::fs::create_dir_all(config.parent().unwrap()).unwrap();
		std::fs::write(&config, b"first\n").unwrap();
		write_pack(pack.path());
		install_with_native_installer(pack.path(), instance.path(), Jobs::default()).unwrap();

		// The index is a generated artifact; without a refresh its recorded
		// hash still describes the old bytes and the install is a no-op that
		// looks like the edit never happened.
		std::fs::write(&config, b"second\n").unwrap();
		install_with_native_installer(pack.path(), instance.path(), Jobs::default()).unwrap();
		assert_eq!(
			std::fs::read(instance.path().join("config").join("fixture.txt")).unwrap(),
			b"second\n"
		);
	}
}
