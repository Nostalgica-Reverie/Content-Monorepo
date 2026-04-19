use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
    thread,
};

const PACKS: &[&str] = &["simply", "rc-plus", "2k", "rekindled"];
const MAX_CONCURRENT: usize = 8;

fn main() -> Result<()> {
    let mut jobs: Vec<PathBuf> = Vec::new();
    for pack in PACKS {
        let pack_dir = PathBuf::from("modpacks").join(pack);
        if !pack_dir.exists() {
            eprintln!("warning: pack directory missing: {}", pack_dir.display());
            continue;
        }

        let entries = fs::read_dir(&pack_dir)
            .with_context(|| format!("failed to read {}", pack_dir.display()))?;

        for entry in entries {
            let entry = entry.context("failed to read directory entry")?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if name.ends_with("-mr") || name.ends_with("-cf") {
                jobs.push(path);
            }
        }
    }

    if jobs.is_empty() {
        println!("no packs to update.");
        return Ok(());
    }

    println!(
        "queued {} subdir(s) across {} pack(s), running up to {} in parallel",
        jobs.len(),
        PACKS.len(),
        MAX_CONCURRENT,
    );

    let jobs = Arc::new(Mutex::new(jobs.into_iter()));
    let failures: Arc<Mutex<Vec<(PathBuf, String)>>> = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();
    for worker_id in 0..MAX_CONCURRENT {
        let jobs = Arc::clone(&jobs);
        let failures = Arc::clone(&failures);

        handles.push(thread::spawn(move || {
            loop {
                let job = { jobs.lock().unwrap().next() };
                let Some(path) = job else { break };

                let label = path.display().to_string();
                println!("[W{worker_id}] updating {label}");

                let output = Command::new("packwiz")
                    .args(["update", "-a", "-y"])
                    .current_dir(&path)
                    .output();

                match output {
                    Ok(o) if o.status.success() => {
                        println!("[W{worker_id}] ok: {label}");
                    }
                    Ok(o) => {
                        let stderr = String::from_utf8_lossy(&o.stderr).into_owned();
                        let stdout = String::from_utf8_lossy(&o.stdout).into_owned();
                        eprintln!("[W{worker_id}] FAIL {label} (exit {})", o.status);
                        if !stdout.is_empty() {
                            eprintln!("  stdout:\n{}", indent(&stdout, "    "));
                        }
                        if !stderr.is_empty() {
                            eprintln!("  stderr:\n{}", indent(&stderr, "    "));
                        }
                        let reason = if !stderr.is_empty() { stderr } else { stdout };
                        failures.lock().unwrap().push((path, reason));
                    }
                    Err(e) => {
                        eprintln!("[W{worker_id}] FAIL {label}: could not launch packwiz: {e}");
                        failures.lock().unwrap().push((path, e.to_string()));
                    }
                }
            }
        }));
    }

    for h in handles {
        h.join().expect("worker thread panicked");
    }

    let failures = failures.lock().unwrap();
    if failures.is_empty() {
        println!("\nall updates finished successfully.");
        Ok(())
    } else {
        eprintln!("\n{} subdir(s) failed:", failures.len());
        for (path, _reason) in failures.iter() {
            eprintln!("  - {}", path.display());
        }
        bail!("{} update(s) failed", failures.len())
    }
}

fn indent(s: &str, prefix: &str) -> String {
    s.lines()
        .map(|l| format!("{prefix}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}
