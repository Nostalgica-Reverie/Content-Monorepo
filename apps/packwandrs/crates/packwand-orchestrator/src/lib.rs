//! Instance lifecycle for Packwand: create, install, launch, and manage the
//! content of a user-owned Minecraft instance.
//!
//! This is the layer between `packwand-instance` (the record) and whatever is
//! driving it. It used to live inside the desktop app's Tauri command module,
//! which meant none of it could be unit-tested without a webview and none of
//! it was reachable from the CLI. Everything here is plain blocking Rust over
//! an [`FsUserInstanceRepository`](packwand_instance::FsUserInstanceRepository).
//!
//! **Keep this crate free of Tauri, clap and axum.** Hosts convert
//! [`OrchestratorError`] into their own error type — its `kind` field exists so
//! that conversion carries the discriminator a UI branches on.
//!
//! Named `orchestrator` rather than `launcher` because `packwand-launch`
//! already exists and owns the process supervisor; one letter between
//! `launcher::launch` and `launch::launch` is not a distinction worth having.

#![forbid(unsafe_code)]

pub mod archive;
pub mod art;
pub mod boot;
pub mod bootstrap;
pub mod content;
pub mod error;
pub mod files;
pub mod install;
pub mod launch;
pub mod lifecycle;
pub mod pack_target;
pub mod paths;
pub mod stages;
pub mod steps;

pub use archive::{ArchiveFormat, ExportResult};
pub use art::ImageKind;
pub use boot::{
	BootError, InstallProgress, PackTarget, PackTargetError, default_managed_root,
	default_offline_session, ensure_instance, instance_id_for, resolve_pack_target,
};
pub use content::InstanceContent;
pub use error::{OrchestratorError, Result};
pub use files::InstanceFileEntry;
pub use install::PendingManualDownload;
pub use launch::{LaunchRequest, LaunchSignal};
pub use lifecycle::{CreateSource, CreateSpec, InstancePatch, Patch, SettingsPatch};
pub use paths::{backing_pack, normalized, now_ms, safe_content_path, safe_instance_file};
pub use stages::standard_steps;
pub use steps::{LaunchStep, Outcome, StepContext, run_steps};
