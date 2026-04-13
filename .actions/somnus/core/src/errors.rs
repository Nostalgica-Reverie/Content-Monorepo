use thiserror::Error;

#[derive(Error, Debug)]
pub enum SomnusError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git Error: {0}")]
    Git(#[from] git2::Error),

    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Missing Manifest: {0}")]
    MissingManifest(String),

    #[error("Generic Error: {0}")]
    Generic(String),
}