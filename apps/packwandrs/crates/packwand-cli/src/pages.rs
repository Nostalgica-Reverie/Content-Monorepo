use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::ArgMatches;
use packwand_pack::Mod;
use packwand_workspace::{Manifest, Project};
use serde::Serialize;
use serde_json::{Map, Value, json};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Serialize)]
struct SubdirResult {
    subdir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mod_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct PagesResult {
    subdirs: Vec<SubdirResult>,
    written: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    projects_index_count: Option<usize>,
}

pub fn run(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let selected = args
        .get_one::<String>("pack")
        .or_else(|| args.get_one::<String>("pack-dir"))
        .map(|value| absolute(&root, value));
    let projects = if let Some(project_root) = selected {
        vec![packwand_workspace::read_project(&root, &project_root)?]
    } else {
        packwand_workspace::discover(&root)?
    };

    let mut results = Vec::new();
    let mut written = 0usize;
    for project in &projects {
        for subdir in &project.subdirs {
            if !subdir.join("mods").is_dir() {
                continue;
            }
            match write_modlist(subdir) {
                Ok(count) => {
                    written += 1;
                    results.push(SubdirResult {
                        subdir: display_relative(&root, subdir),
                        mod_count: Some(count),
                        error: None,
                    });
                    if !args.get_flag("json") {
                        println!(
                            "wrote {}/modlist.md ({count} mods)",
                            display_relative(&root, subdir)
                        );
                    }
                }
                Err(error) => results.push(SubdirResult {
                    subdir: display_relative(&root, subdir),
                    mod_count: None,
                    error: Some(error.to_string()),
                }),
            }
        }
    }
    if results.is_empty() {
        return Err("no modpack subdirectories with a mods directory were found".into());
    }

    let projects_index_count = if args.get_one::<String>("pack").is_none()
        && args.get_one::<String>("pack-dir").is_none()
    {
        Some(write_project_indexes(
            &root,
            &packwand_workspace::discover(&root)?,
        )?)
    } else {
        None
    };
    let report = PagesResult {
        subdirs: results,
        written,
        projects_index_count,
    };
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("generated {written} modlist.md file(s)");
        if let Some(count) = projects_index_count {
            println!("generated project indexes for {count} project(s)");
        }
    }
    if report.subdirs.iter().any(|result| result.error.is_some()) {
        Err("one or more mod lists could not be generated".into())
    } else {
        Ok(())
    }
}

fn write_modlist(root: &Path) -> Result<usize> {
    let mut sections = [Vec::new(), Vec::new(), Vec::new()];
    let mut count = 0usize;
    for entry in fs::read_dir(root.join("mods"))? {
        let entry = entry?;
        if !entry.file_type()?.is_file() || !packwand_pack::metafile::is_metafile(entry.path()) {
            continue;
        }
        let metadata: Mod = match toml::from_str(&fs::read_to_string(entry.path())?) {
            Ok(metadata) => metadata,
            Err(error) => {
                eprintln!("warning: {}: {error}", entry.path().display());
                continue;
            }
        };
        let url = metadata
            .update
            .get("modrinth")
            .and_then(|table| table.get("mod-id"))
            .and_then(serde_json::Value::as_str)
            .map(|id| format!("https://modrinth.com/mod/{id}"))
            .unwrap_or_else(|| metadata.download.url.clone());
        let line = format!("- [{}]({url})", metadata.name);
        let index = match metadata.side.as_str() {
            "client" => 0,
            "server" => 2,
            _ => 1,
        };
        sections[index].push(line);
        count += 1;
    }
    let mut markdown = String::from("# Modlist\n");
    for (title, lines) in ["Client Mods", "Shared Mods", "Server Mods"]
        .into_iter()
        .zip(&mut sections)
    {
        if lines.is_empty() {
            continue;
        }
        lines.sort();
        markdown.push_str(&format!("\n## {title}\n\n"));
        for line in lines {
            markdown.push_str(line);
            markdown.push('\n');
        }
    }
    atomic_write(&root.join("modlist.md"), markdown.as_bytes())?;
    Ok(count)
}

