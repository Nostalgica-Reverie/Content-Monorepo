use std::path::Path;
use walkdir::WalkDir;

pub fn check_naming_conventions(project_path: &str) -> Vec<String> {
    let mut issues = Vec::new();

    for entry in WalkDir::new(project_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            
            if file_name.contains(' ') {
                issues.push(format!("Space in filename: {:?}", path));
            }

            if file_name.chars().any(|c| c.is_uppercase()) {
                issues.push(format!("Uppercase characters in: {:?}", path));
            }
        }
    }
    issues
}