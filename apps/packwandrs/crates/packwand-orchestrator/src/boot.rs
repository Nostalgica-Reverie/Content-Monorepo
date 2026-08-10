//! Turning a pack directory into a launchable instance.
//!
//! Ties together `packwand-minecraft` (metadata + install), `packwand-runtime`
//! (Java discovery), `packwand-auth` (the session), `packwand-instance` (the
//! shared managed install, keyed by Minecraft version + loader), and
//! `packwand-launch` (the launch plan + process supervisor).
//!
//! This does not implement packwiz's format beyond reading the handful of
//! `[versions]` fields needed to pick a Minecraft version and loader.
//! Pointing the pack source and the game directory are deliberately separate:
//! content is resolved from the pack while Minecraft writes only to the
//! instance.
//!
//! This was its own crate, `packwand-devboot`, for as long as booting a pack
//! was a development aid. It is the production launch path now, and a name
//! promising otherwise was the least accurate thing about it.

use std::path::{Path, PathBuf};

use packwand_auth::Session;
use packwand_instance::{FsInstanceRepository, InstanceRecord, InstanceRepository};

use packwand_minecraft::MetadataEndpoints;

use crate::bootstrap;

pub use crate::pack_target::{PackTarget, PackTargetError, resolve_pack_target};
pub use packwand_minecraft::InstallProgress;

#[derive(Debug, thiserror::Error)]
pub enum BootError {
	#[error(transparent)]
	PackTarget(#[from] PackTargetError),
	#[error("failed to load or create the shared instance: {0}")]
	Instance(String),
}

/// A stable, filesystem-safe instance id for one (Minecraft version, loader,
/// loader version) combination, shared across every pack that targets it.
pub fn instance_id_for(target: &PackTarget) -> String {
	let sanitize = |s: &str| -> String {
		s.chars()
			.map(|c| {
				if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
					c
				} else {
					'_'
				}
			})
			.collect()
	};
	match (&target.loader, &target.loader_version) {
		(Some(loader), Some(version)) => format!(
			"{}-{}-{}",
			sanitize(&target.minecraft),
			sanitize(loader),
			sanitize(version)
		),
		(Some(loader), None) => format!("{}-{}", sanitize(&target.minecraft), sanitize(loader)),
		(None, _) => format!("{}-vanilla", sanitize(&target.minecraft)),
	}
}

/// The offline session used when no account is signed in.
///
/// Not a real account: the game runs, but multiplayer and anything else that
/// checks the session service will refuse it.
pub fn default_offline_session() -> Result<Session, String> {
	let username = std::env::var("USERNAME")
		.or_else(|_| std::env::var("USER"))
		.unwrap_or_else(|_| "packwand-dev".to_string());
	packwand_auth::offline_session(&username).map_err(|e| e.to_string())
}

/// The session a launch should run under, and the claim that holds it.
pub struct LaunchSession {
	pub session: Session,
	/// Held for the life of the launch; releasing it frees the account for
	/// another instance. `None` for an offline session, which nothing else
	/// can conflict with.
	pub claim: Option<packwand_msa::AccountClaim>,
	/// What to tell the user about how this session was obtained.
	pub note: Option<String>,
}

/// Resolves the account a launch runs as, falling back to offline only where
/// that is the right answer.
///
/// The fallback is deliberately not unconditional. A service outage should
/// not stop anyone playing, so it drops to offline and says so. An account
/// that needs attention — no Xbox profile, an unverified minor, a rejected
/// sign-in — stops instead: starting the game under a different identity
/// without saying so is how a user loses an afternoon's singleplayer progress
/// to the wrong save directory.
///
/// `msa_client_id` absent means this build has no Azure registration
/// configured, which is not an error; it simply means offline.
pub fn session_for_launch(
	msa_client_id: Option<&str>,
	accounts_root: &Path,
) -> Result<LaunchSession, String> {
	let offline = |note: Option<String>| {
		default_offline_session().map(|session| LaunchSession {
			session,
			claim: None,
			note,
		})
	};
	let Some(client_id) = msa_client_id.filter(|id| !id.trim().is_empty()) else {
		return offline(None);
	};
	let accounts = packwand_msa::Accounts::new(accounts_root);
	let config = packwand_msa::MsaConfig {
		client_id: client_id.to_string(),
	};
	match packwand_msa::session_for_launch(&config, &accounts) {
		packwand_msa::SessionOutcome::Authenticated(session) => {
			let account = packwand_msa::Account {
				uuid: session.uuid.clone(),
				name: session.username.clone(),
				disabled: false,
				last_used_ms: Some(crate::paths::now_ms()),
			};
			// Claim before the game starts: Minecraft allows one session per
			// account, and a second launch would disconnect the first.
			let claim = accounts.claim(&account).map_err(|e| e.to_string())?;
			let _ = accounts.remember(account, None);
			Ok(LaunchSession {
				session,
				claim: Some(claim),
				note: None,
			})
		}
		packwand_msa::SessionOutcome::NoAccount => offline(None),
		packwand_msa::SessionOutcome::Unavailable { state, message } => {
			if state.allows_offline_fallback() {
				offline(Some(format!("Playing offline: {message}")))
			} else {
				Err(message)
			}
		}
	}
}

/// Returns the shared install for `target` under `managed_root`, bootstrapping
/// it (metadata, download, record) only when one is not already there.
///
/// No account is involved. The record's arguments carry `${identity:*}`
/// placeholders rather than one player's name, which is what lets a single
/// install serve every account on that Minecraft version — before that, two
/// accounts took turns rewriting the same record.
///
/// A record written by an older build has identity baked in, so it is
/// rebuilt rather than reused; `Installer::execute` skips already-verified
/// files, making that a metadata refetch rather than a redownload.
pub fn ensure_instance(
	managed_root: &Path,
	target: &PackTarget,
	java: Option<PathBuf>,
	on_progress: impl Fn(InstallProgress) + Sync,
) -> Result<InstanceRecord, BootError> {
	let repo = FsInstanceRepository::new(managed_root.to_path_buf());
	let id = instance_id_for(target);

	if let Ok(record) = repo.get(&id)
		&& record.schema_version == packwand_instance::SCHEMA_VERSION
	{
		return Ok(record);
	}

	let request = bootstrap::BootstrapRequest {
		root: managed_root.to_path_buf(),
		id,
		minecraft: target.minecraft.clone(),
		loader: target.loader.clone(),
		loader_version: target.loader_version.clone(),
		java,
		memory_max_mb: None,
		// 0 defers to the shared default, which the app setting and --jobs
		// both feed.
		workers: 0,
		endpoints: MetadataEndpoints::default(),
	};
	bootstrap::bootstrap_with_progress(&request, on_progress).map_err(BootError::Instance)
}

/// The default shared managed root for the launcher's install cache, given
/// the Tauri app's data directory. Kept as a plain function (not tied to
/// any Tauri type) so it stays testable and reusable from a future CLI.
pub fn default_managed_root(app_data_dir: &Path) -> PathBuf {
	app_data_dir.join("launcher")
}
