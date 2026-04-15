use std::path::PathBuf;
use crate::errors::SomnusError;

pub struct SomnusState {
    pub root: PathBuf,
    pub dev_mode: bool,
}

impl SomnusState {
    pub fn init() -> Result<Self, SomnusError> {
        let root = std::env::current_dir()?;
        let dev_mode = std::cfg!(debug_assertions);
        
        tracing::debug!("state loaded at {}", root.display());
        
        Ok(Self { root, dev_mode })
    }
}