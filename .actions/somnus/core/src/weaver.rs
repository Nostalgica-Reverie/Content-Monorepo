use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn bump_version(project_dir: &str, new_version: &str) -> Result<(), String> {
    let manifest_path = Path::new(project_dir).join("manifest.json");

    if !manifest_path.exists() {
        return Err(format!("no manifest.json found in {}", project_dir));
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Read Error: {}", e))?;

    let mut json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("JSON Parse Error: {}", e))?;

    if let Some(v) = json.get_mut("version") {
        *v = Value::String(new_version.to_string());
    } else {
        return Err(format!("'version' key missing in {}", manifest_path.display()));
    }

    let new_content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Serialization Error: {}", e))?;

    fs::write(&manifest_path, new_content)
        .map_err(|e| format!("Write Error: {}", e))?;
    
    Ok(())
}