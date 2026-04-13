use std::fs;
use std::path::Path;
use somnus_core::get_changed_files;

fn main() {
    if run() {
        std::process::exit(1);
    }
}

pub fn run() -> bool {
    let mut failed = false;

    let changed_files = match get_changed_files() {
        Ok(files) => files,
        Err(e) => {
            eprintln!("::error::Failed to retrieve changed files: {}", e);
            return true;
        }
    };

    for file_path in changed_files {
        let path = Path::new(&file_path);
        if !path.exists() { continue; }

        if file_path.ends_with(".json") || file_path.ends_with(".mcmeta") {
            if ["modpacks/", "resourcepacks/", "datapacks/"]
                .iter()
                .any(|dir| file_path.starts_with(dir)) 
            {
                println!("::group::Linting {}", file_path);
                let content = fs::read_to_string(path).expect("Read Error");
                
                if let Err(e) = serde_json::from_str::<serde_json::Value>(&content) {
                    eprintln!("::error file={}::INVALID JSON: {}", file_path, e);
                    failed = true;
                }
                println!("::endgroup::");
            }
        }
    }

    if failed { 
        eprintln!("Fix yo json chud..."); 
    }
    failed
}