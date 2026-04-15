use serde_json::Value;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

fn main() {
    let diff_output = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
        .output()
        .expect("failed to run git diff");

    let diff_str = String::from_utf8_lossy(&diff_output.stdout);
    let manifest_path_str = diff_str.lines().find(|l| l.ends_with("manifest.json"));

    let manifest_path_str = match manifest_path_str {
        Some(p) => p,
        None => {
            println!("no manifest.json found in recent commit. skipping.");
            std::process::exit(0);
        }
    };

    let manifest_path = Path::new(manifest_path_str);
    let p_dir = manifest_path.parent().expect("Could not find parent dir");

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

    let changelog_file = p_dir.join("changelog.md");
    let mut notes = if changelog_file.exists() {
        fs::read_to_string(&changelog_file).unwrap()
    } else {
        format!("update for {}", raw_name)
    };

    let prev_bump = Command::new("git")
        .args(["log", "-n", "2", "--format=%H", "--", manifest_path_str])
        .output()
        .expect("Failed to get git log for manifest");
    
    let prev_hashes = String::from_utf8_lossy(&prev_bump.stdout);
    let prev_hash = prev_hashes.lines().nth(1).unwrap_or("HEAD~1");

    let logs = Command::new("git")
        .args(["log", &format!("{}..HEAD", prev_hash), "--format=%h %s - %an", "--", p_dir.to_str().unwrap()])
        .output()
        .expect("Failed to get commit logs");

    let commit_lines: Vec<&str> = std::str::from_utf8(&logs.stdout)
        .unwrap()
        .lines()
        .filter(|l| l.contains(": ")) 
        .collect();

    if !commit_lines.is_empty() {
        if !notes.contains("# :purple_circle: Meta-changes") {
            notes.push_str("\n\n# :purple_circle: Meta-changes\n");
        }
        notes.push_str("\n### Automated Commit Log\n");
        for line in commit_lines {
            notes.push_str(&format!("{}\n", line));
        }
    }

    let workspace = env::var("GITHUB_WORKSPACE").unwrap_or_else(|_| ".".to_string());
    let artifacts_dir = Path::new(&workspace).join(p_dir).join("artifacts");

    if artifacts_dir.exists() {
        fs::remove_dir_all(&artifacts_dir).unwrap();
    }
    fs::create_dir_all(&artifacts_dir).unwrap();

    println!("::group::Building Artifacts");
    if p_type == "modpack" {
        for platform in &["mr", "cf"] {
            let target_folder = format!("{}-{}", mc_ver, platform);
            let target_path = p_dir.join(&target_folder);

            if target_path.exists() {
                // Refresh packwiz
                run_cmd("packwiz", &["refresh"], &target_path);

                let (export_cmd, ext) = if *platform == "mr" { ("modrinth", "mrpack") } else { ("curseforge", "zip") };
                let out_file = artifacts_dir.join(format!("{}-{}.{}", filename_base, platform, ext));

                // Export packwiz
                run_cmd("packwiz", &[export_cmd, "export", "--output", out_file.to_str().unwrap()], &target_path);
            } else {
                println!("Skipping {}: folder {} not found", platform, target_path.display());
            }
        }
    } else if p_type == "resourcepack" || p_type == "datapack" {
        let out_file = artifacts_dir.join(format!("{}-{}.zip", manifest["id"].as_str().unwrap(), p_ver));
        let content_dir = p_dir.join("content");

        if content_dir.exists() {
            run_cmd("zip", &["-r", out_file.to_str().unwrap(), "."], &content_dir);
        } else {
            println!("Error: content directory not found at {}", content_dir.display());
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

        let delimiter = "EOF_NOTES_DELIMITER";
        writeln!(out_file, "notes<<{}\n{}\n{}", delimiter, notes.trim(), delimiter).unwrap();
    }
}

fn run_cmd(cmd: &str, args: &[&str], dir: &PathBuf) {
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
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("CRITICAL: Binary '{}' not found in PATH.", cmd);
            eprintln!("Ensure it is installed and added to GITHUB_PATH.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to execute '{}': {}", cmd, e);
            std::process::exit(1);
        }
    }
}