use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::fs;
use std::path::Path;

fn main() {
    let packs_dir = Path::new("resourcepacks");
    let jar_path = dirs::home_dir()
        .expect("Could not find home directory")
        .join("rpv.jar");
    
    let config_path = "resourcepacks/rpv-config.json";

    if !packs_dir.exists() {
        println!("::warning:: no directory found at {:?}. skipping.", packs_dir);
        return;
    }

    if !jar_path.exists() {
        eprintln!("rpv not found {:?}", jar_path);
        std::process::exit(1);
    }

    let entries = fs::read_dir(packs_dir).expect("failed to read resourcepacks directory");
    let mut handles = vec![];
    let errors = Arc::new(Mutex::new(Vec::new()));

    println!("starting validation");

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        let path = entry.path();

        if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
            let err_clone = Arc::clone(&errors);
            let jar_clone = jar_path.clone();
            let config_clone = config_path.to_string();

            let handle = thread::spawn(move || {
                let pack_name = path.file_name().unwrap().to_str().expect("Invalid filename");
                
                println!("::group::Validating {}", pack_name);

                let output = Command::new("java")
                    .args([
                        "-jar", jar_clone.to_str().unwrap(),
                        "-rp", path.to_str().unwrap(),
                        "-config", &config_clone,
                    ])
                    .output();

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        
                        if !stdout.is_empty() { print!("{}", stdout); }
                        if !stderr.is_empty() { eprintln!("{}", stderr); }

                        if !out.status.success() {
                            let mut e = err_clone.lock().unwrap();
                            e.push(format!("failed validation: {}", pack_name));
                        }
                    }
                    Err(err) => {
                        let mut e = err_clone.lock().unwrap();
                        e.push(format!("failed to execute java {}: {}", pack_name, err));
                    }
                }
                println!("::endgroup::");
            });
            handles.push(handle);
        }
    }

    for handle in handles {
        if let Err(e) = handle.join() {
            eprintln!("thread panic: {:?}", e);
        }
    }

    let final_errors = errors.lock().unwrap();
    if !final_errors.is_empty() {
        println!("\nfailures detected");
        for err in final_errors.iter() {
            eprintln!("{}", err);
        }
        std::process::exit(1);
    }

    println!("\nall resourcepacks are good to go");
}