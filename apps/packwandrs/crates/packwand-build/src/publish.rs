use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use packwand_workspace::{Manifest, Variant};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{ExportFormat, ExportOptions, export_pack};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const USER_AGENT: &str =
    "packwand/26.2.0 (+https://git.nostalgica.net/Lasting-Legacy/Lasting-Legacy-Monorepo)";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishMatrixEntry {
    pub manifest: PathBuf,
    pub variant: Option<String>,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishArtifact {
    pub platform: String,
    pub path: PathBuf,
    pub exists: bool,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishTarget {
    pub manifest_path: PathBuf,
    pub project_root: PathBuf,
    pub id: String,
    pub name: String,
    pub project_type: String,
    pub variant: Option<String>,
    pub minecraft_version: String,
    pub loader: String,
    pub version: String,
    pub release_type: String,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
    pub artifacts: Vec<PublishArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishUploadReport {
    pub dry_run: bool,
    pub target: PublishTarget,
    pub attempted: Vec<String>,
    pub uploaded: Vec<String>,
    pub skipped: Vec<String>,
}

pub fn list_publish_targets(
    manifests: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<Vec<PublishMatrixEntry>> {
    let mut output = Vec::new();
    for path in manifests {
        let path = path.as_ref();
        let manifest: Manifest = serde_json::from_slice(&fs::read(path)?)?;
        if manifest.variants.is_empty() {
            output.push(PublishMatrixEntry {
                manifest: path.to_path_buf(),
                variant: None,
                order: 0,
            });
        } else {
            for (order, variant) in manifest.variants.iter().enumerate() {
                output.push(PublishMatrixEntry {
                    manifest: path.to_path_buf(),
                    variant: Some(variant_key(variant)?.to_owned()),
                    order,
                });
            }
        }
    }
    Ok(output)
}

pub fn resolve_publish_target(
    manifest_path: impl AsRef<Path>,
    requested_variant: Option<&str>,
) -> Result<PublishTarget> {
    let manifest_path = manifest_path.as_ref();
    let project_root = manifest_path
        .parent()
        .ok_or("manifest path has no parent")?
        .to_path_buf();
    let manifest: Manifest = serde_json::from_slice(&fs::read(manifest_path)?)?;
    for (field, value) in [
        ("id", manifest.id.as_str()),
        ("name", manifest.name.as_str()),
        ("type", manifest.project_type.as_str()),
        (
            "release_type",
            manifest.release_type.as_deref().unwrap_or_default(),
        ),
    ] {
        if value.trim().is_empty() {
            return Err(format!("manifest is missing {field}").into());
        }
    }
    if manifest.modrinth_id.is_none() && manifest.curseforge_id.is_none() {
        return Err("manifest has neither modrinth_id nor curseforge_id".into());
    }
    let selected = requested_variant
        .map(|wanted| {
            manifest
                .variants
                .iter()
                .find(|variant| variant_key(variant).ok() == Some(wanted))
                .ok_or_else(|| format!("variant {wanted:?} was not found"))
        })
        .transpose()?;
    if requested_variant.is_none() && !manifest.variants.is_empty() {
        return Err("manifest has variants; select one explicitly".into());
    }
    let minecraft_version = selected
        .and_then(|variant| variant.mc_version.clone())
        .or_else(|| manifest.mc_version.clone())
        .ok_or("no Minecraft version resolved")?;
    let loader = selected
        .and_then(|variant| variant.loader.clone())
        .or_else(|| manifest.loader.clone())
        .unwrap_or_default();
    if matches!(manifest.project_type.as_str(), "mod" | "modpack") && loader.is_empty() {
        return Err("no loader resolved".into());
    }
    let base_version = selected
        .and_then(|variant| variant.version.as_deref())
        .filter(|value| !value.is_empty())
        .unwrap_or(&manifest.version);
    if base_version.is_empty() {
        return Err("manifest has no version".into());
    }
    let version = requested_variant.map_or_else(
        || base_version.to_owned(),
        |variant| format!("{base_version}-{variant}"),
    );
    let mut target = PublishTarget {
        manifest_path: manifest_path.to_path_buf(),
        project_root,
        id: manifest.id,
        name: manifest.name,
        project_type: manifest.project_type,
        variant: requested_variant.map(str::to_owned),
        minecraft_version,
        loader,
        version,
        release_type: manifest.release_type.unwrap_or_else(|| "release".into()),
        modrinth_id: manifest.modrinth_id,
        curseforge_id: manifest.curseforge_id,
        artifacts: Vec::new(),
    };
    target.artifacts = expected_artifacts(&target);
    Ok(target)
}

pub fn build_publish_target(
    manifest_path: impl AsRef<Path>,
    variant: Option<&str>,
) -> Result<PublishTarget> {
    let mut target = resolve_publish_target(manifest_path, variant)?;
    let artifacts = target.project_root.join("artifacts");
    fs::create_dir_all(&artifacts)?;
    match target.project_type.as_str() {
        "modpack" => {
            let key = target
                .variant
                .as_deref()
                .unwrap_or(&target.minecraft_version);
            for artifact in &target.artifacts {
                let format = if artifact.platform == "modrinth" {
                    ExportFormat::Modrinth
                } else {
                    ExportFormat::CurseForge
                };
                let suffix = if format == ExportFormat::Modrinth {
                    "mr"
                } else {
                    "cf"
                };
                let source = target.project_root.join(format!("{key}-{suffix}"));
                if source.is_dir() {
                    export_pack(
                        source,
                        format,
                        Some(&artifact.path),
                        ExportOptions::default(),
                    )?;
                }
            }
        }
        "datapack" | "resourcepack" => {
            let source = content_root(&target)?;
            if let Some(artifact) = target.artifacts.first() {
                crate::archive_content_directory(source, &artifact.path)?;
            }
        }
        "mod" => build_mod(&target)?,
        other => return Err(format!("unsupported publish type {other:?}").into()),
    }
    target.artifacts = expected_artifacts(&target);
    if !target.artifacts.iter().any(|artifact| artifact.exists) {
        return Err("publish build produced no artifacts".into());
    }
    Ok(target)
}

pub fn upload_publish_target(
    manifest_path: impl AsRef<Path>,
    variant: Option<&str>,
    live: bool,
    changelog_file: Option<&Path>,
) -> Result<PublishUploadReport> {
    let target = resolve_publish_target(manifest_path, variant)?;
    let changelog = changelog_file
        .map(fs::read_to_string)
        .transpose()?
        .or_else(|| fs::read_to_string(target.project_root.join("changelog.md")).ok())
        .unwrap_or_else(|| format!("Update for {}", target.name));
    let mut report = PublishUploadReport {
        dry_run: !live,
        target: target.clone(),
        attempted: Vec::new(),
        uploaded: Vec::new(),
        skipped: Vec::new(),
    };
    for artifact in target.artifacts.iter().filter(|artifact| artifact.exists) {
        report.attempted.push(artifact.platform.clone());
        if !live {
            report.uploaded.push(artifact.platform.clone());
            continue;
        }
        let project = if artifact.platform == "modrinth" {
            target.modrinth_id.as_deref()
        } else {
            target.curseforge_id.as_deref()
        }
        .ok_or("artifact has no platform project id")?;
        let already_live = if artifact.platform == "modrinth" {
            modrinth_version_exists(project, &target.version)?
        } else {
            curseforge_version_exists(project, &format!("{} {}", target.name, target.version))
        };
        if already_live {
            report.skipped.push(artifact.platform.clone());
            continue;
        }
        if artifact.platform == "modrinth" {
            upload_modrinth(&target, project, artifact, &changelog)?;
        } else {
            upload_curseforge(&target, project, artifact, &changelog)?;
        }
        report.uploaded.push(artifact.platform.clone());
    }
    if report.attempted.is_empty() {
        return Err(format!(
            "no artifacts exist in {}; run publish build first",
            target.project_root.join("artifacts").display()
        )
        .into());
    }
    Ok(report)
}

pub fn verify_publish_target(
    manifest_path: impl AsRef<Path>,
    variant: Option<&str>,
    attempts: usize,
    delay: Duration,
) -> Result<bool> {
    let target = resolve_publish_target(manifest_path, variant)?;
    let Some(project) = target.modrinth_id.as_deref() else {
        return Ok(true);
    };
    let attempts = attempts.max(1);
    for attempt in 0..attempts {
        if modrinth_version_exists(project, &target.version)? {
            return Ok(true);
        }
        if attempt + 1 < attempts {
            thread::sleep(delay);
        }
    }
    Ok(false)
}

fn expected_artifacts(target: &PublishTarget) -> Vec<PublishArtifact> {
    let directory = target.project_root.join("artifacts");
    let stem = artifact_segment(&target.name);
    let common = match target.project_type.as_str() {
        "mod" => format!("{stem}-{}.jar", target.version),
        "datapack" | "resourcepack" => format!("{}-{}.zip", target.id, target.version),
        _ => String::new(),
    };
    [
        ("modrinth", target.modrinth_id.as_ref()),
        ("curseforge", target.curseforge_id.as_ref()),
    ]
    .into_iter()
    .filter_map(|(platform, id)| id.map(|_| platform))
    .map(|platform| {
        let filename = if common.is_empty() {
            let short = if platform == "modrinth" { "mr" } else { "cf" };
            let extension = if platform == "modrinth" {
                "mrpack"
            } else {
                "zip"
            };
            format!(
                "{stem}-{}-{}-{}-{short}.{extension}",
                target.minecraft_version, target.loader, target.version
            )
        } else {
            common.clone()
        };
        let path = directory.join(filename);
        PublishArtifact {
            platform: platform.into(),
            bytes: fs::metadata(&path).map(|value| value.len()).unwrap_or(0),
            exists: path.is_file(),
            path,
        }
    })
    .collect()
}

fn content_root(target: &PublishTarget) -> Result<PathBuf> {
    for candidate in [
        target.project_root.join("content"),
        target.project_root.join(&target.minecraft_version),
        target.project_root.clone(),
    ] {
        if candidate.join("pack.mcmeta").is_file()
            || candidate.join("data").is_dir()
            || candidate.join("assets").is_dir()
        {
            return Ok(candidate);
        }
    }
    Err("no content root found".into())
}

fn build_mod(target: &PublishTarget) -> Result<()> {
    let manifest: Manifest = serde_json::from_slice(&fs::read(&target.manifest_path)?)?;
    let variant = manifest
        .variants
        .iter()
        .find(|candidate| {
            target
                .variant
                .as_deref()
                .is_some_and(|wanted| variant_key(candidate).ok() == Some(wanted))
        })
        .ok_or("mod publish requires a variant")?;
    let project = variant
        .gradle_project
        .as_deref()
        .ok_or("variant has no gradle_project")?;
    safe_segment(project)?;
    let task = format!(":{project}:build");
    let status = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", "gradlew.bat", "--no-daemon", &task])
            .current_dir(&target.project_root)
            .status()?
    } else {
        Command::new("./gradlew")
            .args(["--no-daemon", &task])
            .current_dir(&target.project_root)
            .status()?
    };
    if !status.success() {
        return Err(format!("Gradle task {task} failed with {status}").into());
    }
    let libs = target
        .project_root
        .join("versions")
        .join(project)
        .join("build/libs");
    let mut jars = fs::read_dir(&libs)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            name.ends_with(".jar")
                && ![
                    "-sources.jar",
                    "-javadoc.jar",
                    "-dev.jar",
                    "-dev-shadow.jar",
                ]
                .iter()
                .any(|suffix| name.ends_with(suffix))
        })
        .collect::<Vec<_>>();
    jars.sort();
    let [jar] = jars.as_slice() else {
        return Err(format!("expected one distributable jar in {}", libs.display()).into());
    };
    let artifact = target.artifacts.first().ok_or("no mod artifact target")?;
    fs::copy(jar, &artifact.path)?;
    Ok(())
}

fn modrinth_version_exists(project: &str, version: &str) -> Result<bool> {
    let response = ureq::get(&format!(
        "https://api.modrinth.com/v2/project/{project}/version"
    ))
    .set("User-Agent", USER_AGENT)
    .call()?;
    let values: Vec<serde_json::Value> = response_json(response)?;
    Ok(values.iter().any(|value| {
        value
            .get("version_number")
            .and_then(serde_json::Value::as_str)
            == Some(version)
    }))
}

fn upload_modrinth(
    target: &PublishTarget,
    project: &str,
    artifact: &PublishArtifact,
    changelog: &str,
) -> Result<()> {
    let token = std::env::var("MODRINTH_TOKEN").map_err(|_| "MODRINTH_TOKEN not set")?;
    let project_response: serde_json::Value = response_json(
        ureq::get(&format!("https://api.modrinth.com/v2/project/{project}"))
            .set("User-Agent", USER_AGENT)
            .call()?,
    )?;
    let project_id = project_response
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or("Modrinth project lookup returned no id")?;
    let metadata = serde_json::to_vec(&json!({
        "project_id": project_id,
        "name": format!("{} {}", target.name, target.version),
        "version_number": target.version,
        "changelog": changelog,
        "dependencies": [],
        "game_versions": [target.minecraft_version],
        "version_type": target.release_type,
        "loaders": [if target.loader.is_empty() { "minecraft" } else { &target.loader }],
        "featured": false,
        "file_parts": ["file"],
        "primary_file": "file",
    }))?;
    let (content_type, body) = multipart(vec![
        ("data", None, "application/json", metadata),
        (
            "file",
            artifact.path.file_name().and_then(|name| name.to_str()),
            "application/octet-stream",
            fs::read(&artifact.path)?,
        ),
    ]);
    post_with_retry(
        "https://api.modrinth.com/v2/version",
        &[
            ("Authorization", token.as_str()),
            ("Content-Type", content_type.as_str()),
        ],
        &body,
    )?;
    Ok(())
}

fn upload_curseforge(
    target: &PublishTarget,
    project: &str,
    artifact: &PublishArtifact,
    changelog: &str,
) -> Result<()> {
    let token = std::env::var("CURSEFORGE_TOKEN").map_err(|_| "CURSEFORGE_TOKEN not set")?;
    let version_types: Vec<CfVersionType> = cf_json(&token, "/game/version-types?cache=true")?;
    let versions: Vec<CfVersion> = cf_json(&token, "/game/versions?cache=true")?;
    let minecraft_types = version_types
        .iter()
        .filter(|kind| kind.slug.starts_with("minecraft"))
        .map(|kind| kind.id)
        .collect::<Vec<_>>();
    let loader_types = version_types
        .iter()
        .filter(|kind| kind.slug.starts_with("modloader"))
        .map(|kind| kind.id)
        .collect::<Vec<_>>();
    let game_ids = versions
        .iter()
        .filter(|version| {
            minecraft_types.contains(&version.game_version_type_id)
                && (version.name.eq_ignore_ascii_case(&target.minecraft_version)
                    || version.slug.eq_ignore_ascii_case(&target.minecraft_version))
        })
        .map(|version| version.id)
        .collect::<Vec<_>>();
    if game_ids.is_empty() {
        return Err("CurseForge game-version IDs could not be resolved".into());
    }
    let loader_ids = versions
        .iter()
        .filter(|version| {
            loader_types.contains(&version.game_version_type_id)
                && (version.name.eq_ignore_ascii_case(&target.loader)
                    || version.slug.eq_ignore_ascii_case(&target.loader))
        })
        .map(|version| version.id)
        .collect::<Vec<_>>();
    let mut variants = vec![game_ids.clone()];
    if target.project_type == "mod" && !loader_ids.is_empty() {
        let mut with_loader = game_ids.clone();
        with_loader.extend(loader_ids);
        variants.insert(0, with_loader);
    }
    let url = format!("https://minecraft.curseforge.com/api/projects/{project}/upload-file");
    let file = fs::read(&artifact.path)?;
    for (index, ids) in variants.iter().enumerate() {
        let metadata = serde_json::to_vec(&json!({
            "changelog": changelog, "changelogType": "markdown",
            "displayName": format!("{} {}", target.name, target.version),
            "gameVersions": ids, "releaseType": target.release_type,
        }))?;
        let (content_type, body) = multipart(vec![
            ("metadata", None, "application/json", metadata),
            (
                "file",
                artifact.path.file_name().and_then(|name| name.to_str()),
                "application/octet-stream",
                file.clone(),
            ),
        ]);
        match post_with_retry(
            &url,
            &[
                ("X-Api-Token", token.as_str()),
                ("Content-Type", content_type.as_str()),
            ],
            &body,
        ) {
            Ok(()) => return Ok(()),
            Err(error)
                if index + 1 < variants.len()
                    && error.to_string().contains("errorCode\\\":1009") =>
            {
                continue;
            }
            Err(error) => return Err(error),
        }
    }
    Err("CurseForge upload exhausted game-version variants".into())
}

fn curseforge_version_exists(project: &str, display_name: &str) -> bool {
    let Ok(key) = std::env::var("CURSEFORGE_API_KEY") else {
        return false;
    };
    if project.parse::<u64>().is_err() {
        return false;
    }
    let response = ureq::get(&format!(
        "https://api.curseforge.com/v1/mods/{project}/files?pageSize=50"
    ))
    .set("User-Agent", USER_AGENT)
    .set("x-api-key", &key)
    .set("Accept", "application/json")
    .call();
    let Ok(response) = response else {
        return false;
    };
    let Ok(value) = response_json::<serde_json::Value>(response) else {
        return false;
    };
    value
        .get("data")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|files| {
            files.iter().any(|file| {
                file.get("displayName").and_then(serde_json::Value::as_str) == Some(display_name)
            })
        })
}

