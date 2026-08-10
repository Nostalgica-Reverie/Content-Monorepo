use serde::{Deserialize, Serialize};

/// A failure with a machine-readable `kind` alongside its message.
///
/// The kind is what a UI branches on — `not_found` renders differently from
/// `unsafe_path` — so it is part of the contract rather than a log detail.
/// Deliberately shaped like the desktop app's own error type: the Tauri layer
/// converts field-for-field and adds nothing.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("{message}")]
#[serde(rename_all = "camelCase")]
pub struct OrchestratorError {
	/// A stable, lowercase discriminator.
	pub kind: String,
	/// Human-readable detail.
	pub message: String,
}

impl OrchestratorError {
	/// Builds an error of the given kind.
	pub fn new(kind: impl Into<String>, message: impl std::fmt::Display) -> Self {
		Self {
			kind: kind.into(),
			message: message.to_string(),
		}
	}
}

impl From<std::io::Error> for OrchestratorError {
	fn from(error: std::io::Error) -> Self {
		Self::new("io", error)
	}
}

impl From<serde_json::Error> for OrchestratorError {
	fn from(error: serde_json::Error) -> Self {
		Self::new("json", error)
	}
}

impl From<toml::de::Error> for OrchestratorError {
	fn from(error: toml::de::Error) -> Self {
		Self::new("toml", error)
	}
}

impl From<packwand_instance::InstanceError> for OrchestratorError {
	fn from(error: packwand_instance::InstanceError) -> Self {
		Self::new("instance", error)
	}
}

/// The result type every orchestrator operation returns.
pub type Result<T> = std::result::Result<T, OrchestratorError>;
