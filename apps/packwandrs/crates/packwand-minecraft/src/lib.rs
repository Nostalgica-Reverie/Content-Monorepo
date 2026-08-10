//! Minecraft version metadata, install planning, and transactional
//! installation.
//!
//! Part of the shared Packwand core. This crate
//! must stay free of Tauri, clap, and axum dependencies. It follows the
//! plan/apply split required by the migration spec: metadata is turned
//! into an inspectable [`plan::InstallPlan`] first, and only
//! [`install::Installer`] touches the network and the filesystem.

#![forbid(unsafe_code)]

pub mod args;
pub mod http;
pub mod install;
pub mod merge;
pub mod meta;
pub mod model;
pub mod plan;
pub mod rules;

pub use http::{FixtureHttpClient, HttpClient, HttpError, UreqClient};
pub use install::{InstallProgress, InstallReport, Installer};
pub use meta::{Fetched, InstallerProfile, MetadataClient, MetadataEndpoints};
pub use rules::Host;

#[derive(Debug, thiserror::Error)]
pub enum MinecraftError {
	#[error(transparent)]
	Http(#[from] HttpError),
	#[error("invalid JSON from {context}: {message}")]
	Json { context: String, message: String },
	#[error("invalid XML from {context}: {message}")]
	Xml { context: String, message: String },
	#[error("checksum mismatch for {url}: expected sha1 {expected}, got {actual}")]
	ChecksumMismatch {
		url: String,
		expected: String,
		actual: String,
	},
	#[error("size mismatch for {url}: expected {expected} bytes, got {actual}")]
	SizeMismatch {
		url: String,
		expected: u64,
		actual: u64,
	},
	#[error("refusing unsafe metadata-supplied path {0:?}")]
	UnsafePath(String),
	#[error("library name {0:?} is not a valid maven coordinate")]
	BadLibraryName(String),
	#[error("asset object {object:?} has invalid hash {hash:?}")]
	BadAssetHash { object: String, hash: String },
	#[error("version {0} has no client download")]
	MissingClientDownload(String),
	#[error("version {0} declares no main class")]
	MissingMainClass(String),
	#[error("version {0} declares neither modern nor legacy arguments")]
	MissingArguments(String),
	#[error("this asset index maps to per-instance resources; a resources directory is required")]
	ResourcesDirRequired,
	#[error("version {0:?} was not found in the version manifest")]
	VersionNotFound(String),
	#[error("no Fabric loader version is available for game version {0:?}")]
	NoLoaderVersion(String),
	#[error("loader version {loader:?} is not available for game version {game_version:?}")]
	LoaderVersionNotFound {
		game_version: String,
		loader: String,
	},
	#[error("inheritsFrom chain starting at {0:?} is too deep (metadata cycle?)")]
	InheritanceTooDeep(String),
	#[error("installer archive at {url} is invalid: {message}")]
	InstallerArchive { url: String, message: String },
	#[error("installer archive at {url} is missing {entry}")]
	InstallerEntryMissing { url: String, entry: String },
	#[error("I/O error at {path}: {message}")]
	Io {
		path: std::path::PathBuf,
		message: String,
	},
	#[error("archive error in {path}: {message}")]
	Archive {
		path: std::path::PathBuf,
		message: String,
	},
}
