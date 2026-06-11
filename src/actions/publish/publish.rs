use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
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
        Some("upload") => {
            let manifest_path = args
                .get(2)
                .ok_or_else(|| anyhow!("usage: publish upload <manifest> [variant] [--live]"))?;
            let mut variant: Option<&str> = None;
            let mut live = false;
            for a in args.iter().skip(3) {
                if a == "--live" {
                    live = true;
                } else {
                    variant = Some(a.as_str());
                }
            }
            cmd_upload(Path::new(manifest_path), variant, live)
        }
        Some("verify") => {
            let manifest_path = args.get(2).ok_or_else(|| anyhow!("usage: publish verify <manifest> [variant]"))?;
            let variant = args.get(3).map(String::as_str);
            cmd_verify(Path::new(manifest_path), variant)
        }
        _ => bail!("usage: publish <list|build|upload|verify> <manifest> [variant] [--live]"),
    }
}

fn packwiz_bin() -> String {
    env::var("PACKWIZ_BIN").unwrap_or_else(|_| "packwiz".into())
}

fn read_manifest(manifest_path: &Path) -> Result<Value> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("invalid JSON in {}", manifest_path.display()))
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key].as_str().ok_or_else(|| anyhow!("missing or non-string '{key}'"))
}

struct Resolved {
    p_name: String,
    raw_name: String,
    p_type: String,
    loader: String,
    release_type: String,
    mr_id: String,
    cf_id: String,
    subdir_key: String,
    mc_ver: String,
    p_ver: String,
    display_name: String,
    is_experimental: bool,
}

fn resolve(manifest_path: &Path, variant: Option<&str>) -> Result<Resolved> {
    let filename = manifest_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid manifest filename"))?;
    let is_experimental = filename == "manifest-experimental.json";

    let manifest = read_manifest(manifest_path)?;

    let raw_name = required_str(&manifest, "name")?.to_string();
    let p_name = raw_name.replace(' ', "-");
    let p_type = required_str(&manifest, "type")?.to_string();
    let pack_loader = manifest["loader"].as_str().unwrap_or("").to_string();
    let release_type = required_str(&manifest, "release_type")?.to_string();
    let id = required_str(&manifest, "id")?.to_string();
    let mr_id = manifest["modrinth_id"].as_str().unwrap_or("").to_string();
    let cf_id = manifest["curseforge_id"].as_str().unwrap_or("").to_string();
    if mr_id.is_empty() && cf_id.is_empty() {
        bail!("manifest must set at least one of modrinth_id or curseforge_id");
    }

    let (subdir_key, mc_ver, variant_name, variant_version, variant_loader): (
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = if let Some(vkey) = variant {
        let variants = manifest
            .get("variants")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("variant '{vkey}' requested but manifest has no 'variants'"))?;
        let v = variants
            .iter()
            .find(|v| {
                let k = v
                    .get("id")
                    .and_then(|x| x.as_str())
                    .or_else(|| v.get("mc_version").and_then(|x| x.as_str()));
                k == Some(vkey)
            })
            .ok_or_else(|| anyhow!("variant '{vkey}' not found in manifest"))?;
        let mc = v
            .get("mc_version")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow!("variant '{vkey}' missing mc_version"))?
            .to_string();
        let name = v.get("name").and_then(|x| x.as_str()).map(String::from);
        let ver = v.get("version").and_then(|x| x.as_str()).map(String::from);
        let vloader = v.get("loader").and_then(|x| x.as_str()).map(String::from);
        (vkey.to_string(), mc, name, ver, vloader)
    } else {
        let mc = required_str(&manifest, "mc_version")?.to_string();
        (mc.clone(), mc, None, None, None)
    };

    let loader = variant_loader.unwrap_or(pack_loader);
    if p_type == "modpack" && loader.is_empty() {
        bail!("no loader resolved for '{subdir_key}': set a pack-level 'loader' or a variant 'loader'");
    }

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

    Ok(Resolved {
        p_name,
        raw_name,
        p_type,
        loader,
        release_type,
        mr_id,
        cf_id,
        subdir_key,
        mc_ver,
        p_ver,
        display_name,
        is_experimental,
    })
}

