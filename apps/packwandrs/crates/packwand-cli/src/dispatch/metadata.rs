//! Single-pack metadata commands: listing, adding, pinning, porting,
//! rehashing, scaffolding, and archive import.

use super::*;

pub(super) fn list(args: &ArgMatches) -> Result {
    let workspace = Workspace::open(std::env::current_dir()?)?;
    let side = args.get_one::<String>("side").map(String::as_str);
    let mut entries = Vec::new();
    for item in workspace
        .index()
        .files
        .iter()
        .filter(|item| item.metafile && item.alias.is_none())
    {
        let metadata: Mod = serde_json::from_str(&fs::read_to_string(
            item.file.replace('/', std::path::MAIN_SEPARATOR_STR),
        )?)?;
        if side.is_some_and(|side| metadata.side != side && metadata.side != "both") {
            continue;
        }
        entries.push(ModListEntry {
            name: metadata.name,
            filename: metadata.filename,
            side: metadata.side,
            pin: metadata.pin,
            platforms: metadata.update.keys().cloned().collect(),
        });
    }
    entries.sort_by_key(|entry| entry.name.to_lowercase());
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for entry in entries {
            if args.get_flag("version") {
                println!("{} ({})", entry.name, entry.filename);
            } else {
                println!("{}", entry.name);
            }
        }
    }
    Ok(())
}

pub(super) fn add_workspace(args: &ArgMatches) -> Result {
    let project_input = required(args, "project")?;
    let root = std::env::current_dir()?;
    let filter = args.get_one::<String>("pack").map(String::as_str);
    let mut targets = Vec::new();
    if root.join("pack.toml").is_file() {
        targets.push(root);
    } else {
        for project in packwand_workspace::discover(&root)? {
            if filter.is_none_or(|wanted| {
                project.manifest.id == wanted
                    || project
                        .root
                        .to_string_lossy()
                        .replace('\\', "/")
                        .ends_with(wanted)
            }) {
                targets.extend(project.subdirs);
            }
        }
    }
    if targets.is_empty() {
        return Err("no matching Modrinth/CurseForge pack subdirectories found".into());
    }
    let mut added = 0usize;
    let mut failures = Vec::new();
    for target in targets {
        let name = target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let provider = if name.ends_with("-cf") {
            ProviderKind::CurseForge
        } else if name.ends_with("-mr") {
            ProviderKind::Modrinth
        } else {
            eprintln!(
                "warning: {} has no -mr/-cf suffix; using Modrinth",
                target.display()
            );
            ProviderKind::Modrinth
        };
        let mut workspace = match Workspace::open(target.clone()) {
            Ok(workspace) => workspace,
            Err(error) => {
                failures.push(format!("{}: {error}", target.display()));
                continue;
            }
        };
        let mut request = ResolveRequest::new(project_input);
        request.game_versions = workspace
            .pack()
            .versions
            .get("minecraft")
            .cloned()
            .into_iter()
            .collect();
        request.loaders = workspace
            .pack()
            .versions
            .keys()
            .filter(|key| key.as_str() != "minecraft")
            .cloned()
            .collect();
        request.channels = vec![
            ReleaseChannel::Release,
            ReleaseChannel::Beta,
            ReleaseChannel::Alpha,
        ];
        match resolve_provider(provider, &request, None)
            .and_then(|resolved| workspace.add_resolved(resolved, false).map_err(Into::into))
        {
            Ok(outcome) => {
                println!("{}: added {}", target.display(), outcome.metadata_path);
                added += 1;
            }
            Err(error) => failures.push(format!("{}: {error}", target.display())),
        }
    }
    for failure in &failures {
        eprintln!("warning: {failure}");
    }
    if added == 0 {
        Err(format!("add failed for all {} target(s)", failures.len()).into())
    } else if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "added to {added} target(s); {} target(s) failed",
            failures.len()
        )
        .into())
    }
}

#[derive(Serialize)]
struct ModListEntry {
    name: String,
    filename: String,
    side: String,
    pin: bool,
    platforms: Vec<String>,
}