fn write_project_indexes(root: &Path, projects: &[Project]) -> Result<usize> {
    let generated = rfc3339_now();
    let entries = projects
        .iter()
        .map(|project| project_entry(root, project))
        .collect::<Result<Vec<_>>>()?;
    let main = json!({"generated": generated, "projects": entries});
    let output = std::env::var_os("PROJECTS_INDEX_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("docs/docs/public/projects.json"));
    write_json(&output, &main)?;

    for category in ["modpacks", "resourcepacks", "datapacks"] {
        if !root.join(category).is_dir() {
            continue;
        }
        let category_projects = entries
            .iter()
            .filter(|entry| {
                entry
                    .get("dir")
                    .and_then(Value::as_str)
                    .is_some_and(|dir| dir.starts_with(&format!("{category}/")))
            })
            .cloned()
            .collect::<Vec<_>>();
        let value = json!({
            "_generated": "Used by Packwand do not touch pls thx",
            "generated": generated,
            "projects": category_projects,
        });
        write_json(&root.join(category).join("Project.json"), &value)?;
    }
    Ok(entries.len())
}

fn project_entry(root: &Path, project: &Project) -> Result<Value> {
    let manifest = &project.manifest;
    let mut entry = Map::new();
    insert(&mut entry, "id", manifest.id.clone());
    insert(&mut entry, "name", manifest.effective_name().to_owned());
    insert(&mut entry, "type", manifest.project_type.clone());
    insert(
        &mut entry,
        "category",
        project.category.trim_end_matches('s').to_owned(),
    );
    insert(&mut entry, "dir", display_relative(root, &project.root));
    insert(
        &mut entry,
        "manifest_path",
        display_relative(root, &project.root.join("manifest.json")),
    );
    optional(&mut entry, "loader", manifest.loader.clone());
    optional(&mut entry, "mc_version", manifest.mc_version.clone());
    insert(&mut entry, "version", manifest.version.clone());
    optional(&mut entry, "release_type", manifest.release_type.clone());
    optional(&mut entry, "description", manifest.description.clone());
    insert(&mut entry, "lifecycle", manifest.lifecycle().to_owned());
    let (role, performance_base) = role_fields(manifest);
    insert(&mut entry, "role", role);
    optional(&mut entry, "performance_base", performance_base);
    optional(&mut entry, "shared_assets", manifest.shared_assets.clone());
    entry.insert(
        "auto_update".into(),
        Value::Bool(manifest.automation().auto_update.unwrap_or(true)),
    );
    for (key, value) in [
        ("modrinth_id", manifest.modrinth_id.clone()),
        ("curseforge_id", manifest.curseforge_id.clone()),
        ("github_id", manifest.github_id.clone()),
        ("gitea_id", manifest.gitea_id.clone()),
        ("gitlab_id", manifest.gitlab_id.clone()),
    ] {
        optional(&mut entry, key, value);
    }
    let mut platforms = Map::new();
    for (key, value) in [
        ("modrinth", manifest.modrinth_id.clone()),
        ("curseforge", manifest.curseforge_id.clone()),
        ("github", manifest.github_id.clone()),
        ("gitea", manifest.gitea_id.clone()),
        ("gitlab", manifest.gitlab_id.clone()),
    ] {
        optional(&mut platforms, key, value);
    }
    entry.insert("platforms".into(), Value::Object(platforms));
    if let Some(path) = docs_path(&manifest.project_type, &manifest.id) {
        insert(&mut entry, "docs_path", path);
    }
    if !manifest.variants.is_empty() {
        entry.insert(
            "variants".into(),
            serde_json::to_value(
                manifest
                    .variants
                    .iter()
                    .map(|variant| {
                        json!({
                            "id": variant.id,
                            "mc_version": variant.mc_version,
                            "loader": variant.loader.as_ref().or(manifest.loader.as_ref()),
                            "version": variant.version,
                        })
                    })
                    .collect::<Vec<_>>(),
            )?,
        );
    }
    let subdirs = project
        .subdirs
        .iter()
        .map(|subdir| subdir_entry(root, subdir))
        .collect::<Result<Vec<_>>>()?;
    entry.insert("subdirs".into(), Value::Array(subdirs));
    Ok(Value::Object(entry))
}

fn subdir_entry(root: &Path, subdir: &Path) -> Result<Value> {
    let key = subdir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("pack subdirectory has no valid name")?;
    let mod_count = match fs::read_dir(subdir.join("mods")) {
        Ok(entries) => entries
            .flatten()
            .filter(|entry| packwand_pack::metafile::is_metafile(entry.path()))
            .count(),
        Err(_) => 0,
    };
    Ok(json!({
        "key": key,
        "path": display_relative(root, subdir),
        "platform": if key.ends_with("-cf") { "curseforge" } else if key.ends_with("-mr") { "modrinth" } else { "" },
        "mod_count": mod_count,
        "has_index": subdir.join("index.json").is_file(),
        "has_pack": subdir.join("pack.toml").is_file(),
    }))
}

fn role_fields(manifest: &Manifest) -> (String, Option<String>) {
    match manifest.role.as_ref() {
        Some(Value::String(role)) => (role.clone(), None),
        Some(Value::Object(role)) => {
            let label = role
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("consumer")
                .to_owned();
            let base = role
                .get("pack")
                .or_else(|| role.get("base"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            (label, base)
        }
        _ => ("none".into(), None),
    }
}

fn docs_path(kind: &str, id: &str) -> Option<String> {
    match kind {
        "modpack" => Some(format!("/modpacks/{id}/")),
        "datapack" => Some(format!("/datapacks/{id}/")),
        "resourcepack" => Some(format!("/resource-packs/{id}/")),
        _ => None,
    }
}

fn insert(map: &mut Map<String, Value>, key: &str, value: String) {
    map.insert(key.into(), Value::String(value));
}

fn optional(map: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        insert(map, key, value);
    }
}

fn display_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn absolute(root: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn write_json(path: &Path, value: &Value) -> Result {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    atomic_write(path, &bytes)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result {
    let parent = path.parent().ok_or("output path has no parent")?;
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    use std::io::Write as _;
    temporary.write_all(bytes)?;
    temporary.as_file_mut().sync_all()?;
    temporary.persist(path)?;
    Ok(())
}

fn rfc3339_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days = seconds.div_euclid(86_400);
    let day_seconds = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / 3_600,
        day_seconds % 3_600 / 60,
        day_seconds % 60
    )
}

// Howard Hinnant's proleptic-Gregorian civil-from-days algorithm.
fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::{civil_from_days, write_modlist};
    use std::fs;

    #[test]
    fn civil_epoch_is_correct() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_454), (2026, 1, 1));
    }

    #[test]
    fn modlist_groups_sides_and_prefers_modrinth_page() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("mods")).unwrap();
        fs::write(
            root.path().join("mods/example.pw.json"),
            r#"name = "Example"
filename = "example.jar"
side = "client"

[download]
url = "https://cdn.example.test/example.jar"
hash-format = "sha256"
hash = "00"

[update.modrinth]
mod-id = "example-id"
version = "one"
"#,
        )
        .unwrap();
        assert_eq!(write_modlist(root.path()).unwrap(), 1);
        let output = fs::read_to_string(root.path().join("modlist.md")).unwrap();
        assert!(output.contains("## Client Mods"));
        assert!(output.contains("https://modrinth.com/mod/example-id"));
    }
}