fn cmd_list(manifest_path: &Path) -> Result<()> {
    let manifest = read_manifest(manifest_path)?;
    let manifest_str = manifest_path.to_str().ok_or_else(|| anyhow!("non-UTF8 manifest path"))?;

    let mut entries: Vec<Value> = Vec::new();

    if let Some(variants) = manifest.get("variants").and_then(|v| v.as_array()) {
        for (idx, v) in variants.iter().enumerate() {
            let key = v
                .get("id")
                .and_then(|x| x.as_str())
                .or_else(|| v.get("mc_version").and_then(|x| x.as_str()))
                .ok_or_else(|| anyhow!("variant missing both 'id' and 'mc_version'"))?;
            entries.push(json!({ "manifest": manifest_str, "variant": key, "order": idx }));
        }
    } else {
        entries.push(json!({ "manifest": manifest_str, "variant": Value::Null, "order": 0 }));
    }

    println!("{}", serde_json::to_string(&entries)?);
    Ok(())
}

fn cmd_verify(manifest_path: &Path, variant: Option<&str>) -> Result<()> {
    let r = resolve(manifest_path, variant)?;
    if r.mr_id.is_empty() {
        bail!("verify currently checks Modrinth only, and this manifest has no modrinth_id");
    }
    let url = format!("https://api.modrinth.com/v2/project/{}/version", r.mr_id);
    let versions: Vec<serde_json::Value> = match ureq::get(&url).call() {
        Ok(resp) => resp.into_json().context("parsing Modrinth version list")?,
        Err(ureq::Error::Status(code, resp)) => {
            let detail = resp.into_string().unwrap_or_default();
            bail!("Modrinth version lookup failed (HTTP {code}): {detail}");
        }
        Err(e) => bail!("Modrinth version lookup failed: {e}"),
    };
    let found = versions.iter().find(|v| v["version_number"].as_str() == Some(r.p_ver.as_str()));
    match found {
        Some(v) => {
            let vid = v["id"].as_str().unwrap_or("?");
            let published = v["date_published"].as_str().unwrap_or("?");
            println!("verified: {} {} is live on Modrinth (version id {vid}, published {published})", r.display_name, r.p_ver);
            Ok(())
        }
        None => bail!(
            "version '{}' NOT found on Modrinth project '{}' ({} version(s) listed) — upload may have failed",
            r.p_ver, r.mr_id, versions.len()
        ),
    }
}