pub(super) fn refresh(args: &ArgMatches) -> Result {
    let mut workspace = Workspace::open(std::env::current_dir()?)?;
    if args.get_flag("no-cache") {
        workspace.disable_cache();
    }
    let report = workspace.refresh_metadata_index()?;
    let (hits, misses) = workspace.cache_stats();
    println!(
        "refreshed index: {} added, {} updated, {} removed ({misses} read, {hits} cached)",
        report.added, report.updated, report.removed
    );
    Ok(())
}

pub(super) fn pin(args: &ArgMatches, pinned: bool) -> Result {
    let mut workspace = Workspace::open(std::env::current_dir()?)?;
    let names = strings(args, "names");
    if names.is_empty() {
        return Err("provide one or more metadata names".into());
    }
    for name in names {
        let path = metadata_path(&workspace, &name)?;
        let changed = workspace.set_pinned(&path, pinned)?;
        println!(
            "{} {path}",
            if changed {
                if pinned { "pinned" } else { "unpinned" }
            } else {
                "unchanged"
            }
        );
    }
    Ok(())
}

#[derive(Serialize)]
struct PortResult {
    mr_total: usize,
    cf_matched: usize,
    missing: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    added: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    failed: Vec<String>,
}

pub(super) fn port(args: &ArgMatches) -> Result {
    let mr_dir = absolute(required(args, "mr-subdir")?)?;
    let cf_dir = absolute(required(args, "cf-subdir")?)?;
    if !mr_dir.join("mods").is_dir() {
        return Err(format!("no mods directory in {}", mr_dir.display()).into());
    }
    let mut mr = metadata_slugs(&mr_dir.join("mods"))?;
    let cf = metadata_slugs(&cf_dir.join("mods"))?;
    mr.sort();
    let mut missing = mr
        .iter()
        .filter(|slug| !cf.contains(*slug))
        .cloned()
        .collect::<Vec<_>>();
    let original_missing = missing.clone();
    let mut added = Vec::new();
    let mut failed = Vec::new();
    if args.get_flag("add") {
        let mut workspace = Workspace::open(cf_dir)?;
        let transport = UreqTransport::new();
        let client = CurseForgeClient::new(transport, configured_api_key());
        for slug in &original_missing {
            let mut request = ResolveRequest::new(slug.clone());
            request.game_versions = workspace
                .pack()
                .versions
                .get("minecraft")
                .cloned()
                .into_iter()
                .collect();
            request.loaders = workspace
                .pack()
                .versions
                .keys()
                .filter(|loader| loader.as_str() != "minecraft")
                .cloned()
                .collect();
            let outcome = match client.resolve(&request) {
                Ok(resolved) => workspace
                    .add_resolved(resolved, false)
                    .map(|_| ())
                    .map_err(|error| error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            if let Err(error) = outcome {
                eprintln!("warning: could not port {slug}: {error}");
                failed.push(slug.clone());
            } else {
                added.push(slug.clone());
            }
        }
        missing = failed.clone();
    }
    let result = PortResult {
        mr_total: mr.len(),
        cf_matched: mr.len() - original_missing.len(),
        missing,
        added,
        failed,
    };
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "MR mods: {} | already on CF: {} | missing on CF: {}",
            result.mr_total,
            result.cf_matched,
            original_missing.len()
        );
        for slug in &result.missing {
            println!("  - {slug}");
        }
        if args.get_flag("add") {
            println!(
                "ported {}; {} failed",
                result.added.len(),
                result.failed.len()
            );
        }
    }
    if args.get_flag("add") && !result.failed.is_empty() {
        Err(format!("{} mod(s) could not be ported", result.failed.len()).into())
    } else {
        Ok(())
    }
}

fn metadata_slugs(directory: &Path) -> Result<Vec<String>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let mut slugs = Vec::new();
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(slug) = name.strip_suffix(".pw.json") {
                slugs.push(slug.into());
            }
        }
    }
    Ok(slugs)
}