fn post_with_retry(url: &str, headers: &[(&str, &str)], body: &[u8]) -> Result<()> {
    let mut delay = Duration::from_secs(2);
    let mut last = String::new();
    for attempt in 0..3 {
        let mut request = ureq::post(url).set("User-Agent", USER_AGENT);
        for (name, value) in headers {
            request = request.set(name, value);
        }
        match request.send_bytes(body) {
            Ok(_) => return Ok(()),
            Err(ureq::Error::Status(code, response)) => {
                let mut detail = String::new();
                let _ = response
                    .into_reader()
                    .take(1024 * 1024)
                    .read_to_string(&mut detail);
                last = format!("HTTP {code}: {detail}");
                if code != 429 && code < 500 {
                    return Err(last.into());
                }
            }
            Err(error) => last = error.to_string(),
        }
        if attempt < 2 {
            thread::sleep(delay);
            delay *= 2;
        }
    }
    Err(format!("upload failed after 3 attempts: {last}").into())
}

fn cf_json<T: for<'de> Deserialize<'de>>(token: &str, path: &str) -> Result<T> {
    response_json(
        ureq::get(&format!("https://minecraft.curseforge.com/api{path}"))
            .set("User-Agent", USER_AGENT)
            .set("X-Api-Token", token)
            .call()?,
    )
}

