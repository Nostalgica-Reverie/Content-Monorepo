//! Java runtimes: finding them, identifying them, and installing one when
//! the machine has none that will do.
//!
//! Part of the shared Packwand core. This crate must stay free of Tauri,
//! clap, and axum dependencies.
//!
//! The four modules answer four separate questions, in increasing order of
//! cost — which is also the order a caller should try them in:
//!
//! * [`version`] — how do two Java version strings compare?
//! * [`discovery`] — what is already installed? (filesystem only)
//! * [`probe`] — what exactly is this executable? (starts a JVM, cached)
//! * [`provision`] — nothing here will do; fetch one.

#![forbid(unsafe_code)]

pub mod discovery;
pub mod error;
pub mod probe;
pub mod provision;
pub mod version;

pub use discovery::{
	DiscoveryConfig, DiscoverySource, JavaInstallation, discover, inspect_java_home,
	java_executable_name, select_compatible,
};
pub use error::RuntimeError;
pub use probe::{ProbeCache, ProbedJava};
pub use provision::{Catalog, RuntimeFile, RuntimeSelection, install_runtime, runtime_os};
pub use version::{JavaVersion, parse_major_version};
