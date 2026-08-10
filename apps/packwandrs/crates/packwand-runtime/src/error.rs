//! The error type shared by discovery, probing, and provisioning.

use std::path::PathBuf;

/// Anything that can go wrong locating, inspecting, or installing a JVM.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
	#[error("{home} is not a Java installation: {reason}")]
	NotAJavaHome { home: PathBuf, reason: String },
	#[error("could not parse Java version {0:?}")]
	UnparseableVersion(String),
	#[error("no discovered Java installation satisfies major version {required}; found: [{found}]")]
	NoCompatibleJava { required: u32, found: String },
	#[error("could not run {executable} to identify it: {reason}")]
	ProbeFailed { executable: PathBuf, reason: String },
	/// A JVM that starts but never finishes printing its own properties.
	///
	/// Worth its own variant: a hung probe is the failure that makes a
	/// launcher appear to freeze on startup, and it needs a different
	/// remedy from a JVM that is merely missing.
	#[error("{executable} did not respond within {seconds}s")]
	ProbeTimedOut { executable: PathBuf, seconds: u64 },
	#[error("no downloadable Java runtime for major version {required} on {runtime_os}")]
	NoRuntimeAvailable { required: u32, runtime_os: String },
	#[error("this platform has no Mojang java-runtime key")]
	UnsupportedPlatform,
	#[error("Java runtime metadata from {url} could not be read: {reason}")]
	Metadata { url: String, reason: String },
	#[error("failed to install a Java runtime into {path}: {reason}")]
	Install { path: PathBuf, reason: String },
}
