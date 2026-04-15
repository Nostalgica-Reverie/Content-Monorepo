use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

fn main() {
    let sync_map = vec![
        ("modpacks/lce-core/1.21.10-mr", vec![
            "modpacks/simply/1.21.10-mr",
            "modpacks/rc-plus/1.21.10-mr",
        ]),
        ("modpacks/lce-core/1.21.10-cf", vec![
            "modpacks/simply/1.21.10-cf",
            "modpacks/rc-plus/1.21.10-cf",
        ]),
    ];

    for (src_str, targets) in sync_map {
        let src_path = Path::new(src_str);
        if !src_path.exists() {
            println!("Skipping source {}: Not found", src_str);
            continue;
        }

        for target_str in targets {
            let target_path = Path::new(target_str);
            if !target_path.exists() { continue; }

            println!("Syncing {} -> {}", src_str, target_str);
            
            sync_subfolder(src_path, target_path, "mods");
            sync_subfolder(src_path, target_path, "resourcepacks");
            sync_subfolder(src_path, target_path, "resources");

            refresh_packwiz(target_path);
        }
    }
}

fn sync_subfolder(src_root: &Path, dst_root: &Path, folder: &str) {
    let src = src_root.join(folder);
    let dst = dst_root.join(folder);

    if !src.exists() { return; }

    for entry in WalkDir::new(&src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.path().is_dir()) 
    {
        let src_file = entry.path();
        let relative = src_file.strip_prefix(src_root).unwrap();
        let target_file = dst_root.join(relative);

        if let Some(parent) = target_file.parent() {
            fs::create_dir_all(parent).expect("Failed to create dirs");
        }

        fs::copy(src_file, &target_file).expect("Failed to copy file");
    }
}

fn refresh_packwiz(dir: &Path) {
    let _ = Command::new("packwiz")
        .arg("refresh")
        .current_dir(dir)
        .status();
}