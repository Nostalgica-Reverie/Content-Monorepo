use anyhow::{anyhow, Context, Result};
use std::{
    env,
    fs,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
use walkdir::WalkDir;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let root = args
        .get(1)
        .ok_or_else(|| anyhow!("usage: minify-json <directory>"))?;

    let root_path = Path::new(root);
    if !root_path.is_dir() {
        return Err(anyhow!("not a directory: {root}"));
    }

    let bytes_before = AtomicU64::new(0);
    let bytes_after = AtomicU64::new(0);
    let mut processed = 0usize;
    let mut skipped = 0usize;

    for entry in WalkDir::new(root_path) {
        let entry = entry.context("walkdir error")?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext != "json" && ext != "mcmeta" {
            continue;
        }

        match minify_file(path) {
            Ok((before, after)) => {
                bytes_before.fetch_add(before, Ordering::Relaxed);
                bytes_after.fetch_add(after, Ordering::Relaxed);
                processed += 1;
            }
            Err(e) => {
                eprintln!("warning: skipped {}: {e}", path.display());
                skipped += 1;
            }
        }
    }

    let before = bytes_before.load(Ordering::Relaxed);
    let after = bytes_after.load(Ordering::Relaxed);
    let saved = before.saturating_sub(after);
    let pct = if before > 0 {
        (saved as f64 / before as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "minified {processed} file(s), skipped {skipped}, saved {saved} bytes ({pct:.1}%)"
    );
    Ok(())
}

fn minify_file(path: &Path) -> Result<(u64, u64)> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("read failed: {}", path.display()))?;
    let before = content.len() as u64;

    let value: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("parse failed: {}", path.display()))?;

    let minified = serde_json::to_string(&value)
        .with_context(|| format!("serialize failed: {}", path.display()))?;
    let after = minified.len() as u64;

    if after < before {
        fs::write(path, minified)
            .with_context(|| format!("write failed: {}", path.display()))?;
    }

    Ok((before, after))
}