fn response_json<T: for<'de> Deserialize<'de>>(response: ureq::Response) -> Result<T> {
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(32 * 1024 * 1024)
        .read_to_end(&mut bytes)?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[derive(Deserialize)]
struct CfVersionType {
    id: i64,
    slug: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfVersion {
    id: i64,
    game_version_type_id: i64,
    name: String,
    slug: String,
}

fn multipart(parts: Vec<(&str, Option<&str>, &str, Vec<u8>)>) -> (String, Vec<u8>) {
    const BOUNDARY: &str = "packwand-26-2-0-boundary";
    let mut output = Vec::new();
    for (name, filename, content_type, data) in parts {
        output.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
        if let Some(filename) = filename {
            output.extend_from_slice(
                format!(
                    "Content-Disposition: form-data; name=\"{name}\"; filename=\"{}\"\r\n",
                    filename.replace(['\"', '\r', '\n'], "_")
                )
                .as_bytes(),
            );
        } else {
            output.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{name}\"\r\n").as_bytes(),
            );
        }
        output.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
        output.extend_from_slice(&data);
        output.extend_from_slice(b"\r\n");
    }
    output.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={BOUNDARY}"), output)
}

fn variant_key(variant: &Variant) -> Result<&str> {
    variant
        .id
        .as_deref()
        .or(variant.mc_version.as_deref())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "variant has neither id nor mc_version".into())
}

