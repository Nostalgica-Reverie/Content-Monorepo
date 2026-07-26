//! Multi-pack batch commands and the manifest field editor.

use super::*;

pub(super) fn packs(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let json = args.get_flag("json");
    match args.subcommand() {
        Some(("list", _)) => {
            let projects = packwand_workspace::discover(root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&projects)?);
            } else {
                for project in projects {
                    println!(
                        "{}\t{}\t{}",
                        project.manifest.id,
                        project.manifest.version,
                        project.root.display()
                    );
                }
            }
            Ok(())
        }
        Some(("get", sub)) => {
            let project = packwand_workspace::find(root, required(sub, "id")?)?;
            let value = serde_json::to_value(project.manifest)?;
            if let Some(field) = sub.get_one::<String>("field") {
                let value = value
                    .get(field)
                    .ok_or_else(|| format!("unknown manifest field {field:?}"))?;
                print_value(value, json);
            } else {
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
            Ok(())
        }
        Some(("set", sub)) => set_manifest_field(
            &root,
            required(sub, "id")?,
            required(sub, "field")?,
            required(sub, "value")?,
        ),
        Some(("index", _)) => {
            let projects = packwand_workspace::discover(root)?;
            println!("{}", serde_json::to_string_pretty(&projects)?);
            Ok(())
        }
        _ => Err("packs requires list, get, set, or index".into()),
    }
}

fn set_manifest_field(root: &Path, id: &str, field: &str, raw: &str) -> Result {
    let project = packwand_workspace::find(root, id)?;
    let mut value = serde_json::to_value(project.manifest)?;
    let object = value.as_object_mut().ok_or("manifest is not an object")?;
    const MANIFEST_FIELDS: &[&str] = &[
        "$schema",
        "id",
        "name",
        "type",
        "loader",
        "mc_version",
        "variants",
        "version",
        "release_type",
        "description",
        "modrinth_id",
        "curseforge_id",
        "github_id",
        "gitea_id",
        "gitlab_id",
        "role",
        "shared_assets",
        "lifecycle",
        "automation",
    ];
    let mut segments = field.split('.');
    let top = segments.next().ok_or("manifest field is empty")?;
    if !object.contains_key(top) && !MANIFEST_FIELDS.contains(&top) {
        return Err(format!("unknown manifest field {field:?}").into());
    }
    let parsed =
        serde_json::from_str(raw).unwrap_or_else(|_| serde_json::Value::String(raw.to_owned()));
    let remaining = segments.collect::<Vec<_>>();
    if remaining.is_empty() {
        object.insert(top.to_owned(), parsed);
    } else {
        let mut cursor = object
            .entry(top.to_owned())
            .or_insert_with(|| serde_json::json!({}));
        for segment in &remaining[..remaining.len() - 1] {
            let object = cursor
                .as_object_mut()
                .ok_or_else(|| format!("{segment:?} is not an object"))?;
            cursor = object
                .entry((*segment).to_owned())
                .or_insert_with(|| serde_json::json!({}));
        }
        let leaf = remaining.last().expect("non-empty nested field");
        cursor
            .as_object_mut()
            .ok_or_else(|| format!("{top:?} is not an object"))?
            .insert((*leaf).to_owned(), parsed);
    }
    let manifest = serde_json::from_value(value)?;
    packwand_workspace::write_manifest(project.root, &manifest)?;
    println!("set {id}.{field}");
    Ok(())
}

