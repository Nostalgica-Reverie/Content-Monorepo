//! Single-mod inspection (`explain`) and Modrinth dependency coverage
//! (`deps`, aliased `graph`).

use super::*;

#[derive(Debug, Serialize)]
struct ExplainResult {
    slug: String,
    name: String,
    path: String,
    filename: String,
    side: String,
    pinned: bool,
    hash_format: String,
    hash: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    download_url: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    download_mode: String,
    providers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modrinth_project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modrinth_version_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    curseforge_project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    curseforge_file_id: Option<String>,
}

pub(super) fn explain(args: &ArgMatches) -> Result {
    let root = args
        .get_one::<String>("pack-dir")
        .map(absolute)
        .transpose()?
        .unwrap_or(std::env::current_dir()?);
    let slug = required(args, "mod-slug")?;
    let workspace = Workspace::open(root.clone())?;
    let path = mod_metadata_path(&workspace, slug)?;
    let metadata: Mod = serde_json::from_str(&fs::read_to_string(
        root.join(path.replace('/', std::path::MAIN_SEPARATOR_STR)),
    )?)?;
    let modrinth = metadata.update.get("modrinth");
    let curseforge = metadata.update.get("curseforge");
    let result = ExplainResult {
        slug: slug.to_owned(),
        name: metadata.name,
        path,
        filename: metadata.filename,
        side: if metadata.side.is_empty() {
            "both".into()
        } else {
            metadata.side
        },
        pinned: metadata.pin,
        hash_format: metadata.download.hash_format,
        hash: metadata.download.hash,
        download_url: metadata.download.url,
        download_mode: metadata.download.mode,
        providers: metadata.update.keys().cloned().collect(),
        modrinth_project_id: table_string(modrinth, "mod-id"),
        modrinth_version_id: table_string(modrinth, "version"),
        curseforge_project_id: table_string(curseforge, "project-id"),
        curseforge_file_id: table_string(curseforge, "file-id"),
    };
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{} ({})", result.name, result.slug);
        println!("  path:     {}", result.path);
        println!("  file:     {}", result.filename);
        println!("  side:     {}", result.side);
        println!("  pinned:   {}", result.pinned);
        println!("  hash:     {} {}", result.hash_format, result.hash);
        if !result.download_url.is_empty() {
            println!("  url:      {}", result.download_url);
        }
        if result.providers.is_empty() {
            println!("  provider: none (manually managed)");
        } else {
            println!("  provider: {}", result.providers.join(", "));
        }
        if let Some(id) = &result.modrinth_project_id {
            println!(
                "  modrinth: project {id}, version {}",
                result.modrinth_version_id.as_deref().unwrap_or("?")
            );
        }
        if let Some(id) = &result.curseforge_project_id {
            println!(
                "  curseforge: project {id}, file {}",
                result.curseforge_file_id.as_deref().unwrap_or("?")
            );
        }
    }
    Ok(())
}

/// Looks up an installed mod's metadata path by slug, matching on the
/// `.pw.json` basename regardless of folder (mods/resourcepacks/shaderpacks),
/// the same way `side` locates a mod across a subdir's platform folders.
fn mod_metadata_path(workspace: &Workspace, slug: &str) -> Result<String> {
    let target = format!("{}.pw.json", slug.trim_end_matches(".pw.json"));
    workspace
        .index()
        .files
        .iter()
        .filter(|item| item.metafile && item.alias.is_none())
        .find(|item| {
            Path::new(&item.file)
                .file_name()
                .and_then(|name| name.to_str())
                == Some(target.as_str())
        })
        .map(|item| item.file.clone())
        .ok_or_else(|| format!("mod {slug:?} was not found in the pack index").into())
}

fn table_string(table: Option<&packwand_pack::UpdateTable>, key: &str) -> Option<String> {
    let value = table?.get(key)?;
    Some(match value.as_str() {
        Some(text) => text.to_owned(),
        None => value.to_string(),
    })
}

