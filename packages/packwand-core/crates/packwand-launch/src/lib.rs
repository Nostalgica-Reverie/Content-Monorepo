//! Launch plans, lifecycle events, and process supervision.
//!
//! Part of the `packwand-rs` shared core (see `packwandrs.md`). This crate
//! must stay free of Tauri, clap, and axum dependencies. A launch is a
//! two-step contract: build an inspectable [`LaunchPlan`], then hand the
//! approved plan to [`launch`], which supervises the child process and emits
//! typed [`LaunchEvent`]s.

#![forbid(unsafe_code)]

mod plan;
mod supervisor;

pub use plan::{
    build_launch_plan, host_classpath_separator, LaunchPaths, LaunchPlan, PLAN_SCHEMA_VERSION,
};
pub use supervisor::{
    launch, CancellationToken, LaunchError, LaunchEvent, LaunchHandle, LaunchOptions,
};