pub(super) fn remove(args: &ArgMatches) -> Result {
    let mut workspace = Workspace::open(std::env::current_dir()?)?;
    let names = strings(args, "names");
    if names.is_empty() {
        return Err("provide one or more metadata names".into());
    }
    for name in names {
        let path = metadata_path(&workspace, &name)?;
        workspace.remove_metadata(&path)?;
        println!("removed {path}");
    }
    Ok(())
}

pub(super) fn rehash(args: &ArgMatches) -> Result {
    let format = required(args, "format")?.parse::<packwand_pack::HashFormat>()?;
    if !matches!(
        format,
        packwand_pack::HashFormat::Sha1
            | packwand_pack::HashFormat::Sha256
            | packwand_pack::HashFormat::Sha512
    ) {
        return Err("rehash format must be sha1, sha256, or sha512".into());
    }
    let report = Workspace::open(std::env::current_dir()?)?.rehash(format)?;
    println!(
        "rehashed {} indexed file(s), including {} metadata file(s) and {} download(s), as {}",
        report.indexed_files,
        report.metadata_files,
        report.downloads,
        format.as_str()
    );
    Ok(())
}

pub(super) fn metadata_path(workspace: &Workspace, name: &str) -> Result<String> {
    let normalized = name.trim_end_matches(".pw.json");
    let matches = workspace
        .index()
        .files
        .iter()
        .filter(|item| item.metafile && item.alias.is_none())
        .filter(|item| {
            item.file == name
                || item.file.trim_end_matches(".pw.json") == normalized
                || Path::new(&item.file)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    == Some(normalized)
        })
        .map(|item| item.file.clone())
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [path] => Ok(path.clone()),
        [] => Err(format!("metadata {name:?} was not found").into()),
        _ => Err(format!("metadata name {name:?} is ambiguous").into()),
    }
}

pub(super) fn new_project(args: &ArgMatches) -> Result {
    let category = required(args, "category")?.to_owned();
    let id = required(args, "name")?.to_owned();
    let role = if args.get_flag("base") {
        ProjectRole::Base
    } else if let Some(base) = args.get_one::<String>("consumes") {
        ProjectRole::Consumes(base.clone())
    } else {
        ProjectRole::None
    };
    let request = NewProject {
        category,
        id,
        name: None,
        minecraft_version: args.get_one::<String>("mc").cloned(),
        loader: args.get_one::<String>("loader").cloned(),
        variants: comma_values(args, "variants"),
        role,
    };
    let project = packwand_workspace::create_project(std::env::current_dir()?, &request)?;
    println!("scaffolded {}", project.root.display());
    Ok(())
}

