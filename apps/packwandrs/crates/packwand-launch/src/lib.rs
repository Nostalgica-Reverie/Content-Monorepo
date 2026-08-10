//! Launch plans, lifecycle events, and process supervision.
//!
//! Part of the shared Packwand core. This crate
//! must stay free of Tauri, clap, and axum dependencies. A launch is a
//! two-step contract: build an inspectable [`LaunchPlan`], then hand the
//! approved plan to [`launch`], which supervises the child process and emits
//! typed [`LaunchEvent`]s.

#![forbid(unsafe_code)]

pub mod censor;
pub mod log;
mod plan;
mod supervisor;

pub use censor::Censor;
pub use log::{LogBuffer, LogLevel, LogLine, LogParser, latest_crash_report, read_latest_log};
pub use plan::{
	LaunchPaths, LaunchPlan, PLAN_SCHEMA_VERSION, build_launch_plan, host_classpath_separator,
};
pub use supervisor::{
	CancellationToken, LaunchError, LaunchEvent, LaunchHandle, LaunchOptions, launch,
};
