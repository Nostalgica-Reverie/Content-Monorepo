use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("{message}")]
#[serde(rename_all = "camelCase")]
pub struct SerializableError {
	pub kind: String,
	pub message: String,
}

impl SerializableError {
	pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
		Self {
			kind: kind.into(),
			message: message.into(),
		}
	}
}

impl From<std::io::Error> for SerializableError {
	fn from(error: std::io::Error) -> Self {
		Self::new("io", error.to_string())
	}
}

impl From<serde_json::Error> for SerializableError {
	fn from(error: serde_json::Error) -> Self {
		Self::new("json", error.to_string())
	}
}

impl From<toml::de::Error> for SerializableError {
	fn from(error: toml::de::Error) -> Self {
		Self::new("toml", error.to_string())
	}
}

impl From<packwand_ops::OpsError> for SerializableError {
	fn from(error: packwand_ops::OpsError) -> Self {
		Self::new("pack_operation", error.to_string())
	}
}

impl From<packwand_build::BuildError> for SerializableError {
	fn from(error: packwand_build::BuildError) -> Self {
		Self::new("build", error.to_string())
	}
}

impl From<packwand_workspace::Error> for SerializableError {
	fn from(error: packwand_workspace::Error) -> Self {
		Self::new("workspace_operation", error.to_string())
	}
}

impl From<packwand_providers::ProviderError> for SerializableError {
	fn from(error: packwand_providers::ProviderError) -> Self {
		Self::new("provider", error.to_string())
	}
}

impl From<packwand_instance::InstanceError> for SerializableError {
	fn from(error: packwand_instance::InstanceError) -> Self {
		Self::new("instance", error.to_string())
	}
}

impl From<packwand_orchestrator::OrchestratorError> for SerializableError {
	fn from(error: packwand_orchestrator::OrchestratorError) -> Self {
		// Field-for-field: the orchestrator already carries the discriminator
		// the UI branches on, so re-labelling it here would lose information.
		Self {
			kind: error.kind,
			message: error.message,
		}
	}
}

pub type CommandResult<T> = Result<T, SerializableError>;
