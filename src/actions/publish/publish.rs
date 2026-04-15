use serde_json::Value;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    let manifest_path_str = args.get(1).expect("Usage: publish <path_to_manifest.json>");

    let manifest_path = Path::new(manifest_path_str);
    let p_dir = manifest_path.parent().expect("Could not find parent directory");

    let manifest_content = fs::read_to_string(manifest_path).expect("Failed to read manifest");
    let manifest: Value = serde_json::from_str(&manifest_content).expect("Failed to parse manifest JSON");

    let raw_name = manifest["name"].as_str().unwrap();
    let p_name = raw_name.replace(" ", "-");
    let p_ver = manifest["version"].as_str().unwrap();
    let mc_ver = manifest["mc_version"].as_str().unwrap();
    let p_type = manifest["type"].as_str().unwrap();
    let mr_id = manifest["modrinth_id"].as_str().unwrap_or("");
    let cf_id = manifest["curseforge_id"].as_str().unwrap_or("");
    
    let filename_base = format!("{}-{}-fabric-{}", p_name, mc_ver, p_ver);

    let workspace = env::var("GITHUB_WORKSPACE").unwrap_or_else(|_| ".".to_string());
    let artifacts_dir = Path::new(&workspace).join(p_dir).join("artifacts");

    if artifacts_dir.exists() {
        fs::remove_dir_all(&artifacts_dir).unwrap();
    }
    fs::create_dir_all(&artifacts_dir).unwrap();

    println!("::group::Building Artifacts for {}", raw_name);
    
    if p_type == "modpack" {
        for platform in &["mr", "cf"] {
            let target_folder = format!("{}-{}", mc_ver, platform);
            let target_path = p_dir.join(&target_folder);

            if target_path.exists() {
                run_cmd("packwiz", &["refresh"], &target_path);

                let (export_cmd, ext) = if *platform == "mr" { ("modrinth", "mrpack") } else { ("curseforge", "zip") };
                let out_file = artifacts_dir.join(format!("{}-{}.{}", filename_base, platform, ext));

                run_cmd("packwiz", &[export_cmd, "export", "--output", out_file.to_str().unwrap()], &target_path);
            } else {
                println!("Skipping {}: folder {} not found", platform, target_path.display());
            }
        }
    } else if p_type == "resourcepack" || p_type == "datapack" {
        let out_file = artifacts_dir.join(format!("{}-{}.zip", manifest["id"].as_str().unwrap_or("project"), p_ver));
        let content_dir = p_dir.join("content");

        if content_dir.exists() {
            run_cmd("zip", &["-r", out_file.to_str().unwrap(), "."], &content_dir);
        } else {
            println!("Error: content directory not found at {}", content_dir.display());
            std::process::exit(1);
        }
    }
    println!("::endgroup::");

    if let Ok(out_path) = env::var("GITHUB_OUTPUT") {
        let mut out_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(out_path)
            .expect("Could not open GITHUB_OUTPUT");

        writeln!(out_file, "mr_id={}", mr_id).unwrap();
        writeln!(out_file, "cf_id={}", cf_id).unwrap();
        writeln!(out_file, "name={} {}", raw_name, p_ver).unwrap();
        writeln!(out_file, "ver={}", p_ver).unwrap();
        writeln!(out_file, "mc={}", mc_ver).unwrap();
        writeln!(out_file, "type={}", p_type).unwrap();
        writeln!(out_file, "path={}", p_dir.to_str().unwrap()).unwrap();
    }
}

fn run_cmd(cmd: &str, args: &[&str], dir: &Path) {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status();

    match status {
        Ok(s) if s.success() => (),
        Ok(s) => {
            eprintln!("Command '{} {:?}' failed with exit code: {}", cmd, args, s);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to execute '{}': {}", cmd, e);
            std::process::exit(1);
        }
    }
}