fn cmd_build(manifest_path: &Path, variant: Option<&str>) -> Result<()> {
    let p_dir = manifest_path.parent().ok_or_else(|| anyhow!("manifest has no parent dir"))?;
    let manifest = read_manifest(manifest_path)?;
    let r = resolve(manifest_path, variant)?;

    let workspace = env::var("GITHUB_WORKSPACE").unwrap_or_else(|_| ".".into());
    let artifacts_dir = Path::new(&workspace).join(p_dir).join("artifacts");
    if artifacts_dir.exists() {
        fs::remove_dir_all(&artifacts_dir)?;
    }
    fs::create_dir_all(&artifacts_dir)?;

    let label = match (r.is_experimental, variant) {
        (true, Some(v)) => format!("EXPERIMENTAL {} [{v}] ({})", r.raw_name, r.p_ver),
        (true, None) => format!("EXPERIMENTAL {} ({})", r.raw_name, r.p_ver),
        (false, Some(v)) => format!("{} [{v}]", r.raw_name),
        (false, None) => r.raw_name.clone(),
    };
    println!("::group::Building {label}");

    match r.p_type.as_str() {
        "modpack" => build_modpack(
            p_dir,
            &artifacts_dir,
            &r.p_name,
            &r.subdir_key,
            &r.mc_ver,
            &r.p_ver,
            &r.loader,
            &r.mr_id,
            &r.cf_id,
        )?,
        "datapack" => build_datapack(p_dir, &artifacts_dir, &manifest, &r.p_ver)?,
        other => bail!("unsupported pack type: {other}"),
    }
    println!("::endgroup::");

    write_outputs(OutputData {
        mr_id: &r.mr_id,
        cf_id: &r.cf_id,
        name: &r.display_name,
        p_ver: &r.p_ver,
        mc_ver: &r.mc_ver,
        p_type: &r.p_type,
        loader: &r.loader,
        release_type: &r.release_type,
        p_dir,
        is_experimental: r.is_experimental,
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

        let status = Command::new(packwiz_bin())
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

const MODRINTH_API: &str = "https://api.modrinth.com/v2";
const CURSEFORGE_API: &str = "https://minecraft.curseforge.com/api";

fn cmd_upload(manifest_path: &Path, variant: Option<&str>, live: bool) -> Result<()> {
    let p_dir = manifest_path.parent().ok_or_else(|| anyhow!("manifest has no parent dir"))?;
    let r = resolve(manifest_path, variant)?;

    if r.p_type != "modpack" {
        bail!("upload currently supports modpacks only (got '{}')", r.p_type);
    }

    let changelog_path = p_dir.join("changelog.md");
    let changelog = fs::read_to_string(&changelog_path)
        .unwrap_or_else(|_| format!("Update for {}", r.raw_name));

    let workspace = env::var("GITHUB_WORKSPACE").unwrap_or_else(|_| ".".into());
    let artifacts_dir = Path::new(&workspace).join(p_dir).join("artifacts");
    let filename_base = format!("{}-{}-{}-{}", r.p_name, r.mc_ver, r.loader, r.p_ver);

    if !live {
        println!("[DRY RUN] publish upload — nothing will be sent (pass --live to upload)");
    }

    let mut attempted = 0usize;
    let mut uploaded = 0usize;

    for platform in [Platform::Modrinth, Platform::Curseforge] {
        let plat_id = match platform {
            Platform::Modrinth => &r.mr_id,
            Platform::Curseforge => &r.cf_id,
        };
        if plat_id.is_empty() {
            continue;
        }
        let artifact = artifacts_dir.join(format!("{filename_base}-{}.{}", platform.short(), platform.ext()));
        if !artifact.exists() {
            println!(
                "skipping {}: artifact {} not found (run 'publish build' first)",
                platform.short(),
                artifact.display()
            );
            continue;
        }
        attempted += 1;
        let bytes = fs::read(&artifact).with_context(|| format!("reading {}", artifact.display()))?;
        let file_name = artifact
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("non-UTF8 artifact name"))?
            .to_string();

        match platform {
            Platform::Modrinth => {
                upload_modrinth(&r, plat_id, &changelog, &file_name, &bytes, live)?;
            }
            Platform::Curseforge => {
                upload_curseforge(&r, plat_id, &changelog, &file_name, &bytes, live)?;
            }
        }
        uploaded += 1;
    }

    if attempted == 0 {
        bail!(
            "no artifacts found for '{}' in {} — run 'publish build' before 'publish upload'",
            r.subdir_key,
            artifacts_dir.display()
        );
    }
    let mode = if live { "uploaded" } else { "validated (dry run)" };
    println!("{uploaded} artifact(s) {mode} for {}", r.display_name);
    Ok(())
}

fn upload_modrinth(
    r: &Resolved,
    project_id: &str,
    changelog: &str,
    file_name: &str,
    bytes: &[u8],
    live: bool,
) -> Result<()> {
    let data = json!({
        "project_id": project_id,
        "name": r.display_name,
        "version_number": r.p_ver,
        "changelog": changelog,
        "dependencies": [],
        "game_versions": [r.mc_ver],
        "version_type": r.release_type,
        "loaders": [r.loader],
        "featured": false,
        "file_parts": ["file"],
        "primary_file": "file"
    });

    println!(
        "modrinth: {} -> project {} | version {} | mc {} | loader {} | {} bytes",
        file_name, project_id, r.p_ver, r.mc_ver, r.loader, bytes.len()
    );
    if !live {
        return Ok(());
    }
    let token = env::var("MODRINTH_TOKEN").context("MODRINTH_TOKEN not set")?;

    let (content_type, body) = multipart(&[
        Part { name: "data", file_name: None, content_type: "application/json", bytes: data.to_string().as_bytes().to_vec() },
        Part { name: "file", file_name: Some(file_name), content_type: "application/octet-stream", bytes: bytes.to_vec() },
    ]);

    let resp = ureq::post(&format!("{MODRINTH_API}/version"))
        .set("Authorization", &token)
        .set("Content-Type", &content_type)
        .send_bytes(&body);

    match resp {
        Ok(_) => {
            println!("modrinth: uploaded {} to {}", r.p_ver, project_id);
            Ok(())
        }
        Err(ureq::Error::Status(code, response)) => {
            let detail = response.into_string().unwrap_or_default();
            bail!("modrinth upload failed (HTTP {code}): {detail}");
        }
        Err(e) => bail!("modrinth upload failed: {e}"),
    }
}

fn upload_curseforge(
    r: &Resolved,
    project_id: &str,
    changelog: &str,
    file_name: &str,
    bytes: &[u8],
    live: bool,
) -> Result<()> {
    println!(
        "curseforge: {} -> project {} | version {} | mc {} | loader {} | {} bytes",
        file_name, project_id, r.p_ver, r.mc_ver, r.loader, bytes.len()
    );
    if !live {
        return Ok(());
    }
    let token = env::var("CURSEFORGE_TOKEN").context("CURSEFORGE_TOKEN not set")?;

    let version_ids = cf_game_version_ids(&token, &r.mc_ver, &r.loader)?;

    let metadata = json!({
        "changelog": changelog,
        "changelogType": "markdown",
        "displayName": r.display_name,
        "gameVersions": version_ids,
        "releaseType": r.release_type
    });

    let (content_type, body) = multipart(&[
        Part { name: "metadata", file_name: None, content_type: "application/json", bytes: metadata.to_string().as_bytes().to_vec() },
        Part { name: "file", file_name: Some(file_name), content_type: "application/octet-stream", bytes: bytes.to_vec() },
    ]);

    let resp = ureq::post(&format!("{CURSEFORGE_API}/projects/{project_id}/upload-file"))
        .set("X-Api-Token", &token)
        .set("Content-Type", &content_type)
        .send_bytes(&body);

    match resp {
        Ok(_) => {
            println!("curseforge: uploaded {} to {}", r.p_ver, project_id);
            Ok(())
        }
        Err(ureq::Error::Status(code, response)) => {
            let detail = response.into_string().unwrap_or_default();
            bail!("curseforge upload failed (HTTP {code}): {detail}");
        }
        Err(e) => bail!("curseforge upload failed: {e}"),
    }
}

fn cf_game_version_ids(token: &str, mc_ver: &str, loader: &str) -> Result<Vec<i64>> {
    let resp = ureq::get(&format!("{CURSEFORGE_API}/game/versions"))
        .set("X-Api-Token", token)
        .call();
    let versions: Value = match resp {
        Ok(response) => response.into_json().context("parsing CF game versions")?,
        Err(ureq::Error::Status(code, response)) => {
            let detail = response.into_string().unwrap_or_default();
            bail!("CF game/versions lookup failed (HTTP {code}): {detail}");
        }
        Err(e) => bail!("CF game/versions lookup failed: {e}"),
    };
    let list = versions.as_array().ok_or_else(|| anyhow!("unexpected CF versions payload"))?;

    let mut ids = Vec::new();
    let loader_lc = loader.to_lowercase();
    for entry in list {
        let name = entry["name"].as_str().unwrap_or("");
        let slug = entry["slug"].as_str().unwrap_or("");
        let id = entry["id"].as_i64();
        if let Some(id) = id {
            if name == mc_ver || slug == loader_lc || name.to_lowercase() == loader_lc {
                ids.push(id);
            }
        }
    }
    if ids.len() < 2 {
        bail!(
            "could not resolve CF game-version IDs for mc '{mc_ver}' + loader '{loader}' (matched {} of 2) — check the CF versions list",
            ids.len()
        );
    }
    Ok(ids)
}

struct Part<'a> {
    name: &'a str,
    file_name: Option<&'a str>,
    content_type: &'a str,
    bytes: Vec<u8>,
}

fn multipart(parts: &[Part]) -> (String, Vec<u8>) {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let boundary = format!("----somnus-publish-{stamp}");
    let mut body: Vec<u8> = Vec::new();

    for p in parts {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        match p.file_name {
            Some(fname) => body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n", p.name, fname).as_bytes(),
            ),
            None => body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{}\"\r\n", p.name).as_bytes(),
            ),
        }
        body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", p.content_type).as_bytes());
        body.extend_from_slice(&p.bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    (format!("multipart/form-data; boundary={boundary}"), body)
}