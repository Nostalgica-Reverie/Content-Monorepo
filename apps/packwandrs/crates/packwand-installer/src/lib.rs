//! Native, verified installation of Packwand and packwiz content packs.

#![forbid(unsafe_code)]

pub mod cache;
pub mod download;
pub mod index;
pub mod plan;
pub mod self_update;

use std::path::Path;

use packwand_parallel::Jobs;
use packwand_providers::UreqTransport;

pub use plan::{InstallPlan, InstallSide, ManualDownload, OverwriteMode, PlanAction};

/// Failures while loading, planning, or applying a pack.
#[derive(Debug, thiserror::Error)]
pub enum InstallerError {
	#[error("invalid pack URL: {0}")]
	InvalidUrl(String),
	#[error("invalid pack path: {0}")]
	InvalidPath(String),
	#[error("failed to fetch pack content: {0}")]
	Transport(String),
	#[error("failed to decode pack content: {0}")]
	Decode(String),
	#[error("hash mismatch for {path}: expected {expected}, got {actual}")]
	HashMismatch {
		path: String,
		expected: String,
		actual: String,
	},
	#[error("provider resolution failed: {0}")]
	Provider(String),
	/// A CurseForge author has disabled third-party distribution for this
	/// file — a real, permanent platform state, not a transient failure.
	/// [`plan::build`] catches this variant itself and turns it into a
	/// [`ManualDownload`] entry rather than aborting the whole plan.
	#[error("{name} forbids third-party downloads on CurseForge")]
	ManualDownloadRequired {
		name: String,
		page_url: Option<String>,
	},
	#[error(transparent)]
	Io(#[from] std::io::Error),
}

/// Loads, plans, and applies one remote pack to the current instance.
///
/// Mods a CurseForge author has excluded from third-party distribution are
/// not fetched; they're listed in the returned plan's
/// [`InstallPlan::manual`] instead of aborting the rest of the install.
pub fn install(
	pack_url: &str,
	instance: &Path,
	side: InstallSide,
) -> Result<InstallPlan, InstallerError> {
	let transport = UreqTransport::for_downloads();
	let source = index::RemotePack::load(pack_url, &transport)?;
	let plan = plan::build(&source, instance, side, &transport)?;
	download::apply(&plan, instance, &transport)?;
	Ok(plan)
}

/// Applies a pack that is already on this machine.
///
/// Same result as [`install`], without serving the pack to ourselves over
/// loopback HTTP: metafiles and overrides are read from `pack_dir` directly
/// and only mod downloads touch the network.
///
/// **The caller must refresh the pack's index first.** Under `packwand:27`
/// the index is a generated artifact, so a stale one here installs stale
/// content — and that failure looks exactly like an edit not taking effect.
/// `packwand_build::install_with_native_installer` does this refresh.
pub fn install_local(
	pack_dir: &Path,
	instance: &Path,
	side: InstallSide,
) -> Result<InstallPlan, InstallerError> {
	install_local_with_jobs(pack_dir, instance, side, Jobs::default())
}

/// [`install_local`] with an explicit download width.
pub fn install_local_with_jobs(
	pack_dir: &Path,
	instance: &Path,
	side: InstallSide,
	jobs: Jobs,
) -> Result<InstallPlan, InstallerError> {
	let transport = UreqTransport::for_downloads();
	let source = index::LocalPack::load(pack_dir)?;
	let plan = plan::build(&source, instance, side, &transport)?;
	download::apply_with_jobs(&plan, instance, &transport, jobs)?;
	Ok(plan)
}
