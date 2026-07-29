//! Provider command groups (Modrinth, CurseForge, and the repository
//! providers) plus CurseForge detect/import/open.

use super::*;

pub(super) fn provider_command(provider: ProviderKind, args: &ArgMatches) -> Result {
    let Some((action, sub)) = args.subcommand() else {
        return Err(format!("{} requires a subcommand", provider.name()).into());
    };
    if provider == ProviderKind::CurseForge && action == "update" {
        return curseforge_update(sub);
    }
    if action != "add" {
        return Err(format!(
            "{} does not support the {action} subcommand",
            provider.name()
        )
        .into());
    }
    let root = std::env::current_dir()?;
    let workspace = Workspace::open(&root)?;
    let project = match provider {
        ProviderKind::Modrinth => sub
            .get_one::<String>("project-id")
            .or_else(|| sub.get_one::<String>("project")),
        ProviderKind::CurseForge => sub
            .get_one::<String>("addon-id")
            .or_else(|| sub.get_one::<String>("project")),
        _ => sub.get_one::<String>("project"),
    }
    .ok_or("provide a project ID, slug, or URL")?
    .clone();
    let (project, inferred_file_id) = if provider == ProviderKind::CurseForge {
        packwand_providers::parse_file_url(&project).unwrap_or((project, String::new()))
    } else {
        (project, String::new())
    };
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
    request.channels =
        release_channels(sub.get_one::<String>("release-channel").map(String::as_str));
    request.branch = sub.get_one::<String>("branch").cloned();
    request.asset_pattern = sub.get_one::<String>("regex").cloned();
    request.version_id = sub
        .get_one::<String>("version-id")
        .or_else(|| sub.get_one::<String>("file-id"))
        .cloned()
        .or_else(|| (!inferred_file_id.is_empty()).then_some(inferred_file_id));
    request.version_filename = sub.get_one::<String>("version-filename").cloned();
    let instance = sub.get_one::<String>("instance").cloned();
    let resolved = resolve_provider(provider, &request, instance)?;
    let path = resolved.metadata_path();
    Workspace::open(root)?.add_resolved(resolved, false)?;
    println!("added {path}");
    Ok(())
}

fn curseforge_update(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let mut workspace = Workspace::open(&root)?;
    let name = required(args, "name")?;
    let file_url = required(args, "file")?;
    let (project, file_id) = packwand_providers::parse_file_url(file_url).ok_or(
        "provide a CurseForge file URL such as https://www.curseforge.com/minecraft/mc-mods/sodium/files/8396428",
    )?;

    let mut request = ResolveRequest::new(project);
    request.version_id = Some(file_id);
    let resolved = resolve_provider(ProviderKind::CurseForge, &request, None)?;
    let path = metadata_path(&workspace, name)?;
    let outcome = workspace.replace_with_resolved(&path, resolved)?;
    if outcome.changed {
        println!(
            "updated {} -> {}",
            outcome.old_filename, outcome.new_filename
        );
    } else {
        println!(
            "{} is already on that CurseForge file",
            outcome.metadata_path
        );
    }
    Ok(())
}

pub(super) fn platform_command(provider: ProviderKind, args: &ArgMatches) -> Result {
    match args.subcommand() {
        Some(("detect", _)) if provider == ProviderKind::CurseForge => curseforge_detect(),
        Some(("import", sub)) if provider == ProviderKind::CurseForge => curseforge_import(sub),
        Some(("open", sub)) if provider == ProviderKind::CurseForge => curseforge_open(sub),
        Some(("export", sub)) => {
            let format = match provider {
                ProviderKind::Modrinth => ExportFormat::Modrinth,
                ProviderKind::CurseForge => ExportFormat::CurseForge,
                _ => unreachable!("only distribution platforms export archives"),
            };
            let output = sub.get_one::<String>("output").map(PathBuf::from);
            let artifact = export_pack(
                std::env::current_dir()?,
                format,
                output.as_deref(),
                ExportOptions {
                    restrict_modrinth_domains: true,
                    verify_hashes: sub.get_flag("verify"),
                },
            )?;
            println!(
                "exported {} file(s) to {} ({} bytes)",
                artifact.files,
                artifact.path.display(),
                artifact.bytes
            );
            Ok(())
        }
        _ => provider_command(provider, args),
    }
}

