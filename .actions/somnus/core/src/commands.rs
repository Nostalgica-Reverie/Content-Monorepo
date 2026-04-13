use crate::manager::{SomnusManager, Project};
use crate::errors::SomnusError;
use tracing::{info, error, instrument};

pub struct SomnusCommands {
    manager: SomnusManager,
}

impl SomnusCommands {
    pub fn new() -> Self {
        Self {
            manager: SomnusManager::new(),
        }
    }

    #[instrument(skip(self))]
    pub fn bump_project(&self, category: &str, name: &str, version: &str) -> Result<(), SomnusError> {
        info!("Attempting to bump {}/{} to {}", category, name, version);
        
        let project = self.manager.get_project(category, name)
            .ok_or_else(|| SomnusError::MissingManifest(format!("{}/{}", category, name)))?;

        project.bump(version).map_err(|e| {
            error!("Failed to bump project: {}", e);
            SomnusError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;

        info!("Successfully bumped {}", name);
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn bump_all_in_category(&self, category: &str, version: &str) -> Result<(), SomnusError> {
        let projects = self.manager.discover_projects(category);
        info!("Found {} projects in {}", projects.len(), category);

        for project in projects {
            project.bump(version).map_err(|e| {
                SomnusError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
        }

        Ok(())
    }
}