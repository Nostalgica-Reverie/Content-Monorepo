//! Feature-gated adapter over the packwand-rs core crates (the Phase 1
//! experiment from `packwandrs.md`).
//!
//! This proves the shared Rust core is consumable in-process by the desktop
//! shell — no Go sidecar, no loopback HTTP. It deliberately exposes only
//! `core_list_instances` and `core_plan_launch`, does not touch the current
//! boot flow, never accesses the network, and is compiled solely behind the
//! off-by-default `launcher-spike` feature.

use std::path::PathBuf;

use packwand_instance::{FsInstanceRepository, InstanceRepository, ListEntry};
use packwand_launch::{build_launch_plan, LaunchPlan};

/// Lists the instances stored under `root`, including error entries for
/// corrupt or future-schema records.
#[tauri::command]
pub fn core_list_instances(root: String) -> Result<Vec<ListEntry>, String> {
    FsInstanceRepository::new(PathBuf::from(root))
        .list()
        .map_err(|e| e.to_string())
}

/// Builds the deterministic launch plan for one instance under `root`.
#[tauri::command]
pub fn core_plan_launch(root: String, instance: String) -> Result<LaunchPlan, String> {
    let repo = FsInstanceRepository::new(PathBuf::from(root));
    let record = repo.get(&instance).map_err(|e| e.to_string())?;
    Ok(build_launch_plan(&record, &repo.instance_paths(&record.id)))
}
