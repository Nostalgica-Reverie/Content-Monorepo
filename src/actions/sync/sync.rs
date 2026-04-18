use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

const SUBFOLDERS: &[&str] = &["mods", "resourcepacks", "resources"];

const SYNC_MAP: &[(&str, &[&str])] = &[
    (
        "modpacks/lce-core/1.21.10-mr",
        &[
            "modpacks/simply/1.21.10-mr",
            "modpacks/rc-plus/1.21.10-mr",
        ],
    ),
    (
        "modpacks/lce-core/1.21.10-cf",
        &[
            "modpacks/simply/1.21.10-cf",
            "modpacks/rc-plus/1.21.10-cf",
        ],
    ),
];

fn main() -> Result<()> {
    for (src_str, targets) in SYNC_MAP {
        let src_path = Path::new(src_str);
        if !src_path.exists() {
            println!("skipping source {src_str}: not found");
            continue;
        }

        for target_str in *targets {
            let target_path = Path::new(target_str);
            if !target_path.exists() {
                println!("skipping target {target_str}: not found");
                continue;
            }

            println!("syncing {src_str} -> {target_str}");

            for folder in SUBFOLDERS {
                sync_subfolder(src_path, target_path, folder)
                    .with_context(|| format!("failed syncing '{folder}' to {target_str}"))?;
            }

            refresh_packwiz(target_path)
                .with_context(|| format!("packwiz refresh failed in {target_str}"))?;
        }
    }

    println!("all syncs completed.");
    Ok(())
}

fn sync_subfolder(src_root: &Path, dst_root: &Path, folder: &str) -> Result<()> {
    let src = src_root.join(folder);
    if !src.exists() {
        return Ok(());
    }

    let mut copied = 0usize;

    for entry in WalkDir::new(&src) {
        let entry = entry.context("walkdir error")?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let relative = path.strip_prefix(src_root).context("strip_prefix failed")?;
        let target_file = dst_root.join(relative);

        if let Some(parent) = target_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        fs::copy(path, &target_file).with_context(|| {
            format!(
                "failed to copy {} -> {}",
                path.display(),
                target_file.display()
            )
        })?;
        copied += 1;
    }

    println!("  {folder}: {copied} file(s) copied");
    Ok(())
}

fn refresh_packwiz(dir: &Path) -> Result<()> {
    let status = Command::new("packwiz")
        .args(["refresh", "-y"])
        .current_dir(dir)
        .status()
        .context("failed to invoke packwiz")?;
    if !status.success() {
        bail!("packwiz refresh exited with {status}");
    }
    Ok(())
}