pub(super) fn batch_command(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    match args.subcommand() {
        Some(("status", sub)) => {
            let mut statuses = Vec::new();
            for project in batch_projects(&root)?
                .into_iter()
                .filter(|project| project.category == "modpacks")
            {
                let mut subdirs = Vec::new();
                let mut total = 0;
                for path in &project.subdirs {
                    let count = Workspace::open(path)
                        .map(|workspace| {
                            workspace
                                .index()
                                .files
                                .iter()
                                .filter(|file| file.metafile && file.alias.is_none())
                                .count()
                        })
                        .unwrap_or(0);
                    total += count;
                    subdirs.push(BatchSubdir {
                        key: path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                        mod_count: count,
                    });
                }
                let lifecycle = project.manifest.lifecycle().to_owned();
                statuses.push(BatchStatus {
                    id: project.manifest.id,
                    name: project.manifest.name,
                    version: project.manifest.version,
                    lifecycle,
                    total_mods: total,
                    subdirs,
                });
            }
            if sub.get_flag("json") {
                println!("{}", serde_json::to_string_pretty(&statuses)?);
            } else {
                for status in statuses {
                    println!(
                        "{} {}: {} subdirs, {} mods",
                        status.id,
                        status.version,
                        status.subdirs.len(),
                        status.total_mods
                    );
                }
            }
            Ok(())
        }
        Some(("refresh", sub)) => {
            let targets =
                batch_targets(&root, sub.get_one::<String>("pack-dir").map(String::as_str))?;
            if sub.get_flag("dry-run") {
                for target in targets {
                    println!("{}", target.display());
                }
                return Ok(());
            }
            for target in targets {
                let report = Workspace::open(&target)?.refresh_metadata_index()?;
                println!(
                    "{}: +{} ~{} -{}",
                    target.display(),
                    report.added,
                    report.updated,
                    report.removed
                );
            }
            Ok(())
        }
        Some(("export", sub)) => {
            let targets =
                batch_targets(&root, sub.get_one::<String>("pack-dir").map(String::as_str))?;
            let mut built = 0usize;
            for target in targets {
                let name = target
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default();
                let format = if name.ends_with("-cf") {
                    ExportFormat::CurseForge
                } else if name.ends_with("-mr") {
                    ExportFormat::Modrinth
                } else {
                    continue;
                };
                let artifact =
                    export_pack(&target, format, None::<&Path>, ExportOptions::default())?;
                println!("exported {}", artifact.path.display());
                built += 1;
            }
            if built == 0 {
                Err("no batch export targets matched".into())
            } else {
                Ok(())
            }
        }
        Some(("mr", provider_args)) => {
            batch_provider_add(&root, ProviderKind::Modrinth, provider_args)
        }
        Some(("cf", provider_args)) => {
            batch_provider_add(&root, ProviderKind::CurseForge, provider_args)
        }
        Some(("update", sub)) => {
            let targets =
                batch_targets(&root, sub.get_one::<String>("pack-dir").map(String::as_str))?;
            let dry_run = sub.get_flag("check");
            let mut results = Vec::new();
            let mut failures = 0usize;
            for target in targets {
                match update_workspace(&target, None, true, dry_run) {
                    Ok(records) => {
                        failures += records.iter().filter(|record| is_failure(record)).count();
                        if !sub.get_flag("json") {
                            println!("{}:", target.display());
                            print_update_records(&records);
                        }
                        results.push(serde_json::json!({"dir":target,"updates":records}));
                    }
                    Err(error) => {
                        failures += 1;
                        results.push(
                            serde_json::json!({"dir":target,"updates":[],"error":error.to_string()}),
                        );
                    }
                }
            }
            let bytes = serde_json::to_vec_pretty(&serde_json::json!({
                "dry_run": dry_run,
                "failed_checks": failures,
                "subdirs": results,
            }))?;
            if sub.get_flag("json") {
                println!("{}", String::from_utf8_lossy(&bytes));
            }
            if let Some(path) = sub.get_one::<String>("report") {
                let path = absolute(path)?;
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(path, bytes)?;
            }
            if failures == 0 {
                Ok(())
            } else {
                Err(format!("{failures} batch update(s) failed").into())
            }
        }
        Some(("loader-update", sub)) => {
            let requested = sub
                .get_one::<String>("version")
                .map(String::as_str)
                .unwrap_or("latest");
            let targets =
                batch_targets(&root, sub.get_one::<String>("pack-dir").map(String::as_str))?;
            for target in targets {
                let mut workspace = Workspace::open(&target)?;
                let minecraft = workspace
                    .pack()
                    .versions
                    .get("minecraft")
                    .cloned()
                    .ok_or("pack has no Minecraft version")?;
                let loaders = workspace
                    .pack()
                    .versions
                    .keys()
                    .filter(|key| key.as_str() != "minecraft")
                    .cloned()
                    .collect::<Vec<_>>();
                for loader in loaders {
                    let version = resolve_loader_version(&loader, &minecraft, requested)?;
                    workspace.set_version(&loader, &version)?;
                    println!("{}: {loader} -> {version}", target.display());
                }
            }
            Ok(())
        }
        Some(("migrate", sub)) => {
            let migration = strings(sub, "migration");
            if migration.is_empty() {
                return Err("batch migrate requires a migration".into());
            }
            for target in batch_targets(&root, None)? {
                let mut workspace = Workspace::open(&target)?;
                match migration[0].as_str() {
                    "format" => {
                        let (old, new) = workspace.migrate_format()?;
                        println!("{}: format {old} -> {new}", target.display());
                    }
                    "minecraft" => {
                        let version = migration.get(1).ok_or("minecraft requires a version")?;
                        workspace.set_version("minecraft", version)?;
                        println!("{}: minecraft -> {version}", target.display());
                    }
                    "loader" => {
                        let requested = migration.get(1).map(String::as_str).unwrap_or("latest");
                        let minecraft = workspace
                            .pack()
                            .versions
                            .get("minecraft")
                            .cloned()
                            .ok_or("pack has no Minecraft version")?;
                        let loaders = workspace
                            .pack()
                            .versions
                            .keys()
                            .filter(|key| key.as_str() != "minecraft")
                            .cloned()
                            .collect::<Vec<_>>();
                        for loader in loaders {
                            let version = if matches!(requested, "latest" | "recommended") {
                                resolve_loader_version(&loader, &minecraft, requested)?
                            } else {
                                requested.into()
                            };
                            workspace.set_version(&loader, &version)?;
                            println!("{}: {loader} -> {version}", target.display());
                        }
                    }
                    name => return Err(format!("unknown migration {name:?}").into()),
                }
            }
            Ok(())
        }
        Some(("sync", sub)) => {
            let report =
                packwand_workspace::sync_performance_bases(&root, sub.get_flag("dry-run"))?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        Some((name, _)) => Err(format!("unknown batch operation {name:?}").into()),
        None => Err("batch requires a subcommand".into()),
    }
}

fn batch_provider_add(root: &Path, provider: ProviderKind, args: &ArgMatches) -> Result {
    let Some(("add", sub)) = args.subcommand() else {
        return Err(format!("{} batch operation requires add", provider.name()).into());
    };
    let projects = strings(sub, "projects");
    if projects.is_empty() {
        return Err("provide one or more projects".into());
    }
    let suffix = if provider == ProviderKind::Modrinth {
        "-mr"
    } else {
        "-cf"
    };
    let targets = batch_targets(root, None)?
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(suffix))
        })
        .collect::<Vec<_>>();
    let mut failures = Vec::new();
    let mut added = 0usize;
    for target in targets {
        let mut workspace = Workspace::open(&target)?;
        for project in &projects {
            let mut request = ResolveRequest::new(project);
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
            match resolve_provider(provider, &request, None)
                .and_then(|resolved| workspace.add_resolved(resolved, false).map_err(Into::into))
            {
                Ok(outcome) => {
                    println!("{}: added {}", target.display(), outcome.metadata_path);
                    added += 1;
                }
                Err(error) => failures.push(format!("{} / {project}: {error}", target.display())),
            }
        }
    }
    for failure in &failures {
        eprintln!("warning: {failure}");
    }
    if failures.is_empty() {
        println!("added {added} batch metadata file(s)");
        Ok(())
    } else {
        Err(format!("added {added}; {} batch add(s) failed", failures.len()).into())
    }
}