fn curseforge_detect() -> Result {
    let root = std::env::current_dir()?;
    let mods = root.join("mods");
    if !mods.is_dir() {
        return Err(format!("no mods directory at {}", mods.display()).into());
    }
    let mut paths_by_fingerprint = std::collections::BTreeMap::<u32, Vec<PathBuf>>::new();
    for entry in walkdir::WalkDir::new(&mods).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file()
            || !entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| matches!(extension, "jar" | "litemod"))
        {
            continue;
        }
        let fingerprint =
            packwand_pack::hash_file(packwand_pack::HashFormat::Murmur2, entry.path())?
                .parse::<u32>()?;
        paths_by_fingerprint
            .entry(fingerprint)
            .or_default()
            .push(entry.into_path());
    }
    if paths_by_fingerprint.is_empty() {
        println!(
            "no local .jar or .litemod files found in {}",
            mods.display()
        );
        return Ok(());
    }
    let client = CurseForgeClient::new(UreqTransport::new(), configured_api_key());
    let fingerprints = paths_by_fingerprint.keys().copied().collect::<Vec<_>>();
    let matches = client.match_fingerprints(&fingerprints)?;
    let mut workspace = Workspace::open(&root)?;
    let mut converted = 0usize;
    for matched in matches.exact {
        let Some(paths) = paths_by_fingerprint.get_mut(&matched.fingerprint) else {
            continue;
        };
        let Some(path) = paths.pop() else {
            continue;
        };
        let relative = path
            .strip_prefix(&root)?
            .to_string_lossy()
            .replace('\\', "/");
        let mut request = ResolveRequest::new(matched.project_id.to_string());
        request.version_id = Some(matched.file_id.to_string());
        let resolved = client.resolve(&request)?;
        let outcome = workspace.replace_local_with_resolved(&relative, resolved)?;
        println!("{} -> {}", relative, outcome.metadata_path);
        converted += 1;
    }
    for fingerprint in matches.partial {
        if let Some(paths) = paths_by_fingerprint.get(&fingerprint) {
            for path in paths {
                eprintln!("warning: partial CurseForge match for {}", path.display());
            }
        }
    }
    for fingerprint in matches.unmatched {
        if let Some(paths) = paths_by_fingerprint.get(&fingerprint) {
            for path in paths {
                eprintln!("warning: no CurseForge match for {}", path.display());
            }
        }
    }
    let unresolved = paths_by_fingerprint.values().map(Vec::len).sum::<usize>();
    println!("detected {converted} file(s); {unresolved} file(s) remain local");
    Ok(())
}

fn curseforge_import(args: &ArgMatches) -> Result {
    let archive = absolute(required(args, "path")?)?;
    if !archive.is_file() {
        return Err(format!("archive was not found: {}", archive.display()).into());
    }
    let root = std::env::current_dir()?;
    let temporary = tempfile::tempdir_in(root.parent().unwrap_or(&root))?;
    let imported_root = temporary.path().join("pack");
    let client = CurseForgeClient::new(UreqTransport::new(), configured_api_key());
    let imported = import_curseforge_archive(&archive, &imported_root, |project_id, file_id| {
        let mut request = ResolveRequest::new(project_id.to_string());
        request.version_id = Some(file_id.to_string());
        let resolved = client
            .resolve(&request)
            .map_err(|error| error.to_string())?;
        let path = resolved.metadata_path();
        let metadata = resolved.into_mod().map_err(|error| error.to_string())?;
        Ok((path, metadata))
    })?;
    if root.join("pack.toml").is_file() {
        let report = Workspace::open(&root)?.merge_imported_pack(&imported_root)?;
        println!(
            "merged {} indexed file(s), including {} metadata file(s), from {} {}",
            report.files, report.metadata_files, imported.name, imported.version
        );
    } else {
        install_imported_pack(&imported_root, &root)?;
        println!(
            "imported {} {} ({} metadata file(s), {} indexed file(s))",
            imported.name, imported.version, imported.metadata_files, imported.files
        );
    }
    Ok(())
}

fn install_imported_pack(source: &Path, destination: &Path) -> Result {
    let pack: packwand_pack::Pack = toml::from_str(&fs::read_to_string(source.join("pack.toml"))?)?;
    let index: packwand_pack::Index =
        toml::from_str(&fs::read_to_string(source.join(&pack.index.file))?)?;
    let mut files = index
        .files
        .iter()
        .map(|entry| entry.file.clone())
        .collect::<Vec<_>>();
    files.extend([pack.index.file.clone(), ".packwizignore".into()]);
    for relative in &files {
        let target = destination.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if target.exists() {
            return Err(format!("import would overwrite {}", target.display()).into());
        }
    }
    if destination.join("pack.toml").exists() {
        return Err("import would overwrite pack.toml".into());
    }
    for relative in files {
        let source_file = source.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !source_file.is_file() {
            continue;
        }
        let target = destination.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source_file, target)?;
    }
    // Write pack.toml last: an interrupted copy is not mistaken for a valid pack.
    fs::copy(source.join("pack.toml"), destination.join("pack.toml"))?;
    Ok(())
}

fn curseforge_open(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let workspace = Workspace::open(&root)?;
    let path = metadata_path(&workspace, required(args, "name")?)?;
    let metadata: Mod = toml::from_str(&fs::read_to_string(
        root.join(path.replace('/', std::path::MAIN_SEPARATOR_STR)),
    )?)?;
    let project_id = metadata
        .update
        .get("curseforge")
        .and_then(|table| table.get("project-id"))
        .and_then(|value| {
            value
                .as_integer()
                .map(|id| id.to_string())
                .or_else(|| value.as_str().map(str::to_owned))
        })
        .ok_or("metadata has no CurseForge project ID")?;
    let url = format!("https://www.curseforge.com/projects/{project_id}");
    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", &url])
        .status();
    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open").arg(&url).status();
    #[cfg(all(unix, not(target_os = "macos")))]
    let status = std::process::Command::new("xdg-open").arg(&url).status();
    match status {
        Ok(status) if status.success() => println!("opened {url}"),
        Ok(status) => println!("browser exited with {status}; open {url}"),
        Err(error) => println!("could not open a browser ({error}); open {url}"),
    }
    Ok(())
}
