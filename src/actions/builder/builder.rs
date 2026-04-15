// this was written in 60m
use std::{env, fs::{self, File}, io::{Write, Read}, path::Path, process::{Command, exit}};
use std::collections::HashSet;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

fn main() {
    let args: Vec<String> = env::args().collect();
    let short_sha = args.get(1).map(|s| s.as_str()).unwrap_or("unknown");

    println!("detecting changed files...");
    let output = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
        .output()
        .expect("Failed to get git diff");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let mut changed_targets = HashSet::new();

    for line in stdout.lines() {
        if line.starts_with("external/") || line.starts_with(".actions/") || line.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split('/').collect();
        if parts.len() >= 2 {
            changed_targets.insert((parts[0], parts[1]));
        }
    }

    if changed_targets.is_empty() {
        println!(no packs detected in git diff.");
        return;
    }

    let _ = fs::create_dir_all("artifacts");
    let mut all_success = true;

    for (category, pack_id) in changed_targets {
        match category {
            "modpacks" => {
                if !build_modpack(pack_id, short_sha) { all_success = false; }
            },
            "resourcepacks" => build_resource_pack(pack_id, short_sha),
            "datapacks" => build_datapack(pack_id, short_sha),
            _ => println!("category '{}' does not require a build.", category),
        }
    }

    if !all_success {
        eprintln!("one or more builds failed.");
        exit(1);
    } else {
        println!("all builds completed successfully.");
    }
}

fn build_modpack(pack_id: &str, sha: &str) -> bool {
    println!("building modpack: {}", pack_id);
    let base_path = format!("modpacks/{}", pack_id);
    let mut built_something = false;

    for entry in WalkDir::new(&base_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == "manifest.json" {
            let manifest_path = entry.path();
            let p_dir = manifest_path.parent().unwrap();

            let file = match File::open(manifest_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("hailed to open {:?}: {}", manifest_path, e);
                    continue;
                }
            };

            let json: serde_json::Value = match serde_json::from_reader(file) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("invalid JSON in {:?}: {}", manifest_path, e);
                    continue; 
                }
            };

            let mc_ver = json["mc_version"].as_str().unwrap_or("1.20.1");
            let p_ver = json["version"].as_str().unwrap_or("1.0.0");

            for p in ["mr", "cf"] {
                let target_subdir = format!("{}-{}", mc_ver, p);
                let target_path = p_dir.join(&target_subdir);

                if target_path.exists() {
                    built_something = true;
                    let ext = if p == "mr" { "mrpack" } else { "zip" };
                    let platform = if p == "mr" { "modrinth" } else { "curseforge" };
                    let output_name = format!("{}-{}-{}-{}-{}.{}", pack_id, mc_ver, p, p_ver, sha, ext);

                    let _ = Command::new("packwiz").args(["refresh", "-y"]).current_dir(&target_path).status();
                    
                    let export = Command::new("packwiz")
                        .args([platform, "export", "--output", &format!("../../../artifacts/{}", output_name)])
                        .current_dir(&target_path)
                        .status();

                    match export {
                        Ok(s) if !s.success() => {
                            eprintln!("packwiz export failed for {}", target_subdir);
                            return false;
                        },
                        Err(e) => {
                            eprintln!("failed to launch packwiz: {}", e);
                            return false;
                        },
                        _ => println!("exported {}", output_name),
                    }
                }
            }
        }
    }
    
    if !built_something {
        println!("found no valid targets (mr/cf) or manifest for {}", pack_id);
    }
    
    true
}

fn build_resource_pack(pack_id: &str, sha: &str) {
    println!("building resource pack: {}", pack_id);
    let src = format!("resourcepacks/{}", pack_id);
    let dest = format!("artifacts/{}-{}.zip", pack_id, sha);

    let status = Command::new("packsquash")
        .args(["packsquash.toml", "--pack-directory", &src, "--output-file-path", &dest])
        .status()
        .expect("Failed to execute PackSquash");

    if !status.success() { exit(1); }
}

fn build_datapack(pack_id: &str, sha: &str) {
    println!("zipping datapack: {}", pack_id);
    let src = format!("datapacks/{}", pack_id);
    let dest = format!("artifacts/{}-{}.zip", pack_id, sha);
    if let Err(e) = zip_dir(&src, &dest) {
        eprintln!("❌ Failed to zip datapack {}: {}", pack_id, e);
        exit(1);
    }
}

fn zip_dir(src: &str, dest: &str) -> zip::result::ZipResult<()> {
    let file = File::create(dest)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(src)).unwrap();

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}
// say wallahi bro make this shit work!