#[derive(Serialize)]
struct BatchStatus {
    id: String,
    name: String,
    version: String,
    lifecycle: String,
    total_mods: usize,
    subdirs: Vec<BatchSubdir>,
}
#[derive(Serialize)]
struct BatchSubdir {
    key: String,
    mod_count: usize,
}

fn batch_targets(root: &Path, pack: Option<&str>) -> Result<Vec<PathBuf>> {
    if let Some(pack) = pack {
        return Ok(packwand_workspace::read_project(root, &absolute(pack)?)?.subdirs);
    }
    Ok(batch_projects(root)?
        .into_iter()
        .filter(|project| project.category == "modpacks")
        .flat_map(|project| project.subdirs)
        .collect())
}

/// Returns the project containing `root` when invoked from a project directory
/// (or one of its variant subdirectories). Otherwise, discover the projects in
/// the directory as the workspace root.
fn batch_projects(root: &Path) -> Result<Vec<packwand_workspace::Project>> {
    for project_root in root.ancestors() {
        if project_root.join("manifest.json").is_file() {
            let workspace_root = project_root
                .parent()
                .and_then(Path::parent)
                .ok_or_else(|| format!("{} is not inside a Packwand workspace", root.display()))?;
            return Ok(vec![packwand_workspace::read_project(
                workspace_root,
                project_root,
            )?]);
        }
    }
    Ok(packwand_workspace::discover(root)?)
}
