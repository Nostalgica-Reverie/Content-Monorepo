use std::path::{Path, PathBuf};
use crate::weaver;

pub enum ProjectCategory {
    Modpack,
    ResourcePack,
    DataPack,
}

pub struct Project {
    pub name: String,
    pub category: ProjectCategory,
    pub path: PathBuf,
}

impl Project {
    pub fn bump(&self, version: &str) -> Result<(), String> {
        weaver::bump_version(self.path.to_str().unwrap(), version)
    }
}

pub struct SomnusManager {
    pub root_dir: PathBuf,
}

impl SomnusManager {
    pub fn new() -> Self {
        Self {
            root_dir: PathBuf::from("."),
        }
    }

    pub fn get_project(&self, category: &str, name: &str) -> Option<Project> {
        let path = self.root_dir.join(category).join(name);
        
        if path.exists() {
            let cat_enum = match category {
                "resourcepacks" => ProjectCategory::ResourcePack,
                "datapacks" => ProjectCategory::DataPack,
                _ => ProjectCategory::Modpack,
            };

            Some(Project {
                name: name.to_string(),
                category: cat_enum,
                path,
            })
        } else {
            None
        }
    }

    pub fn discover_projects(&self, category: &str) -> Vec<Project> {
        let cat_path = self.root_dir.join(category);
        let mut projects = Vec::new();

        if let Ok(entries) = std::fs::read_dir(cat_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.path().is_dir() {
                    let name = entry.file_name().into_string().unwrap_or_default();
                    if let Some(p) = self.get_project(category, &name) {
                        projects.push(p);
                    }
                }
            }
        }
        projects
    }
}