#[derive(Debug, Serialize)]
struct DependencyRequirement {
    project_id: String,
    satisfied: bool,
}

#[derive(Debug, Serialize)]
struct DependencyMod {
    slug: String,
    name: String,
    requires: Vec<DependencyRequirement>,
}

#[derive(Debug, Serialize)]
struct DependencyReport {
    dir: String,
    checked: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    skipped: Vec<String>,
    mods: Vec<DependencyMod>,
    missing: usize,
}

struct InstalledModrinthMod {
    slug: String,
    name: String,
    project_id: String,
    version_id: String,
}

pub(super) fn deps(args: &ArgMatches) -> Result {
    let root = args
        .get_one::<String>("pack-dir")
        .map(absolute)
        .transpose()?
        .unwrap_or(std::env::current_dir()?);
    let workspace = Workspace::open(root.clone())?;
    let mut installed = Vec::new();
    let mut skipped = Vec::new();
    for item in workspace
        .index()
        .files
        .iter()
        .filter(|item| item.metafile && item.alias.is_none())
    {
        let slug = Path::new(&item.file)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(&item.file)
            .trim_end_matches(".pw")
            .to_owned();
        let metadata: Mod = serde_json::from_str(&fs::read_to_string(
            root.join(item.file.replace('/', std::path::MAIN_SEPARATOR_STR)),
        )?)?;
        let modrinth = metadata.update.get("modrinth");
        match (
            table_string(modrinth, "mod-id"),
            table_string(modrinth, "version"),
        ) {
            (Some(project_id), Some(version_id)) => installed.push(InstalledModrinthMod {
                slug,
                name: metadata.name,
                project_id,
                version_id,
            }),
            _ => skipped.push(slug),
        }
    }
    let dir = root.to_string_lossy().replace('\\', "/");
    if installed.is_empty() {
        let report = DependencyReport {
            dir,
            checked: 0,
            skipped,
            mods: Vec::new(),
            missing: 0,
        };
        if args.get_flag("json") {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("no Modrinth-provenanced mods in {}", report.dir);
        }
        return Ok(());
    }
    let version_ids = installed
        .iter()
        .map(|installed| installed.version_id.clone())
        .collect::<Vec<_>>();
    let client = ModrinthClient::new(UreqTransport::new());
    let required_by_version = client.dependencies_by_version(&version_ids)?;
    let installed_ids = installed
        .iter()
        .map(|installed| installed.project_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut mods = Vec::new();
    let mut missing = 0usize;
    for installed in &installed {
        let Some(required) = required_by_version.get(&installed.version_id) else {
            continue;
        };
        if required.is_empty() {
            continue;
        }
        let requires = required
            .iter()
            .map(|project_id| {
                let satisfied = installed_ids.contains(project_id.as_str());
                if !satisfied {
                    missing += 1;
                }
                DependencyRequirement {
                    project_id: project_id.clone(),
                    satisfied,
                }
            })
            .collect();
        mods.push(DependencyMod {
            slug: installed.slug.clone(),
            name: installed.name.clone(),
            requires,
        });
    }
    mods.sort_by_key(|entry| entry.name.to_lowercase());
    let report = DependencyReport {
        dir,
        checked: installed.len(),
        skipped,
        mods,
        missing,
    };
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for entry in &report.mods {
            for requirement in &entry.requires {
                println!(
                    "{} requires {} [{}]",
                    entry.name,
                    requirement.project_id,
                    if requirement.satisfied {
                        "ok"
                    } else {
                        "MISSING"
                    }
                );
            }
        }
        println!(
            "checked {} Modrinth mod(s), {} skipped, {} missing required dependenc{}",
            report.checked,
            report.skipped.len(),
            report.missing,
            if report.missing == 1 { "y" } else { "ies" }
        );
    }
    if report.missing == 0 {
        Ok(())
    } else {
        Err(format!(
            "{} required dependency(ies) missing from the pack",
            report.missing
        )
        .into())
    }
}
