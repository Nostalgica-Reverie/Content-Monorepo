use anyhow::{anyhow, bail, Context, Result};
use std::{
    collections::HashSet,
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::Command,
    thread,
};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

#[derive(Debug, Clone, Copy)]
enum Platform {
    Modrinth,
    Curseforge,
}

impl Platform {
    fn from_suffix(s: &str) -> Option<Self> {
        match s {
            "mr" => Some(Platform::Modrinth),
            "cf" => Some(Platform::Curseforge),
            _ => None,
        }
    }

    fn short(self) -> &'static str {
        match self {
            Platform::Modrinth => "mr",
            Platform::Curseforge => "cf",
        }
    }

    fn cli(self) -> &'static str {
        match self {
            Platform::Modrinth => "modrinth",
            Platform::Curseforge => "curseforge",
        }
    }

    fn ext(self) -> &'static str {
        match self {
            Platform::Modrinth => "mrpack",
            Platform::Curseforge => "zip",
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let short_sha = args
        .get(1)
        .ok_or_else(|| anyhow!("usage: builder <short-sha>"))?
        .clone();

    let repo_root = env::current_dir().context("failed to get current directory")?;
    let artifacts_dir = repo_root.join("artifacts");
    fs::create_dir_all(&artifacts_dir)
        .with_context(|| format!("failed to create {}", artifacts_dir.display()))?;

    let changed = detect_changed_targets().context("failed to detect changed targets")?;

    if changed.is_empty() {
        println!("no packs detected in git diff.");
        return Ok(());
    }

    for (category, pack_id) in &changed {
        match category.as_str() {
            "modpacks" => build_modpack(pack_id, &short_sha, &artifacts_dir)
                .with_context(|| format!("modpack '{pack_id}' failed"))?,
            "datapacks" => build_datapack(pack_id, &short_sha, &artifacts_dir)
                .with_context(|| format!("datapack '{pack_id}' failed"))?,
            other => println!("category '{other}' does not require a build."),
        }
    }

    println!("all builds completed successfully.");
    Ok(())
}

fn detect_changed_targets() -> Result<HashSet<(String, String)>> {
    println!("detecting changed files...");
    let output = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
        .output()
        .context("failed to invoke git")?;

    if !output.status.success() {
        bail!(
            "git diff-tree failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut targets = HashSet::new();

    for line in stdout.lines() {
        if line.is_empty() || line.starts_with("external/") || line.starts_with(".actions/") {
            continue;
        }
        let mut parts = line.splitn(3, '/');
        if let (Some(cat), Some(pack)) = (parts.next(), parts.next()) {
            targets.insert((cat.to_string(), pack.to_string()));
        }
    }

    Ok(targets)
}

fn build_modpack(pack_id: &str, sha: &str, artifacts_dir: &Path) -> Result<()> {
    println!("building modpack: {pack_id}");
    let pack_dir = PathBuf::from("modpacks").join(pack_id);

    let manifest_path = pack_dir.join("manifest.json");
    let manifest: serde_json::Value = {
        let file = File::open(&manifest_path)
            .with_context(|| format!("failed to open {}", manifest_path.display()))?;
        serde_json::from_reader(file)
            .with_context(|| format!("invalid JSON in {}", manifest_path.display()))?
    };
    let p_ver = manifest["version"]
        .as_str()
        .ok_or_else(|| anyhow!("missing 'version' in {}", manifest_path.display()))?
        .to_string();

    let mut jobs: Vec<(Platform, PathBuf, String)> = Vec::new();
    let entries = fs::read_dir(&pack_dir)
        .with_context(|| format!("failed to read {}", pack_dir.display()))?;

    for entry in entries {
        let entry = entry.context("failed to read directory entry")?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some((mc_ver, suffix)) = dir_name.rsplit_once('-') else {
            continue;
        };
        let Some(platform) = Platform::from_suffix(suffix) else {
            continue;
        };
        let mc_ver = mc_ver.to_string();
        jobs.push((platform, path, mc_ver));
    }

    if jobs.is_empty() {
        bail!("no valid version dirs (expected '{{mc_ver}}-mr' or '{{mc_ver}}-cf') for {pack_id}");
    }

    let pack_id_owned = pack_id.to_string();
    let sha_owned = sha.to_string();
    let artifacts_dir_owned = artifacts_dir.to_path_buf();

    let mut handles = Vec::new();
    for (platform, target_path, mc_ver) in jobs {
        let pack_id = pack_id_owned.clone();
        let sha = sha_owned.clone();
        let p_ver = p_ver.clone();
        let artifacts_dir = artifacts_dir_owned.clone();

        handles.push(thread::spawn(move || -> Result<()> {
            let output_name = format!(
                "{}-{}-{}-{}-{}.{}",
                pack_id,
                mc_ver,
                platform.short(),
                p_ver,
                sha,
                platform.ext()
            );
            let output_path = artifacts_dir.join(&output_name);
            let out_str = output_path
                .to_str()
                .ok_or_else(|| anyhow!("non-UTF8 output path"))?;

            let export = Command::new("packwiz")
                .args([platform.cli(), "export", "--output", out_str])
                .current_dir(&target_path)
                .status()
                .context("failed to invoke packwiz export")?;
            if !export.success() {
                bail!("packwiz export failed for {}", target_path.display());
            }

            println!("exported {output_name}");
            Ok(())
        }));
    }

    let mut errors = Vec::new();
    for h in handles {
        match h.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => errors.push(e),
            Err(_) => errors.push(anyhow!("export thread panicked")),
        }
    }

    if !errors.is_empty() {
        for e in &errors {
            eprintln!("error: {e:#}");
        }
        bail!("{} export(s) failed for {pack_id}", errors.len());
    }

    Ok(())
}

fn build_datapack(pack_id: &str, sha: &str, artifacts_dir: &Path) -> Result<()> {
    println!("zipping datapack: {pack_id}");
    let src = PathBuf::from("datapacks").join(pack_id);
    let dest = artifacts_dir.join(format!("{pack_id}-{sha}.zip"));
    zip_dir(&src, &dest).with_context(|| format!("failed to zip {}", src.display()))
}

fn zip_dir(src: &Path, dest: &Path) -> Result<()> {
    let file = File::create(dest)
        .with_context(|| format!("failed to create {}", dest.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(src) {
        let entry = entry.context("walkdir error")?;
        let path = entry.path();
        let name = path.strip_prefix(src).context("strip_prefix failed")?;

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)
                .with_context(|| format!("failed to open {}", path.display()))?;
            io::copy(&mut f, &mut zip)
                .with_context(|| format!("failed to write {} to zip", path.display()))?;
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

