use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy)]
enum Platform {
    Modrinth,
    Curseforge,
}
impl Platform {
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

    let mode = args.get(1).map(String::as_str);
    match mode {
        Some("list") => {
            let manifest_path = args.get(2).ok_or_else(|| anyhow!("usage: publish list <manifest>"))?;
            cmd_list(Path::new(manifest_path))
        }
        Some("build") => {
            let manifest_path = args.get(2).ok_or_else(|| anyhow!("usage: publish build <manifest> [variant]"))?;
            let variant = args.get(3).map(String::as_str);
            cmd_build(Path::new(manifest_path), variant)
        }
        _ => bail!("usage: publish <list|build> <manifest> [variant]"),
    }
}

fn read_manifest(manifest_path: &Path) -> Result<Value> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("invalid JSON in {}", manifest_path.display()))
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key].as_str().ok_or_else(|| anyhow!("missing or non-string '{key}'"))
}

fn cmd_list(manifest_path: &Path) -> Result<()> {
    let manifest = read_manifest(manifest_path)?;
    let manifest_str = manifest_path.to_str().ok_or_else(|| anyhow!("non-UTF8 manifest path"))?;

    let mut entries: Vec<Value> = Vec::new();

    if let Some(variants) = manifest.get("variants").and_then(|v| v.as_array()) {
        for v in variants {
            let key = v
                .get("id")
                .and_then(|x| x.as_str())
                .or_else(|| v.get("mc_version").and_then(|x| x.as_str()))
                .ok_or_else(|| anyhow!("variant missing both 'id' and 'mc_version'"))?;
            entries.push(json!({ "manifest": manifest_str, "variant": key }));
        }
    } else {
        entries.push(json!({ "manifest": manifest_str, "variant": Value::Null }));
    }

    println!("{}", serde_json::to_string(&entries)?);
    Ok(())
}

fn cmd_build(manifest_path: &Path, variant: Option<&str>) -> Result<()> {
    let p_dir = manifest_path.parent().ok_or_else(|| anyhow!("manifest has no parent dir"))?;
    let filename = manifest_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid manifest filename"))?;
    let is_experimental = filename == "manifest-experimental.json";

    let manifest = read_manifest(manifest_path)?;

    let raw_name = required_str(&manifest, "name")?;
    let p_name = raw_name.replace(' ', "-");
    let p_type = required_str(&manifest, "type")?;
    let loader = required_str(&manifest, "loader")?;
    let release_type = required_str(&manifest, "release_type")?;
    let id = required_str(&manifest, "id")?;
    let mr_id = manifest["modrinth_id"].as_str().unwrap_or("");
    let cf_id = manifest["curseforge_id"].as_str().unwrap_or("");
    if mr_id.is_empty() && cf_id.is_empty() {
        bail!("manifest must set at least one of modrinth_id or curseforge_id");
    }

    let (subdir_key, mc_ver, variant_name, variant_version): (String, String, Option<String>, Option<String>) =
        if let Some(vkey) = variant {
            let variants = manifest
                .get("variants")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow!("variant '{vkey}' requested but manifest has no 'variants'"))?;
            let v = variants
                .iter()
                .find(|v| {
                    let k = v.get("id").and_then(|x| x.as_str())
                        .or_else(|| v.get("mc_version").and_then(|x| x.as_str()));
                    k == Some(vkey)
                })
                .ok_or_else(|| anyhow!("variant '{vkey}' not found in manifest"))?;
            let mc = v.get("mc_version").and_then(|x| x.as_str())
                .ok_or_else(|| anyhow!("variant '{vkey}' missing mc_version"))?
                .to_string();
            let name = v.get("name").and_then(|x| x.as_str()).map(String::from);
            let ver = v.get("version").and_then(|x| x.as_str()).map(String::from);
            (vkey.to_string(), mc, name, ver)
        } else {
            let mc = required_str(&manifest, "mc_version")?.to_string();
            (mc.clone(), mc, None, None)
        };

    let p_ver: String = if is_experimental {
        let sha = env::var("GITHUB_SHA").context("GITHUB_SHA not set; required for experimental")?;
        let short: String = sha.chars().take(7).collect();
        let cycle = Utc::now().format("%y.%m").to_string();
        match variant {
            Some(vkey) => format!("{id}-{vkey}-{cycle}-{short}"),
            None => format!("{id}-{cycle}-{short}"),
        }
    } else {
        let base_ver = variant_version
            .or_else(|| manifest["version"].as_str().map(String::from))
            .ok_or_else(|| anyhow!("missing 'version'"))?;
        match variant {
            Some(vkey) => format!("{base_ver}-{vkey}"),
            None => base_ver,
        }
    };

    let workspace = env::var("GITHUB_WORKSPACE").unwrap_or_else(|_| ".".into());
    let artifacts_dir = Path::new(&workspace).join(p_dir).join("artifacts");
    if artifacts_dir.exists() {
        fs::remove_dir_all(&artifacts_dir)?;
    }
    fs::create_dir_all(&artifacts_dir)?;

    let label = match (is_experimental, variant) {
        (true, Some(v)) => format!("EXPERIMENTAL {raw_name} [{v}] ({p_ver})"),
        (true, None) => format!("EXPERIMENTAL {raw_name} ({p_ver})"),
        (false, Some(v)) => format!("{raw_name} [{v}]"),
        (false, None) => raw_name.to_string(),
    };
    println!("::group::Building {label}");

    match p_type {
        "modpack" => build_modpack(p_dir, &artifacts_dir, &p_name, &subdir_key, &mc_ver, &p_ver, loader, mr_id, cf_id)?,
        "datapack" => build_datapack(p_dir, &artifacts_dir, &manifest, &p_ver)?,
        other => bail!("unsupported pack type: {other}"),
    }
    println!("::endgroup::");

    let display_name = match (&variant_name, variant) {
        (Some(vn), _) => format!("{raw_name} {vn} {p_ver}"),
        (None, Some(v)) => format!("{raw_name} {v} {p_ver}"),
        (None, None) => format!("{raw_name} {p_ver}"),
    };
    let display_name = if is_experimental {
        format!("[EXPERIMENTAL] {display_name}")
    } else {
        display_name
    };

    write_outputs(OutputData {
        mr_id,
        cf_id,
        name: &display_name,
        p_ver: &p_ver,
        mc_ver: &mc_ver,
        p_type,
        loader,
        release_type,
        p_dir,
        is_experimental,
    })?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_modpack(
    p_dir: &Path,
    artifacts_dir: &Path,
    p_name: &str,
    subdir_key: &str,
    mc_ver: &str,
    p_ver: &str,
    loader: &str,
    mr_id: &str,
    cf_id: &str,
) -> Result<()> {
    let filename_base = format!("{p_name}-{mc_ver}-{loader}-{p_ver}");
    let mut built = 0usize;

    for (platform, plat_id) in [(Platform::Modrinth, mr_id), (Platform::Curseforge, cf_id)] {
        if plat_id.is_empty() {
            continue;
        }
        let target_folder = format!("{subdir_key}-{}", platform.short());
        let target_path = p_dir.join(&target_folder);
        if !target_path.exists() {
            println!("skipping {}: folder {} not found", platform.short(), target_path.display());
            continue;
        }
        let out_file = artifacts_dir.join(format!("{filename_base}-{}.{}", platform.short(), platform.ext()));
        let out_str = out_file.to_str().ok_or_else(|| anyhow!("non-UTF8 output path"))?;

        let status = Command::new("packwiz")
            .args([platform.cli(), "export", "--output", out_str])
            .current_dir(&target_path)
            .status()
            .context("failed to invoke packwiz export")?;
        if !status.success() {
            bail!("packwiz export failed for {}", target_path.display());
        }
        println!("exported {}", out_file.display());
        built += 1;
    }

    if built == 0 {
        bail!("no platform folders found for subdir key '{subdir_key}' (expected {subdir_key}-mr / {subdir_key}-cf)");
    }
    Ok(())
}

