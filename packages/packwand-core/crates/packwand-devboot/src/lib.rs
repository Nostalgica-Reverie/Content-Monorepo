//! Dev-testing boot orchestration for the Packwand GUI.
//!
//! Ties together `packwand-minecraft` (metadata + install), `packwand-runtime`
//! (Java discovery), `packwand-auth` (offline session), `packwand-instance`
//! (the shared managed install, keyed by Minecraft version + loader), and
//! `packwand-launch` (the launch plan + process supervisor) into one call:
//! given a packwiz pack subdir, boot it.
//!
//! This crate does not implement packwiz's format beyond reading the handful
//! of `[versions]` fields needed to pick a Minecraft version/loader — mods,
//! indexes, and hashing remain exclusively Go/packwiz-owned. Pointing the
//! launch plan's game directory straight at the pack subdir is what makes
//! that unnecessary: Minecraft/the loader read `mods/`, `config/`, and
//! `resourcepacks/` relative to the game directory on their own.

pub mod bootstrap;
pub mod pack_target;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use packwand_auth::{SecretString, Session};
use packwand_instance::{FsInstanceRepository, InstancePaths, InstanceRecord, InstanceRepository};
use packwand_launch::{build_launch_plan, LaunchPlan};
use packwand_minecraft::MetadataEndpoints;
use serde::{Deserialize, Serialize};

pub use pack_target::{resolve_pack_target, PackTarget, PackTargetError};
pub use packwand_minecraft::InstallProgress;

/// Everything needed to hand a plan to `packwand_launch::launch`: the plan
/// itself and the resolved secret values for its `${secret:<name>}`
/// placeholders (never part of the plan's own serialization).
pub struct BootedPack {
    pub record: InstanceRecord,
    pub plan: LaunchPlan,
    pub secrets: BTreeMap<String, SecretString>,
}

#[derive(Debug, thiserror::Error)]
pub enum DevBootError {
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

/// The offline session used for dev-testing boots when no account is signed
/// in. Not a real account — see `packwandrs.md` for the real-auth plan
/// (`packwand-msa`); this remains available unconditionally as the
/// dev-testing fallback.
pub fn default_offline_session() -> Result<Session, String> {
    let username = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "packwand-dev".to_string());
    packwand_auth::offline_session(&username).map_err(|e| e.to_string())
}

/// Tracks which session's identity is currently baked into a shared
/// instance's `game_args`, so a later boot under a *different* session (e.g.
/// switching from offline dev-testing to a real signed-in account, or
/// between two Microsoft accounts) knows to re-bake rather than reuse. Not
/// part of `packwand-instance`'s own schema — purely this crate's
/// bookkeeping, stored alongside `instance.json`.
#[derive(Serialize, Deserialize)]
struct BakedIdentity {
    uuid: String,
}

fn identity_marker_path(paths: &InstancePaths) -> PathBuf {
    paths.game_dir.join("devboot-identity.json")
}

fn read_baked_identity(paths: &InstancePaths) -> Option<String> {
    let data = std::fs::read(identity_marker_path(paths)).ok()?;
    serde_json::from_slice::<BakedIdentity>(&data)
        .ok()
        .map(|marker| marker.uuid)
}

fn write_baked_identity(paths: &InstancePaths, uuid: &str) -> Result<(), String> {
    let bytes = serde_json::to_vec(&BakedIdentity {
        uuid: uuid.to_string(),
    })
    .map_err(|e| e.to_string())?;
    std::fs::write(identity_marker_path(paths), bytes).map_err(|e| e.to_string())
}

/// Returns the shared instance for `target` under `managed_root`, baked for
/// `session`'s identity. Bootstraps (fetches metadata, installs, persists
/// the record) when no install exists yet, or when the existing install was
/// baked for a *different* identity — `Installer::execute` already skips
/// re-verified files, so re-baking a different identity is cheap (small
/// metadata re-fetch, no re-download of already-verified assets/libraries).
/// Does nothing at all (no network) when the install already matches.
pub fn ensure_instance_for_session(
    managed_root: &Path,
    target: &PackTarget,
    session: &Session,
    on_progress: impl Fn(InstallProgress) + Sync,
) -> Result<InstanceRecord, DevBootError> {
    let repo = FsInstanceRepository::new(managed_root.to_path_buf());
    let id = instance_id_for(target);
    let paths = repo.instance_paths(&id);

    if let Ok(record) = repo.get(&id) {
        if read_baked_identity(&paths).as_deref() == Some(session.uuid.as_str()) {
            return Ok(record);
        }
    }

    let request = bootstrap::BootstrapRequest {
        root: managed_root.to_path_buf(),
        id,
        minecraft: target.minecraft.clone(),
        loader: target.loader.clone(),
        loader_version: target.loader_version.clone(),
        session: session.clone(),
        java: None,
        memory_max_mb: None,
        workers: 8,
        endpoints: MetadataEndpoints::default(),
    };
    let record = bootstrap::bootstrap_with_progress(&request, on_progress)
        .map_err(DevBootError::Instance)?;
    write_baked_identity(&paths, &session.uuid).map_err(DevBootError::Instance)?;
    Ok(record)
}

/// Builds the launch plan for booting `pack_dir` for dev testing: the game
/// directory is `pack_dir` itself (so the pack's own `mods/`, `config/`, and
/// `resourcepacks/` are what gets loaded, live, with no copying), logs go to
/// a new `.packwand-launcher/logs/` next to the pack for easy crash-log
/// discovery, and natives/assets/libraries stay in the shared managed root
/// (pure binary cache, correctly shared across every pack on this
/// Minecraft version + loader).
pub fn boot_pack(
    managed_root: &Path,
    pack_dir: &Path,
    session: &Session,
    on_progress: impl Fn(InstallProgress) + Sync,
) -> Result<BootedPack, DevBootError> {
    let pack_toml = pack_dir.join("pack.toml");
    let target = resolve_pack_target(&pack_toml)?;
    let record = ensure_instance_for_session(managed_root, &target, session, on_progress)?;

    let managed_paths = FsInstanceRepository::new(managed_root.to_path_buf())
        .instance_paths(&instance_id_for(&target));
    let launcher_dir = pack_dir.join(".packwand-launcher");
    let paths = InstancePaths {
        game_dir: pack_dir.to_path_buf(),
        logs_dir: launcher_dir.join("logs"),
        natives_dir: managed_paths.natives_dir,
        assets_dir: managed_paths.assets_dir,
        libraries_dir: managed_paths.libraries_dir,
    };
    let plan = build_launch_plan(&record, &paths);

    // `session` is the same identity already baked into the instance's
    // game_args (ensure_instance_for_session guarantees that); its secrets
    // are exactly the `${secret:<name>}` values `launch` needs to resolve.
    let secrets = if record.session_placeholders.is_empty() {
        BTreeMap::new()
    } else {
        session.secrets()
    };

    Ok(BootedPack {
        record,
        plan,
        secrets,
    })
}

/// The default shared managed root for the launcher's install cache, given
/// the Tauri app's data directory. Kept as a plain function (not tied to
/// any Tauri type) so it stays testable and reusable from a future CLI.
pub fn default_managed_root(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("launcher")
}
