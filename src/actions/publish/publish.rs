use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    thread,
};

#[derive(Clone, Copy)]
enum Platform {
    Modrinth,
    Curseforge,
}

impl Platform {
    const ALL: [Platform; 2] = [Platform::Modrinth, Platform::Curseforge];

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
    let manifest_path_str = args
        .get(1)
        .ok_or_else(|| anyhow!("usage: publish <path_to_manifest.json>"))?;
    let manifest_path = Path::new(manifest_path_str);
    let p_dir = manifest_path
        .parent()
        .ok_or_else(|| anyhow!("manifest has no parent directory"))?;

    let manifest_content = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: Value = serde_json::from_str(&manifest_content)
        .with_context(|| format!("invalid JSON in {}", manifest_path.display()))?;

    let raw_name = required_str(&manifest, "name")?;
    let p_name = raw_name.replace(' ', "-");
    let p_ver = required_str(&manifest, "version")?;
    let mc_ver = required_str(&manifest, "mc_version")?;
    let p_type = required_str(&manifest, "type")?;
    let loader = manifest["loader"].as_str().unwrap_or("fabric");
    let mr_id = manifest["modrinth_id"].as_str().unwrap_or("");
    let cf_id = manifest["curseforge_id"].as_str().unwrap_or("");

    let workspace = env::var("GITHUB_WORKSPACE").unwrap_or_else(|_| ".".into());
    let artifacts_dir = Path::new(&workspace).join(p_dir).join("artifacts");
    if artifacts_dir.exists() {
        fs::remove_dir_all(&artifacts_dir)
            .with_context(|| format!("failed to clear {}", artifacts_dir.display()))?;
    }
    fs::create_dir_all(&artifacts_dir)
        .with_context(|| format!("failed to create {}", artifacts_dir.display()))?;

    println!("::group::Building artifacts for {raw_name}");

    match p_type {
        "modpack" => build_modpack(p_dir, &artifacts_dir, &p_name, mc_ver, p_ver, loader)?,
        "datapack" => build_datapack(p_dir, &artifacts_dir, &manifest, p_ver)?,
        other => bail!("unsupported pack type: {other}"),
    }

    println!("::endgroup::");

    write_outputs(OutputData {
        mr_id,
        cf_id,
        raw_name,
        p_ver,
        mc_ver,
        p_type,
        loader,
        p_dir,
    })?;

    Ok(())
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| anyhow!("missing or non-string '{key}' in manifest"))
}

fn build_modpack(
    p_dir: &Path,
    artifacts_dir: &Path,
    p_name: &str,
    mc_ver: &str,
    p_ver: &str,
    loader: &str,
) -> Result<()> {
    let filename_base = format!("{p_name}-{mc_ver}-{loader}-{p_ver}");

    let mut jobs: Vec<(Platform, PathBuf)> = Vec::new();
    for platform in Platform::ALL {
        let target_folder = format!("{mc_ver}-{}", platform.short());
        let target_path = p_dir.join(&target_folder);
        if target_path.exists() {
            jobs.push((platform, target_path));
        } else {
            println!(
                "skipping {}: folder {} not found",
                platform.short(),
                target_path.display()
            );
        }
    }

    if jobs.is_empty() {
        bail!("no platform folders (mc_ver-mr / mc_ver-cf) found");
    }

    let mut handles = Vec::new();
    for (platform, target_path) in jobs {
        let filename_base = filename_base.clone();
        let artifacts_dir = artifacts_dir.to_path_buf();

        handles.push(thread::spawn(move || -> Result<()> {
            let out_file = artifacts_dir.join(format!(
                "{filename_base}-{}.{}",
                platform.short(),
                platform.ext()
            ));
            let out_str = out_file
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

            println!("exported {}", out_file.display());
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
        bail!("{} export(s) failed", errors.len());
    }

    Ok(())
}

fn build_datapack(p_dir: &Path, artifacts_dir: &Path, manifest: &Value, p_ver: &str) -> Result<()> {
    let id = manifest["id"]
        .as_str()
        .ok_or_else(|| anyhow!("datapack manifest missing 'id' field"))?;
    let out_file = artifacts_dir.join(format!("{id}-{p_ver}.zip"));
    let content_dir = p_dir.join("content");
    if !content_dir.exists() {
        bail!("content directory not found at {}", content_dir.display());
    }
    let out_str = out_file
        .to_str()
        .ok_or_else(|| anyhow!("non-UTF8 output path"))?;
    let status = Command::new("zip")
        .args(["-r", out_str, "."])
        .current_dir(&content_dir)
        .status()
        .context("failed to invoke zip")?;
    if !status.success() {
        bail!("zip failed for datapack");
    }
    Ok(())
}

struct OutputData<'a> {
    mr_id: &'a str,
    cf_id: &'a str,
    raw_name: &'a str,
    p_ver: &'a str,
    mc_ver: &'a str,
    p_type: &'a str,
    loader: &'a str,
    p_dir: &'a Path,
}

fn write_outputs(d: OutputData) -> Result<()> {
    let Ok(out_path) = env::var("GITHUB_OUTPUT") else {
        return Ok(());
    };
    let mut f = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&out_path)
        .with_context(|| format!("failed to open {out_path}"))?;
    writeln!(f, "mr_id={}", d.mr_id)?;
    writeln!(f, "cf_id={}", d.cf_id)?;
    writeln!(f, "name={} {}", d.raw_name, d.p_ver)?;
    writeln!(f, "ver={}", d.p_ver)?;
    writeln!(f, "mc={}", d.mc_ver)?;
    writeln!(f, "type={}", d.p_type)?;
    writeln!(f, "loader={}", d.loader)?;
    writeln!(f, "path={}", d.p_dir.display())?;
    Ok(())
}