fn artifact_segment(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn safe_segment(value: &str) -> Result<()> {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        Ok(())
    } else {
        Err(format!("invalid build segment {value:?}").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_and_resolves_manifest_variants() {
        let root = tempfile::tempdir().unwrap();
        let manifest = root.path().join("manifest.json");
        fs::write(
            &manifest,
            br#"{"id":"demo","name":"Demo","type":"modpack","loader":"fabric","version":"1.0","release_type":"release","modrinth_id":"demo","variants":[{"id":"1.21","mc_version":"1.21"}]}"#,
        )
        .unwrap();
        let matrix = list_publish_targets([&manifest]).unwrap();
        assert_eq!(matrix[0].variant.as_deref(), Some("1.21"));
        let target = resolve_publish_target(&manifest, Some("1.21")).unwrap();
        assert_eq!(target.version, "1.0-1.21");
        assert!(
            target.artifacts[0]
                .path
                .ends_with("Demo-1.21-fabric-1.0-1.21-mr.mrpack")
        );
    }

    #[test]
    fn builds_and_dry_runs_a_publish_upload_without_network_credentials() {
        let root = tempfile::tempdir().unwrap();
        let manifest = root.path().join("manifest.json");
        fs::write(&manifest, br#"{"id":"demo-data","name":"Demo Data","type":"datapack","mc_version":"1.21.1","version":"26.7","release_type":"release","modrinth_id":"demo-data"}"#).unwrap();
        fs::write(
            root.path().join("pack.mcmeta"),
            br#"{"pack":{"pack_format":48,"description":"demo"}}"#,
        )
        .unwrap();
        fs::write(root.path().join("changelog.md"), "Dry-run fixture\n").unwrap();
        let target = build_publish_target(&manifest, None).unwrap();
        assert!(target.artifacts[0].exists);
        let report = upload_publish_target(&manifest, None, false, None).unwrap();
        assert!(report.dry_run);
        assert_eq!(report.attempted, vec!["modrinth"]);
        assert_eq!(report.uploaded, vec!["modrinth"]);
    }
}
