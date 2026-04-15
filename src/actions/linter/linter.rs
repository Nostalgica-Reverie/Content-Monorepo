// this was written in like 30 minutes
use std::fs;
use std::process::{Command, exit};
use std::path::Path;

fn main() {
    let output = Command::new("git")
        .args(&["diff", "--name-only", "HEAD~1", "HEAD"])
        .output()
        .expect("Failed to execute git diff");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut failed = false;

    for file_path in stdout.lines() {
        let path = Path::new(file_path);
        
        if !path.exists() { continue; }

        if file_path.ends_with(".json") || file_path.ends_with(".mcmeta") {
            if file_path.starts_with("modpacks/") || 
               file_path.starts_with("resourcepacks/") || 
               file_path.starts_with("datapacks/") {
                
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
        exit(1);
    } else {
        println!("Lint success");
    }
}
// if this doesnt work i will cry bro i havent used rust since i was 11