fn build_datapack(p_dir: &Path, artifacts_dir: &Path, manifest: &Value, p_ver: &str) -> Result<()> {
    let id = manifest["id"].as_str().ok_or_else(|| anyhow!("datapack missing 'id'"))?;
    let out_file = artifacts_dir.join(format!("{id}-{p_ver}.zip"));
    let content_dir = p_dir.join("content");
    if !content_dir.exists() {
        bail!("content directory not found at {}", content_dir.display());
    }
    let out_str = out_file.to_str().ok_or_else(|| anyhow!("non-UTF8 output path"))?;
    let status = Command::new("zip").args(["-r", out_str, "."]).current_dir(&content_dir).status()?;
    if !status.success() {
        bail!("zip failed");
    }
    Ok(())
}

struct OutputData<'a> {
    mr_id: &'a str,
    cf_id: &'a str,
    name: &'a str,
    p_ver: &'a str,
    mc_ver: &'a str,
    p_type: &'a str,
    loader: &'a str,
    release_type: &'a str,
    p_dir: &'a Path,
    is_experimental: bool,
}

fn write_outputs(d: OutputData) -> Result<()> {
    let Ok(out_path) = env::var("GITHUB_OUTPUT") else { return Ok(()); };
    let mut f = OpenOptions::new().append(true).create(true).open(&out_path)?;
    writeln!(f, "mr_id={}", d.mr_id)?;
    writeln!(f, "cf_id={}", d.cf_id)?;
    writeln!(f, "name={}", d.name)?;
    writeln!(f, "ver={}", d.p_ver)?;
    writeln!(f, "mc={}", d.mc_ver)?;
    writeln!(f, "type={}", d.p_type)?;
    writeln!(f, "loader={}", d.loader)?;
    writeln!(f, "release_type={}", d.release_type)?;
    writeln!(f, "path={}", d.p_dir.display())?;
    writeln!(f, "is_experimental={}", d.is_experimental)?;
    Ok(())
}
