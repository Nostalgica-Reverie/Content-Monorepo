use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;
use walkdir::WalkDir;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let data_dir = Path::new("datapacks");
    if !data_dir.exists() {
        println!("::warning:: no datapacks directory found. skipping.");
        return;
    }

    let entries = fs::read_dir(data_dir).expect("Failed to read datapacks directory");
    let mut handles = vec![];
    let errors = Arc::new(Mutex::new(Vec::new()));

    println!("validating datapacks");

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
            let err_clone = Arc::clone(&errors);
            let handle = thread::spawn(move || {
                validate_datapack(path, err_clone);
            });
            handles.push(handle);
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_errors = errors.lock().unwrap();
    if !final_errors.is_empty() {
        println!("\n datapack validation failed:");
        for err in final_errors.iter() {
            eprintln!("{}", err);
        }
        std::process::exit(1);
    }

    println!("\n all datapacks validated!");
}

fn validate_datapack(path: PathBuf, errors: Arc<Mutex<Vec<String>>>) {
    let pack_name = path.file_name().unwrap().to_str().unwrap();
    println!("::group::Checking Datapack: {}", pack_name);

    let mcmeta = path.join("pack.mcmeta");
    if !mcmeta.exists() {
        errors.lock().unwrap().push(format!("{}: missing pack.mcmeta", pack_name));
    }

    for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        let f_path = entry.path();
        
        if f_path.extension().map_or(false, |ext| ext == "json") {
            let content = fs::read_to_string(f_path).unwrap_or_default();
            if let Err(e) = serde_json::from_str::<Value>(&content) {
                errors.lock().unwrap().push(format!(
                    "{}: Invalid JSON in {:?} -> {}", 
                    pack_name, entry.file_name(), e
                ));
            }
        }

        if f_path.is_file() {
            let relative = f_path.strip_prefix(&path).unwrap();
            let components: Vec<_> = relative.components().collect();
            
            if components.len() > 1 && components[0].as_os_str() == "data" {
                if components.len() < 3 {
                    errors.lock().unwrap().push(format!(
                        "{}: File {:?} is outside of a namespace folder!", 
                        pack_name, relative
                    ));
                }
            }
        }
    }
    println!("::endgroup::");
}