pub(super) fn init_pack(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let loader = args
        .get_one::<String>("modloader")
        .cloned()
        .unwrap_or_else(|| "fabric".into());
    let version_key = format!("{loader}-version");
    let loader_version = args
        .get_one::<String>(&version_key)
        .cloned()
        .unwrap_or_else(|| "latest".into());
    let request = packwand_workspace::InitPack {
        name: args
            .get_one::<String>("name")
            .cloned()
            .or_else(|| {
                root.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| "Pack".into()),
        author: args
            .get_one::<String>("author")
            .cloned()
            .unwrap_or_else(|| "CHANGEME".into()),
        version: args
            .get_one::<String>("version")
            .cloned()
            .unwrap_or_else(|| "1.0.0".into()),
        minecraft_version: args
            .get_one::<String>("mc-version")
            .cloned()
            .unwrap_or_else(|| "26.1.2".into()),
        loader,
        loader_version,
    };
    if args.get_flag("reinit") && root.join("pack.toml").is_file() {
        fs::remove_file(root.join("pack.toml"))?;
    }
    packwand_workspace::init_pack(&root, &request)?;
    println!("initialised {}", root.display());
    Ok(())
}

pub(super) fn import_archive(args: &ArgMatches) -> Result {
    let source = required(args, "archive")?;
    let mut downloaded = None;
    let archive =
        if url::Url::parse(source).is_ok_and(|url| matches!(url.scheme(), "http" | "https")) {
            let mut file = tempfile::NamedTempFile::new()?;
            let response = ureq::get(source)
                .set("User-Agent", "packwand/26.2.0")
                .call()
                .map_err(|error| format!("could not download {source}: {error}"))?;
            const MAX_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
            let mut reader = response.into_reader().take(MAX_ARCHIVE_BYTES + 1);
            let copied = std::io::copy(&mut reader, file.as_file_mut())?;
            if copied > MAX_ARCHIVE_BYTES {
                return Err("downloaded archive exceeds 2 GiB".into());
            }
            let path = file.path().to_path_buf();
            downloaded = Some(file);
            path
        } else {
            absolute(source)?
        };
    let id = args.get_one::<String>("id").cloned().unwrap_or_else(|| {
        archive
            .file_stem()
            .and_then(|name| name.to_str())
            .map(slugify)
            .unwrap_or_else(|| "imported-pack".into())
    });
    if id.is_empty() || id.contains(['/', '\\']) || id == "." || id == ".." {
        return Err(format!("invalid imported pack id {id:?}").into());
    }
    let workspace = std::env::current_dir()?;
    let project_root = workspace.join("modpacks").join(&id);
    if project_root.exists() {
        return Err(format!(
            "project {id:?} already exists at {}",
            project_root.display()
        )
        .into());
    }
    fs::create_dir_all(&project_root)?;
    let modrinth_archive = source
        .split(['?', '#'])
        .next()
        .is_some_and(|path| path.to_ascii_lowercase().ends_with(".mrpack"));
    let platform = if modrinth_archive { "mr" } else { "cf" };
    let provisional = project_root.join(format!("imported-{platform}"));
    let imported = match if modrinth_archive {
        import_modrinth_archive(&archive, &provisional)
    } else {
        let client = CurseForgeClient::new(UreqTransport::new(), configured_api_key());
        import_curseforge_archive(&archive, &provisional, |project_id, file_id| {
            let mut request = ResolveRequest::new(project_id.to_string());
            request.version_id = Some(file_id.to_string());
            let resolved = client
                .resolve(&request)
                .map_err(|error| error.to_string())?;
            let path = resolved.metadata_path();
            let metadata = resolved.into_mod().map_err(|error| error.to_string())?;
            Ok((path, metadata))
        })
    } {
        Ok(imported) => imported,
        Err(error) => {
            let _ = fs::remove_dir_all(&project_root);
            return Err(error.into());
        }
    };
    drop(downloaded);
    let variant_id = imported
        .minecraft_version
        .clone()
        .unwrap_or_else(|| "imported".into());
    let safe_variant = variant_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    let subdir = project_root.join(format!("{safe_variant}-{platform}"));
    fs::rename(&provisional, &subdir)?;
    let manifest = Manifest {
        schema: Some("../../tools/manifest/schema.json".into()),
        id: id.clone(),
        name: imported.name.clone(),
        project_type: "modpack".into(),
        loader: imported.loader.clone(),
        mc_version: imported.minecraft_version.clone(),
        variants: Vec::new(),
        version: imported.version.clone(),
        modrinth_id: (platform == "mr").then(|| id.clone()),
        curseforge_id: (platform == "cf").then(|| id.clone()),
        role: Some(serde_json::Value::String("none".into())),
        ..Manifest::default()
    };
    if let Err(error) =
        packwand_workspace::write_manifest(&project_root, &manifest).and_then(|_| {
            fs::write(
                project_root.join("changelog.md"),
                format!("# {}\n\nImported from {}.\n", imported.name, source),
            )
            .map_err(packwand_workspace::Error::from)
        })
    {
        let _ = fs::remove_dir_all(&project_root);
        return Err(error.into());
    }
    println!(
        "imported {} {} as {} ({} metadata file(s), {} indexed file(s))",
        imported.name,
        imported.version,
        subdir.display(),
        imported.metadata_files,
        imported.files
    );
    Ok(())
}
