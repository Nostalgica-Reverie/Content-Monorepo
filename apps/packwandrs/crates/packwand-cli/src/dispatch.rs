use std::error::Error;
use std::fs;
use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use clap_complete::{Shell, generate};
use packwand_build::{
    ExportFormat, ExportOptions, export_pack, import_curseforge_archive, import_modrinth_archive,
};
use packwand_ops::Workspace;
use packwand_pack::Mod;
use packwand_providers::configured_api_key;
use packwand_providers::{
    CurseForgeClient, ForgejoClient, GitHubClient, GitLabClient, ModrinthClient, ProviderKind,
    ProviderResolver, ReleaseChannel, ResolveRequest, ResolvedProject, Transport, UreqTransport,
};
use packwand_workspace::{Manifest, NewProject, ProjectRole};
use serde::Serialize;

use crate::cli;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub fn run() -> Result {
    let mut root = cli::build();
    let matches = root.clone().get_matches();
    if let Some(path) = requested_group_path(&root, &matches) {
        let mut command = &mut root;
        for name in path {
            command = command
                .find_subcommand_mut(&name)
                .ok_or_else(|| format!("command {name:?} disappeared from the command tree"))?;
        }
        print_mascot();
        command.print_help()?;
        println!();
        return Ok(());
    }
    match matches.subcommand() {
        Some(("completion", args)) => completion(&mut root, args),
        Some(("version", _)) => {
            print_mascot();
            println!("packwand 26.2.0");
            Ok(())
        }
        Some(("list", args)) => list(args),
        Some(("add", args)) => add_workspace(args),
        Some(("refresh", _)) => refresh(),
        Some(("pin", args)) => pin(args, true),
        Some(("port", args)) => port(args),
        Some(("unpin", args)) => pin(args, false),
        Some(("remove", args)) => remove(args),
        Some(("rehash", args)) => rehash(args),
        Some(("new", args)) => new_project(args),
        Some(("init", args)) => init_pack(args),
        Some(("import", args)) => import_archive(args),
        Some(("build", args)) => crate::build_cmd::run(args),
        Some(("bump", args)) => bump(args),
        Some(("freeze", args)) => freeze(args, true),
        Some(("unfreeze", args)) => freeze(args, false),
        Some(("side", args)) => side(args),
        Some(("packs", args)) => packs(args),
        Some(("workspace", args)) => workspace_command(args),
        Some(("automation", args)) => automation(args),
        Some(("cache", args)) => cache_command(args, &matches),
        Some(("api", args)) => crate::api_cmd::run(args),
        Some(("doctor", args)) => doctor(args),
        Some(("content-lint", args)) => content_lint_command(args),
        Some(("ci-local", args)) => ci_local(args),
        Some(("preflight", args)) => preflight(args),
        Some(("registry", args)) => registry_command(args),
        Some(("diff", args)) => diff_command(args),
        Some(("gui", _)) => launch_gui(),
        Some(("json", args)) => json(args),
        Some(("modlist", args)) => modlist(args),
        Some(("nix", args)) => nix_command(args),
        Some(("pages", args)) => crate::pages::run(args),
        Some(("settings", args)) => settings_command(args),
        Some(("run", args)) => run_script(args),
        Some(("serve", args)) => crate::serve::run(args),
        Some(("test", args)) => crate::test_cmd::run(args),
        Some(("modrinth", args)) => platform_command(ProviderKind::Modrinth, args),
        Some(("curseforge", args)) => platform_command(ProviderKind::CurseForge, args),
        Some(("forgejo", args)) => provider_command(ProviderKind::Forgejo, args),
        Some(("github", args)) => provider_command(ProviderKind::GitHub, args),
        Some(("gitlab", args)) => provider_command(ProviderKind::GitLab, args),
        Some(("url", args)) => url_command(args),
        Some(("migrate", args)) => migrate(args),
        Some(("update", args)) => update_command(args),
        Some(("export", args)) => export_local(args),
        Some(("publish", args)) => publish_command(args),
        Some(("lint", args)) => lint(args),
        Some(("parity", args)) => parity(args),
        Some(("validate", args)) => validate(args),
        Some(("utils", args)) => utils(args),
        Some((name, _)) => Err(format!("unknown command dispatch {name:?}").into()),
        None => {
            print_mascot();
            root.print_help()?;
            println!();
            Ok(())
        }
    }
}

fn requested_group_path(root: &clap::Command, matches: &ArgMatches) -> Option<Vec<String>> {
    let mut command = root;
    let mut arguments = matches;
    let mut path = Vec::new();
    loop {
        let Some((name, nested)) = arguments.subcommand() else {
            return (!path.is_empty() && command.has_subcommands()).then_some(path);
        };
        command = command.find_subcommand(name)?;
        arguments = nested;
        path.push(name.to_owned());
    }
}

fn print_mascot() {
    if !std::io::stderr().is_terminal() {
        return;
    }
    let _ = std::io::stderr().write_all(
        concat!(
            "                          z\n",
            "      ▄▄▄▄▄▄▄▄▄▄▄▄        z z\n",
            "      ██ ──  ── ██\n",
            "  ▄█▄ ████████████ ▄█▄\n",
            "  ▀▀  ████████████  ▀▀\n",
            "      ▀█▀ ▀█▀▀█▀ ▀█▀    mimimi...\n\n",
        )
        .as_bytes(),
    );
}

fn completion(root: &mut clap::Command, args: &ArgMatches) -> Result {
    let shell = match required(args, "shell")? {
        "bash" => Shell::Bash,
        "elvish" => Shell::Elvish,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        "zsh" => Shell::Zsh,
        value => return Err(format!("unsupported shell {value:?}").into()),
    };
    generate(shell, root, "packwand", &mut std::io::stdout());
    Ok(())
}

fn list(args: &ArgMatches) -> Result {
    let workspace = Workspace::open(std::env::current_dir()?)?;
    let side = args.get_one::<String>("side").map(String::as_str);
    let mut entries = Vec::new();
    for item in workspace
        .index()
        .files
        .iter()
        .filter(|item| item.metafile && item.alias.is_none())
    {
        let metadata: Mod = toml::from_str(&fs::read_to_string(
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

fn add_workspace(args: &ArgMatches) -> Result {
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

fn refresh() -> Result {
    let mut workspace = Workspace::open(std::env::current_dir()?)?;
    let report = workspace.refresh_metadata_index()?;
    println!(
        "refreshed index: {} added, {} updated, {} removed",
        report.added, report.updated, report.removed
    );
    Ok(())
}

fn pin(args: &ArgMatches, pinned: bool) -> Result {
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

fn port(args: &ArgMatches) -> Result {
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
            if let Some(slug) = name.strip_suffix(".pw.toml") {
                slugs.push(slug.into());
            }
        }
    }
    Ok(slugs)
}

fn remove(args: &ArgMatches) -> Result {
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

fn rehash(args: &ArgMatches) -> Result {
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

fn metadata_path(workspace: &Workspace, name: &str) -> Result<String> {
    let normalized = name.trim_end_matches(".pw.toml");
    let matches = workspace
        .index()
        .files
        .iter()
        .filter(|item| item.metafile && item.alias.is_none())
        .filter(|item| {
            item.file == name
                || item.file.trim_end_matches(".pw.toml") == normalized
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

fn new_project(args: &ArgMatches) -> Result {
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

fn init_pack(args: &ArgMatches) -> Result {
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

fn import_archive(args: &ArgMatches) -> Result {
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

fn bump(args: &ArgMatches) -> Result {
    let root = absolute(required(args, "pack-dir")?)?;
    let workspace_root = root
        .parent()
        .and_then(Path::parent)
        .ok_or("project is not inside a category directory")?;
    let project = packwand_workspace::read_project(workspace_root, &root)?;
    let old = project.manifest.version.clone();
    let mut manifest = project.manifest;
    manifest.version = required(args, "new-version")?.to_owned();
    packwand_workspace::write_manifest(&root, &manifest)?;
    println!("bumped {}: {} -> {}", manifest.id, old, manifest.version);
    Ok(())
}

fn freeze(args: &ArgMatches, frozen: bool) -> Result {
    let subdir = absolute(required(args, "pack-subdir")?)?;
    let project_root = subdir.parent().ok_or("pack subdir has no project parent")?;
    let workspace_root = project_root
        .parent()
        .and_then(Path::parent)
        .ok_or("project is not inside a category directory")?;
    let project = packwand_workspace::read_project(workspace_root, project_root)?;
    let subdir_name = subdir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("invalid subdir name")?;
    let slugs = strings(args, "mod-slugs");
    if slugs.is_empty() {
        let values = project
            .manifest
            .automation()
            .freeze
            .get(subdir_name)
            .cloned()
            .unwrap_or_default();
        if args.get_flag("json") {
            println!("{}", serde_json::to_string_pretty(&values)?);
        } else {
            for slug in values {
                println!("{slug}");
            }
        }
        return Ok(());
    }
    let changed = packwand_workspace::set_frozen(
        workspace_root,
        &project.manifest.id,
        subdir_name,
        &slugs,
        frozen,
    )?;
    println!(
        "{} {} mod(s)",
        if frozen { "froze" } else { "unfroze" },
        changed.len()
    );
    Ok(())
}

fn side(args: &ArgMatches) -> Result {
    let project_root = absolute(required(args, "pack-dir")?)?;
    let workspace_root = project_root
        .parent()
        .and_then(Path::parent)
        .ok_or("project is not inside a category directory")?;
    let project = packwand_workspace::read_project(workspace_root, &project_root)?;
    let slug = required(args, "mod-slug")?;
    let requested = args.get_one::<String>("side").map(|value| {
        if value == "either" {
            "both"
        } else {
            value.as_str()
        }
    });
    if requested.is_some_and(|value| !matches!(value, "client" | "server" | "both")) {
        return Err("side must be client, server, both, or either".into());
    }
    let mut found = 0;
    for subdir in project.subdirs {
        for folder in ["mods", "resourcepacks", "shaderpacks"] {
            let relative = format!("{folder}/{slug}.pw.toml");
            if !subdir.join(&relative).is_file() {
                continue;
            }
            found += 1;
            let mut workspace = Workspace::open(&subdir)?;
            if let Some(side) = requested {
                let changed = workspace.set_side(&relative, side)?;
                println!(
                    "{}: {}",
                    subdir.display(),
                    if changed { side } else { "unchanged" }
                );
            } else {
                let metadata: Mod = toml::from_str(&fs::read_to_string(subdir.join(relative))?)?;
                println!(
                    "{}: {}",
                    subdir.display(),
                    if metadata.side.is_empty() {
                        "both"
                    } else {
                        &metadata.side
                    }
                );
            }
        }
    }
    if found == 0 {
        Err(format!("{slug} was not found in any pack subdir").into())
    } else {
        Ok(())
    }
}

fn packs(args: &ArgMatches) -> Result {
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

fn workspace_command(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    match args.subcommand() {
        Some(("status", sub)) => {
            let mut statuses = Vec::new();
            for project in workspace_projects(&root)?
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
                    subdirs.push(WorkspaceSubdir {
                        key: path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                        mod_count: count,
                    });
                }
                let lifecycle = project.manifest.lifecycle().to_owned();
                statuses.push(WorkspaceStatus {
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
                workspace_targets(&root, sub.get_one::<String>("pack-dir").map(String::as_str))?;
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
                workspace_targets(&root, sub.get_one::<String>("pack-dir").map(String::as_str))?;
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
                Err("no workspace export targets matched".into())
            } else {
                Ok(())
            }
        }
        Some(("mr", provider_args)) => {
            workspace_provider_add(&root, ProviderKind::Modrinth, provider_args)
        }
        Some(("cf", provider_args)) => {
            workspace_provider_add(&root, ProviderKind::CurseForge, provider_args)
        }
        Some(("update", sub)) => {
            let targets =
                workspace_targets(&root, sub.get_one::<String>("pack-dir").map(String::as_str))?;
            let dry_run = sub.get_flag("check");
            let mut results = Vec::new();
            let mut failures = 0usize;
            for target in targets {
                match update_workspace(&target, None, true, dry_run) {
                    Ok(records) => {
                        failures += records
                            .iter()
                            .filter(|record| {
                                record.error.is_some() && record.error.as_deref() != Some("pinned")
                            })
                            .count();
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
                Err(format!("{failures} workspace update(s) failed").into())
            }
        }
        Some(("loader-update", sub)) => {
            let requested = sub
                .get_one::<String>("version")
                .map(String::as_str)
                .unwrap_or("latest");
            let targets =
                workspace_targets(&root, sub.get_one::<String>("pack-dir").map(String::as_str))?;
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
                return Err("workspace migrate requires a migration".into());
            }
            for target in workspace_targets(&root, None)? {
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
        Some((name, _)) => Err(format!("unknown workspace operation {name:?}").into()),
        None => Err("workspace requires a subcommand".into()),
    }
}

fn workspace_provider_add(root: &Path, provider: ProviderKind, args: &ArgMatches) -> Result {
    let Some(("add", sub)) = args.subcommand() else {
        return Err(format!("{} workspace operation requires add", provider.name()).into());
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
    let targets = workspace_targets(root, None)?
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
        println!("added {added} workspace metadata file(s)");
        Ok(())
    } else {
        Err(format!("added {added}; {} workspace add(s) failed", failures.len()).into())
    }
}

#[derive(Serialize)]
struct WorkspaceStatus {
    id: String,
    name: String,
    version: String,
    lifecycle: String,
    total_mods: usize,
    subdirs: Vec<WorkspaceSubdir>,
}
#[derive(Serialize)]
struct WorkspaceSubdir {
    key: String,
    mod_count: usize,
}

fn workspace_targets(root: &Path, pack: Option<&str>) -> Result<Vec<PathBuf>> {
    if let Some(pack) = pack {
        return Ok(packwand_workspace::read_project(root, &absolute(pack)?)?.subdirs);
    }
    Ok(workspace_projects(root)?
        .into_iter()
        .filter(|project| project.category == "modpacks")
        .flat_map(|project| project.subdirs)
        .collect())
}

/// Returns the project containing `root` when invoked from a project directory
/// (or one of its variant subdirectories). Otherwise, discover the projects in
/// the directory as the workspace root.
fn workspace_projects(root: &Path) -> Result<Vec<packwand_workspace::Project>> {
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

fn automation(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    match args.subcommand() {
        Some(("get", sub)) => {
            let project_root = absolute(required(sub, "pack-dir")?)?;
            let project = packwand_workspace::read_project(&root, &project_root)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&project.manifest.automation())?
            );
            Ok(())
        }
        Some(("list-full-auto", _)) => {
            for project in packwand_workspace::discover(root)? {
                if project
                    .manifest
                    .automation
                    .as_ref()
                    .and_then(|automation| automation.full_auto.as_ref())
                    .and_then(|value| value.get("enabled"))
                    .and_then(serde_json::Value::as_bool)
                    == Some(true)
                {
                    println!("{}", project.manifest.id);
                }
            }
            Ok(())
        }
        Some(("run", sub)) => automation_run(&root, sub),
        _ => Err("automation requires get, run, or list-full-auto".into()),
    }
}

#[derive(Serialize)]
struct AutomationStep {
    name: &'static str,
    status: &'static str,
    detail: String,
}

#[derive(Serialize)]
struct AutomationReport {
    pack_dir: String,
    pack_id: String,
    old_version: String,
    new_version: String,
    dry_run: bool,
    status: &'static str,
    steps: Vec<AutomationStep>,
}

fn automation_run(root: &Path, args: &ArgMatches) -> Result {
    let project_root = absolute(required(args, "pack-dir")?)?;
    let project = packwand_workspace::read_project(root, &project_root)?;
    let full_auto = project
        .manifest
        .automation
        .as_ref()
        .and_then(|value| value.full_auto.as_ref());
    if full_auto
        .and_then(|value| value.get("enabled"))
        .and_then(serde_json::Value::as_bool)
        != Some(true)
    {
        return Err("automation.full_auto.enabled is not true".into());
    }
    if project.subdirs.is_empty() {
        return Err("automation target has no pack subdirectories".into());
    }
    let dry_run = args.get_flag("dry-run");
    let validation = packwand_diagnostics::validate_projects(root)?;
    if !validation.valid() {
        return Err("pre-automation manifest validation failed".into());
    }
    let mut steps = vec![AutomationStep {
        name: "validate",
        status: "ok",
        detail: format!("{} document(s) checked", validation.checked),
    }];
    let mut changed = 0usize;
    for subdir in &project.subdirs {
        let records = update_workspace(subdir, None, true, dry_run)?;
        let failures = records
            .iter()
            .filter(|record| {
                record
                    .error
                    .as_deref()
                    .is_some_and(|error| error != "pinned")
            })
            .count();
        if failures > 0 {
            return Err(format!(
                "{failures} provider update(s) failed in {}",
                subdir.display()
            )
            .into());
        }
        changed += records.iter().filter(|record| record.changed).count();
    }
    steps.push(AutomationStep {
        name: "update",
        status: "ok",
        detail: format!(
            "{changed} compatible update(s){}",
            if dry_run { " found" } else { " applied" }
        ),
    });
    let sync = packwand_workspace::sync_performance_bases(root, dry_run)?;
    steps.push(AutomationStep {
        name: "sync",
        status: "ok",
        detail: format!("{} copied, {} deleted", sync.copied, sync.deleted),
    });
    let tests = full_auto
        .and_then(|value| value.get("tests"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    if dry_run {
        steps.push(AutomationStep {
            name: "tests",
            status: "skipped",
            detail: format!("would run {} configured test(s)", tests.len()),
        });
    } else {
        for test in &tests {
            #[cfg(windows)]
            let status = std::process::Command::new("cmd")
                .args(["/C", test])
                .current_dir(&project.root)
                .status()?;
            #[cfg(not(windows))]
            let status = std::process::Command::new("sh")
                .args(["-c", test])
                .current_dir(&project.root)
                .status()?;
            if !status.success() {
                return Err(format!("automation test {test:?} failed with {status}").into());
            }
        }
        steps.push(AutomationStep {
            name: "tests",
            status: if tests.is_empty() { "skipped" } else { "ok" },
            detail: format!("{} configured test(s)", tests.len()),
        });
    }
    let next = next_calver(&project.manifest.version);
    if dry_run {
        steps.push(AutomationStep {
            name: "bump",
            status: "skipped",
            detail: format!("would bump {} -> {next}", project.manifest.version),
        });
    } else {
        packwand_workspace::bump(root, &project.manifest.id, &next)?;
        steps.push(AutomationStep {
            name: "bump",
            status: "ok",
            detail: format!("{} -> {next}", project.manifest.version),
        });
    }
    let report = AutomationReport {
        pack_dir: project.root.to_string_lossy().into_owned(),
        pack_id: project.manifest.id,
        old_version: project.manifest.version,
        new_version: next,
        dry_run,
        status: if changed == 0 {
            "no_changes"
        } else {
            "ready_to_publish"
        },
        steps,
    };
    let encoded = serde_json::to_vec_pretty(&report)?;
    if let Some(path) = args.get_one::<String>("report") {
        let path = absolute(path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &encoded)?;
    }
    if args.get_flag("json") {
        println!("{}", String::from_utf8(encoded)?);
    } else {
        println!("automation {}: {}", report.pack_id, report.status);
        for step in &report.steps {
            println!("  {:<10} {:<8} {}", step.name, step.status, step.detail);
        }
    }
    Ok(())
}

fn next_calver(current: &str) -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let z = seconds.div_euclid(86_400) + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    let cycle = format!("{:02}.{month:02}", year % 100);
    let mut parts = current.split('.');
    let previous = match (parts.next(), parts.next()) {
        (Some(year), Some(month)) => format!("{year}.{month}"),
        _ => String::new(),
    };
    if previous != cycle {
        return cycle;
    }
    let patch = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0)
        + 1;
    format!("{cycle}.{patch}")
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct CacheIndex {
    version: u32,
    hashes: std::collections::BTreeMap<String, Vec<String>>,
}

#[derive(Serialize)]
struct CachePruneEntry {
    hash: String,
    size_bytes: u64,
}

#[derive(Serialize)]
struct CachePruneResult {
    scanned_entries: usize,
    removed_entries: Vec<CachePruneEntry>,
    removed_bytes: u64,
    dry_run: bool,
}

fn cache_command(args: &ArgMatches, root_args: &ArgMatches) -> Result {
    let Some(("prune", sub)) = args.subcommand() else {
        return Err("cache requires prune".into());
    };
    let cache = root_args
        .get_one::<String>("cache")
        .map(PathBuf::from)
        .unwrap_or_else(default_cache_path);
    let index_path = cache.join("index.json");
    let mut index = match fs::read(&index_path) {
        Ok(bytes) => serde_json::from_slice::<CacheIndex>(&bytes)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => CacheIndex {
            version: 2,
            ..CacheIndex::default()
        },
        Err(error) => return Err(error.into()),
    };
    if index.version > 2 {
        return Err(format!(
            "cache index version {} is newer than supported version 2",
            index.version
        )
        .into());
    }
    let referenced = referenced_download_hashes(std::env::current_dir()?)?;
    let sha256 = index.hashes.get("sha256").cloned().unwrap_or_default();
    let mut removals = Vec::new();
    let mut remove_indices = Vec::new();
    for (position, hash) in sha256.iter().enumerate() {
        let used = index.hashes.values().any(|hashes| {
            hashes
                .get(position)
                .is_some_and(|candidate| referenced.contains(&candidate.to_ascii_lowercase()))
        });
        if used || hash.is_empty() {
            continue;
        }
        if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!("unsafe sha256 cache key {hash:?}").into());
        }
        let path = cache.join(&hash[..2]).join(&hash[2..]);
        let size = fs::metadata(&path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        removals.push(CachePruneEntry {
            hash: hash.clone(),
            size_bytes: size,
        });
        remove_indices.push(position);
        if !sub.get_flag("dry-run") {
            match fs::remove_file(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(format!("failed to remove {}: {error}", path.display()).into());
                }
            }
        }
    }
    let removed_bytes = removals.iter().map(|entry| entry.size_bytes).sum();
    let result = CachePruneResult {
        scanned_entries: sha256.len(),
        removed_entries: removals,
        removed_bytes,
        dry_run: sub.get_flag("dry-run"),
    };
    if !result.dry_run && !remove_indices.is_empty() {
        for hashes in index.hashes.values_mut() {
            let mut position = 0usize;
            hashes.retain(|_| {
                let keep = !remove_indices.contains(&position);
                position += 1;
                keep
            });
        }
        fs::create_dir_all(&cache)?;
        let mut bytes = serde_json::to_vec(&index)?;
        bytes.push(b'\n');
        let mut temporary = tempfile::NamedTempFile::new_in(&cache)?;
        use std::io::Write as _;
        temporary.write_all(&bytes)?;
        temporary.persist(&index_path)?;
    }
    if sub.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "{} {}/{} cache entries ({:.1} MB)",
            if result.dry_run {
                "would remove"
            } else {
                "removed"
            },
            result.removed_entries.len(),
            result.scanned_entries,
            result.removed_bytes as f64 / 1_000_000.0
        );
    }
    Ok(())
}

fn default_cache_path() -> PathBuf {
    if let Some(path) = std::env::var_os("PACKWAND_CACHE") {
        return path.into();
    }
    #[cfg(windows)]
    if let Some(path) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(path).join("packwand/cache");
    }
    if let Some(path) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(path).join("packwand");
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache/packwand")
}

fn referenced_download_hashes(root: PathBuf) -> Result<std::collections::BTreeSet<String>> {
    let mut hashes = std::collections::BTreeSet::new();
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0
                || !entry.file_type().is_dir()
                || !matches!(
                    entry.file_name().to_str(),
                    Some(".git" | "target" | "node_modules")
                )
        })
    {
        let entry = entry?;
        if !entry.file_type().is_file()
            || !entry.file_name().to_string_lossy().ends_with(".pw.toml")
        {
            continue;
        }
        let metadata: Mod = toml::from_str(&fs::read_to_string(entry.path())?)?;
        if !metadata.download.hash.is_empty() {
            hashes.insert(metadata.download.hash.to_ascii_lowercase());
        }
        hashes.extend(
            metadata
                .download
                .extra_hashes
                .values()
                .filter(|hash| !hash.is_empty())
                .map(|hash| hash.to_ascii_lowercase()),
        );
    }
    Ok(hashes)
}

fn doctor(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let mut checks = Vec::new();
    checks.push(DoctorCheck {
        name: "repo-root",
        ok: root.join(".git").exists() || root.join("modpacks").is_dir(),
        detail: root.display().to_string(),
    });
    match packwand_workspace::discover(&root) {
        Ok(projects) => checks.push(DoctorCheck {
            name: "manifests",
            ok: !projects.is_empty(),
            detail: format!("{} project(s)", projects.len()),
        }),
        Err(error) => checks.push(DoctorCheck {
            name: "manifests",
            ok: false,
            detail: error.to_string(),
        }),
    }
    for tool in ["git", "java"] {
        let available = std::process::Command::new(tool)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());
        checks.push(DoctorCheck {
            name: tool,
            ok: available,
            detail: if available {
                "available".into()
            } else {
                "not found".into()
            },
        });
    }
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&checks)?);
    } else {
        for check in &checks {
            println!(
                "{} {}: {}",
                if check.ok { "ok" } else { "FAIL" },
                check.name,
                check.detail
            );
        }
    }
    if checks.iter().all(|check| check.ok) {
        Ok(())
    } else {
        Err("doctor found one or more problems".into())
    }
}

#[derive(Default, Serialize)]
struct DiffResult {
    old_ref: String,
    new_ref: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    path_prefix: String,
    subdirs: Vec<DiffSubdir>,
    total_added: usize,
    total_removed: usize,
    total_updated: usize,
}

#[derive(Serialize)]
struct DiffSubdir {
    subdir: String,
    added: usize,
    removed: usize,
    updated: usize,
    mods: Vec<DiffMod>,
}

#[derive(Serialize)]
struct DiffMod {
    slug: String,
    path: String,
    change: &'static str,
    #[serde(skip_serializing_if = "String::is_empty")]
    old_filename: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    new_filename: String,
}

fn diff_command(args: &ArgMatches) -> Result {
    let old_ref = required(args, "old-ref")?;
    let new_ref = required(args, "new-ref")?;
    let path_prefix = args
        .get_one::<String>("path-prefix")
        .cloned()
        .unwrap_or_default();
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", &format!("{old_ref}..{new_ref}")])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    let mut changed = String::from_utf8(output.stdout)?
        .lines()
        .map(str::trim)
        .filter(|path| path.ends_with(".pw.toml"))
        .filter(|path| path_prefix.is_empty() || path.starts_with(&path_prefix))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    changed.sort();
    let mut grouped = std::collections::BTreeMap::<String, Vec<String>>::new();
    for path in changed {
        let subdir = Path::new(&path)
            .parent()
            .and_then(Path::parent)
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        grouped.entry(subdir).or_default().push(path);
    }
    let mut result = DiffResult {
        old_ref: old_ref.into(),
        new_ref: new_ref.into(),
        path_prefix,
        ..DiffResult::default()
    };
    for (subdir, paths) in grouped {
        let mut summary = DiffSubdir {
            subdir,
            added: 0,
            removed: 0,
            updated: 0,
            mods: Vec::new(),
        };
        for path in paths {
            let old = git_show(old_ref, &path)?;
            let new = git_show(new_ref, &path)?;
            let old_filename = old.as_deref().map(metadata_filename).unwrap_or_default();
            let new_filename = new.as_deref().map(metadata_filename).unwrap_or_default();
            let change = match (old.is_some(), new.is_some()) {
                (false, true) => {
                    summary.added += 1;
                    result.total_added += 1;
                    "added"
                }
                (true, false) => {
                    summary.removed += 1;
                    result.total_removed += 1;
                    "removed"
                }
                _ => {
                    summary.updated += 1;
                    result.total_updated += 1;
                    "updated"
                }
            };
            summary.mods.push(DiffMod {
                slug: Path::new(&path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(&path)
                    .trim_end_matches(".pw.toml")
                    .into(),
                path,
                change,
                old_filename,
                new_filename,
            });
        }
        result.subdirs.push(summary);
    }
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if result.subdirs.is_empty() {
        println!("no .pw.toml changes between {old_ref} and {new_ref}");
    } else {
        for subdir in &result.subdirs {
            println!("{}:", subdir.subdir);
            for change in &subdir.mods {
                match change.change {
                    "added" => println!("  + {:38} {}", change.slug, change.new_filename),
                    "removed" => println!("  - {:38} {}", change.slug, change.old_filename),
                    _ if change.old_filename != change.new_filename => println!(
                        "  ~ {:38} {} -> {}",
                        change.slug, change.old_filename, change.new_filename
                    ),
                    _ => println!("  ~ {}", change.slug),
                }
            }
            println!(
                "  +{} -{} ~{}\n",
                subdir.added, subdir.removed, subdir.updated
            );
        }
        println!(
            "{old_ref}..{new_ref}: +{} added  -{} removed  ~{} updated",
            result.total_added, result.total_removed, result.total_updated
        );
    }
    Ok(())
}

fn launch_gui() -> Result {
    let executable = std::env::current_exe()?;
    let filename = if cfg!(windows) {
        "packwand-gui.exe"
    } else {
        "packwand-gui"
    };
    let candidate = executable
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(filename);
    if !candidate.is_file() {
        return Err(format!(
            "native GUI not found beside the CLI at {}; install/build packwand-gui",
            candidate.display()
        )
        .into());
    }
    std::process::Command::new(&candidate)
        .current_dir(std::env::current_dir()?)
        .spawn()?;
    println!("launched {}", candidate.display());
    Ok(())
}

fn git_show(reference: &str, path: &str) -> Result<Option<String>> {
    let output = std::process::Command::new("git")
        .args(["show", &format!("{reference}:{path}")])
        .output()?;
    if output.status.success() {
        Ok(Some(String::from_utf8(output.stdout)?))
    } else {
        Ok(None)
    }
}

fn metadata_filename(source: &str) -> String {
    toml::from_str::<Mod>(source)
        .map(|metadata| metadata.filename)
        .unwrap_or_default()
}

#[derive(Serialize)]
struct DoctorCheck {
    name: &'static str,
    ok: bool,
    detail: String,
}

fn json(args: &ArgMatches) -> Result {
    let Some(("minify", sub)) = args.subcommand() else {
        return Err("json requires minify".into());
    };
    let check = sub.get_flag("check");
    let strict = sub.get_flag("strict");
    let mut files = Vec::new();
    for raw in strings(sub, "paths") {
        let path = absolute(raw)?;
        if path.is_dir() {
            files.extend(
                walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_entry(|entry| {
                        entry.depth() == 0
                            || !entry.file_type().is_dir()
                            || !matches!(entry.file_name().to_str(), Some(".git" | "node_modules"))
                    })
                    .flatten()
                    .filter(|entry| entry.file_type().is_file() && is_json_path(entry.path()))
                    .map(|entry| entry.into_path()),
            );
        } else if is_json_path(&path) {
            files.push(path);
        }
    }
    let mut changed = 0;
    let mut skipped = 0;
    let mut saved = 0usize;
    for path in &files {
        let source = fs::read(path)?;
        if let Err(error) = serde_json::from_slice::<serde_json::Value>(&source) {
            if strict {
                return Err(format!("{} is not valid JSON: {error}", path.display()).into());
            }
            skipped += 1;
            continue;
        }
        let compact = compact_json(&source);
        if compact.len() < source.len() {
            changed += 1;
            saved += source.len() - compact.len();
            if !check {
                fs::write(path, compact)?;
            }
        }
    }
    println!(
        "{} {changed} of {} JSON file(s), saving {saved} bytes; {skipped} skipped",
        if check { "would minify" } else { "minified" },
        files.len()
    );
    if check && changed > 0 {
        Err(format!("{changed} file(s) require minification").into())
    } else {
        Ok(())
    }
}

fn is_json_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext, "json" | "mcmeta"))
}

fn compact_json(source: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(source.len());
    let mut quoted = false;
    let mut escaped = false;
    for &byte in source {
        if quoted {
            output.push(byte);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else if byte == b'"' {
            quoted = true;
            output.push(byte);
        } else if !byte.is_ascii_whitespace() {
            output.push(byte);
        }
    }
    output
}

fn modlist(args: &ArgMatches) -> Result {
    let subdir = absolute(required(args, "subdir")?)?;
    let mods_dir = subdir.join("mods");
    if !mods_dir.is_dir() {
        return Err(format!("no mods directory at {}", mods_dir.display()).into());
    }
    let mut entries = std::collections::BTreeMap::new();
    let mut parsed = 0;
    for entry in fs::read_dir(&mods_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file()
            || !entry.file_name().to_string_lossy().ends_with(".pw.toml")
        {
            continue;
        }
        let metadata: Mod = toml::from_str(&fs::read_to_string(entry.path())?)?;
        parsed += 1;
        let modrinth_id = metadata
            .update
            .get("modrinth")
            .and_then(|table| table.get("mod-id"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let cf_file = metadata
            .update
            .get("curseforge")
            .and_then(|table| table.get("file-id"))
            .and_then(toml::Value::as_integer);
        let mr_hash = (modrinth_id.is_some()
            && metadata.download.hash_format == "sha1"
            && !metadata.download.hash.is_empty())
        .then(|| metadata.download.hash.clone());
        entries.insert(
            metadata.filename.clone(),
            CrashMod {
                jar_name: metadata.filename.clone(),
                mod_id: modrinth_id,
                name: metadata.name,
                version: metadata.filename.trim_end_matches(".jar").to_owned(),
                curse_forge_hash: cf_file,
                modrinth_hash: mr_hash,
            },
        );
    }
    let output_dir = subdir.join("config/crash_assistant");
    fs::create_dir_all(&output_dir)?;
    let output = output_dir.join("modlist.json");
    let mut data = serde_json::to_vec_pretty(&entries)?;
    data.push(b'\n');
    fs::write(&output, data)?;
    if args.get_flag("json") {
        println!(
            "{}",
            serde_json::json!({"subdir":subdir,"out_path":output,"mod_count":parsed})
        );
    } else {
        println!("wrote {} ({parsed} mods)", output.display());
    }
    Ok(())
}

#[derive(Serialize)]
struct NixChecksum {
    url: String,
    sha256: String,
}

fn nix_command(args: &ArgMatches) -> Result {
    let Some(("gen", sub)) = args.subcommand() else {
        return Err("nix requires gen".into());
    };
    let output = required(sub, "output")?;
    let root = std::env::current_dir()?;
    if sub.get_flag("all") {
        let mut pack_roots = Vec::new();
        for entry in walkdir::WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                entry.depth() == 0
                    || !entry.file_type().is_dir()
                    || !matches!(
                        entry.file_name().to_str(),
                        Some(".git" | "target" | "node_modules")
                    )
            })
        {
            let entry = entry?;
            if entry.file_type().is_file()
                && entry.file_name() == "pack.toml"
                && let Some(parent) = entry.path().parent()
            {
                pack_roots.push(parent.to_path_buf());
            }
        }
        pack_roots.sort();
        if pack_roots.is_empty() {
            return Err("no pack subdirectories found".into());
        }
        for pack_root in &pack_roots {
            write_nix_checksums(pack_root, output)?;
        }
        println!("generated checksums for {} pack(s)", pack_roots.len());
        Ok(())
    } else {
        let count = write_nix_checksums(&root, output)?;
        println!("wrote {} ({count} mod(s))", root.join(output).display());
        Ok(())
    }
}

fn settings_command(args: &ArgMatches) -> Result {
    let (sub, key, noun) = match args.subcommand() {
        Some(("acceptable-loaders", sub)) => (sub, "acceptable-game-loaders", "loader"),
        Some(("acceptable-versions", sub)) => (sub, "acceptable-game-versions", "version"),
        _ => return Err("settings requires acceptable-loaders or acceptable-versions".into()),
    };
    if sub.get_flag("add") && sub.get_flag("remove") {
        return Err("--add and --remove are mutually exclusive".into());
    }
    let input = required(sub, noun)?;
    let mut workspace = Workspace::open(std::env::current_dir()?)?;
    let mut values = workspace
        .pack()
        .options
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let action = if sub.get_flag("add") {
        if values.iter().any(|value| value == input) {
            return Err(format!("{noun} {input:?} is already accepted").into());
        }
        values.push(input.into());
        "Added"
    } else if sub.get_flag("remove") {
        let before = values.len();
        values.retain(|value| value != input);
        if before == values.len() {
            return Err(format!("{noun} {input:?} is not accepted").into());
        }
        "Removed"
    } else {
        values.clear();
        for value in input
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            if !values.iter().any(|existing| existing == value) {
                values.push(value.into());
            }
        }
        "Set"
    };
    workspace.set_string_list_option(key, values.clone())?;
    println!("{action} acceptable {noun}s: {}", values.join(", "));
    Ok(())
}

fn write_nix_checksums(root: &Path, output: &str) -> Result<usize> {
    let workspace = Workspace::open(root.to_path_buf())?;
    let mut checksums = std::collections::BTreeMap::new();
    let transport = UreqTransport::new();
    for item in workspace.index().files.iter().filter(|entry| {
        entry.metafile
            && entry.alias.is_none()
            && Path::new(&entry.file)
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|name| name == "mods")
    }) {
        let metadata: Mod = toml::from_str(&fs::read_to_string(
            root.join(item.file.replace('/', std::path::MAIN_SEPARATOR_STR)),
        )?)?;
        if !metadata.download.mode.is_empty() && metadata.download.mode != "url" {
            eprintln!(
                "warning: {} uses {} download mode and was skipped",
                metadata.name, metadata.download.mode
            );
            continue;
        }
        if metadata.download.url.is_empty() {
            eprintln!(
                "warning: {} has no download URL and was skipped",
                metadata.name
            );
            continue;
        }
        let sha256 = if metadata.download.hash_format == "sha256" {
            metadata.download.hash.clone()
        } else if let Some(hash) = metadata.download.extra_hashes.get("sha256") {
            hash.clone()
        } else {
            let bytes =
                transport.get(packwand_providers::HttpRequest::get(&metadata.download.url))?;
            packwand_pack::hash_bytes(packwand_pack::HashFormat::Sha256, &bytes)
        };
        let name = Path::new(&item.file)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("metadata path has no filename")?
            .to_owned();
        checksums.insert(
            name,
            NixChecksum {
                url: metadata.download.url,
                sha256,
            },
        );
    }
    if checksums.is_empty() {
        return Err(format!("no URL-mode mods found in {}", root.join("mods").display()).into());
    }
    let output = PathBuf::from(output);
    let output = if output.is_absolute() {
        output
    } else {
        root.join(output)
    };
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = serde_json::to_vec_pretty(&checksums)?;
    bytes.push(b'\n');
    fs::write(output, bytes)?;
    Ok(checksums.len())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CrashMod {
    jar_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mod_id: Option<String>,
    name: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    curse_forge_hash: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modrinth_hash: Option<String>,
}

fn run_script(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let pack: packwand_pack::Pack = toml::from_str(&fs::read_to_string(root.join("pack.toml"))?)?;
    let name = required(args, "script")?;
    let script = pack
        .scripts
        .get(name)
        .ok_or_else(|| format!("script {name:?} not found"))?;
    println!("Running script {name:?}: {script}");
    #[cfg(windows)]
    let mut process = {
        let mut command = std::process::Command::new("cmd");
        command.args(["/C", script]);
        command
    };
    #[cfg(not(windows))]
    let mut process = {
        let mut command = std::process::Command::new("sh");
        command.args(["-c", script]);
        command
    };
    let status = process.current_dir(root).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("script {name:?} failed with {status}").into())
    }
}

fn provider_command(provider: ProviderKind, args: &ArgMatches) -> Result {
    let Some(("add", sub)) = args.subcommand() else {
        return Err(format!("{} requires the add subcommand", provider.name()).into());
    };
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
        .cloned();
    request.version_filename = sub.get_one::<String>("version-filename").cloned();
    let instance = sub.get_one::<String>("instance").cloned();
    let resolved = resolve_provider(provider, &request, instance)?;
    let path = resolved.metadata_path();
    Workspace::open(root)?.add_resolved(resolved, false)?;
    println!("added {path}");
    Ok(())
}

fn platform_command(provider: ProviderKind, args: &ArgMatches) -> Result {
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

fn export_local(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let filter = args.get_one::<String>("pack-name").map(String::as_str);
    if root.join("pack.toml").is_file() {
        let format = if root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("-cf"))
        {
            ExportFormat::CurseForge
        } else {
            ExportFormat::Modrinth
        };
        let artifact = export_pack(&root, format, None::<&Path>, ExportOptions::default())?;
        println!("exported {}", artifact.path.display());
        return Ok(());
    }
    let projects = packwand_workspace::discover(&root)?;
    let mut built = 0usize;
    for project in projects.into_iter().filter(|project| {
        filter.is_none_or(|value| {
            project.manifest.id == value
                || project
                    .root
                    .to_string_lossy()
                    .replace('\\', "/")
                    .ends_with(value)
        })
    }) {
        let output_dir = project.root.join("build");
        fs::create_dir_all(&output_dir)?;
        for subdir in project.subdirs {
            let name = subdir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("pack");
            let format = if name.ends_with("-cf") {
                ExportFormat::CurseForge
            } else {
                ExportFormat::Modrinth
            };
            let output = output_dir.join(format!(
                "{}-{}.{}",
                project.manifest.id,
                name,
                format.extension()
            ));
            let artifact = export_pack(&subdir, format, Some(&output), ExportOptions::default())?;
            println!("exported {}", artifact.path.display());
            built += 1;
        }
    }
    if built == 0 {
        Err("no exportable pack variants matched".into())
    } else {
        println!("built {built} archive(s)");
        Ok(())
    }
}

fn publish_command(args: &ArgMatches) -> Result {
    match args.subcommand() {
        Some(("list", sub)) => {
            let manifests = strings(sub, "manifests")
                .into_iter()
                .map(absolute)
                .collect::<Result<Vec<_>>>()?;
            if manifests.is_empty() {
                return Err("publish list requires manifest path(s)".into());
            }
            println!(
                "{}",
                serde_json::to_string(
                    &packwand_build::list_publish_targets(manifests)
                        .map_err(|error| error.to_string())?,
                )?
            );
            Ok(())
        }
        Some(("build", sub)) => {
            let manifest = absolute(required(sub, "manifest")?)?;
            let target = packwand_build::build_publish_target(
                manifest,
                sub.get_one::<String>("variant").map(String::as_str),
            )
            .map_err(|error| error.to_string())?;
            println!("{}", serde_json::to_string_pretty(&target)?);
            Ok(())
        }
        Some(("upload", sub)) => {
            let manifest = absolute(required(sub, "manifest")?)?;
            let changelog = sub
                .get_one::<String>("changelog-file")
                .map(absolute)
                .transpose()?;
            let report = packwand_build::upload_publish_target(
                manifest,
                sub.get_one::<String>("variant").map(String::as_str),
                sub.get_flag("live"),
                changelog.as_deref(),
            )
            .map_err(|error| error.to_string())?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        Some(("verify", sub)) => {
            let manifest = absolute(required(sub, "manifest")?)?;
            let found = packwand_build::verify_publish_target(
                manifest,
                sub.get_one::<String>("variant").map(String::as_str),
                8,
                std::time::Duration::from_secs(15),
            )
            .map_err(|error| error.to_string())?;
            if found {
                println!("publish target is live");
                Ok(())
            } else {
                Err("publish target was not found after verification retries".into())
            }
        }
        Some(("plan", sub)) => {
            let root = std::env::current_dir()?;
            if !sub.get_flag("no-validate") {
                let report = packwand_diagnostics::validate_projects(&root)?;
                if !report.valid() {
                    return Err(format!(
                        "manifest validation failed with {} issue(s)",
                        report.issues.len()
                    )
                    .into());
                }
            }
            let from = sub
                .get_one::<String>("from")
                .map(String::as_str)
                .unwrap_or("HEAD^");
            let to = required(sub, "to")?;
            let output = std::process::Command::new("git")
                .args(["diff", "--name-only", &format!("{from}..{to}")])
                .current_dir(&root)
                .output()?;
            if !output.status.success() {
                return Err(format!(
                    "git diff failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                )
                .into());
            }
            let pack_filter = sub.get_one::<String>("pack").map(String::as_str);
            let changed = String::from_utf8(output.stdout)?
                .lines()
                .filter_map(|line| {
                    let mut parts = line.split('/');
                    Some((parts.next()?.to_owned(), parts.next()?.to_owned()))
                })
                .collect::<std::collections::BTreeSet<_>>();
            let manifests = packwand_workspace::discover(&root)?
                .into_iter()
                .filter(|project| {
                    pack_filter.is_none_or(|wanted| {
                        project.manifest.id == wanted
                            || project.root.file_name().is_some_and(|name| name == wanted)
                    })
                })
                .filter(|project| {
                    let directory = project
                        .root
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    pack_filter.is_some()
                        || changed.contains(&(project.category.clone(), directory))
                })
                .map(|project| project.root.join("manifest.json"))
                .collect::<Vec<_>>();
            let plan = packwand_build::list_publish_targets(manifests)
                .map_err(|error| error.to_string())?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            Ok(())
        }
        _ => Err("publish requires plan, list, build, upload, or verify".into()),
    }
}

fn release_channels(minimum: Option<&str>) -> Vec<ReleaseChannel> {
    match minimum {
        Some("release") => vec![ReleaseChannel::Release],
        Some("beta") => vec![ReleaseChannel::Release, ReleaseChannel::Beta],
        _ => vec![
            ReleaseChannel::Release,
            ReleaseChannel::Beta,
            ReleaseChannel::Alpha,
        ],
    }
}

fn resolve_provider(
    provider: ProviderKind,
    request: &ResolveRequest,
    instance: Option<String>,
) -> Result<ResolvedProject> {
    let transport = UreqTransport::new();
    Ok(match provider {
        ProviderKind::Modrinth => ModrinthClient::new(transport).resolve(request)?,
        ProviderKind::CurseForge => {
            CurseForgeClient::new(transport, configured_api_key()).resolve(request)?
        }
        ProviderKind::GitHub => {
            GitHubClient::new(transport, std::env::var("GITHUB_TOKEN").unwrap_or_default())
                .resolve(request)?
        }
        ProviderKind::Forgejo => match instance {
            Some(instance) => ForgejoClient::for_instance(
                transport,
                instance,
                std::env::var("FORGEJO_TOKEN").unwrap_or_default(),
            )
            .resolve(request)?,
            None => ForgejoClient::new(
                transport,
                std::env::var("FORGEJO_TOKEN").unwrap_or_default(),
            )
            .resolve(request)?,
        },
        ProviderKind::GitLab => match instance {
            Some(instance) => GitLabClient::for_instance(
                transport,
                instance,
                std::env::var("GITLAB_TOKEN").unwrap_or_default(),
            )
            .resolve(request)?,
            None => GitLabClient::new(transport, std::env::var("GITLAB_TOKEN").unwrap_or_default())
                .resolve(request)?,
        },
    })
}

fn url_command(args: &ArgMatches) -> Result {
    let Some(("add", sub)) = args.subcommand() else {
        return Err("url requires add".into());
    };
    let name = required(sub, "name")?;
    let raw_url = required(sub, "url")?;
    let parsed = url::Url::parse(raw_url)?;
    let filename = parsed
        .path_segments()
        .and_then(Iterator::last)
        .filter(|value| !value.is_empty())
        .ok_or("download URL has no filename")?
        .to_owned();
    let bytes = UreqTransport::new().get(packwand_providers::HttpRequest::get(raw_url))?;
    let slug = sub
        .get_one::<String>("meta-name")
        .cloned()
        .unwrap_or_else(|| slugify(name));
    let metadata = Mod {
        name: name.to_owned(),
        filename,
        side: "both".into(),
        download: packwand_pack::Download {
            url: raw_url.to_owned(),
            hash_format: "sha512".into(),
            hash: packwand_pack::hash_bytes(packwand_pack::HashFormat::Sha512, &bytes),
            size: bytes.len() as u64,
            ..packwand_pack::Download::default()
        },
        ..Mod::default()
    };
    let path = format!("mods/{}.pw.toml", slug.trim_end_matches(".pw.toml"));
    Workspace::open(std::env::current_dir()?)?.add_metadata(&path, metadata, false)?;
    println!("added {path}");
    Ok(())
}

fn migrate(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let mut workspace = Workspace::open(root)?;
    match args.subcommand() {
        Some(("format", _)) => {
            let (old, new) = workspace.migrate_format()?;
            println!("pack format: {old} -> {new}");
            Ok(())
        }
        Some(("minecraft", sub)) => {
            let version = required(sub, "version")?;
            let old = workspace
                .set_version("minecraft", version)?
                .unwrap_or_else(|| "<unset>".into());
            println!("minecraft: {old} -> {version}");
            Ok(())
        }
        Some(("loader", sub)) => {
            let requested = required(sub, "version")?;
            let loaders = workspace
                .pack()
                .versions
                .keys()
                .filter(|key| key.as_str() != "minecraft")
                .cloned()
                .collect::<Vec<_>>();
            if loaders.is_empty() {
                return Err("pack has no configured loader".into());
            }
            for loader in loaders {
                let version = if matches!(requested, "latest" | "recommended") {
                    let minecraft = workspace
                        .pack()
                        .versions
                        .get("minecraft")
                        .ok_or("pack has no Minecraft version")?;
                    resolve_loader_version(&loader, minecraft, requested)?
                } else {
                    requested.to_owned()
                };
                let old = workspace
                    .set_version(&loader, &version)?
                    .unwrap_or_else(|| "<unset>".into());
                println!("{loader}: {old} -> {version}");
            }
            Ok(())
        }
        _ => Err("migrate requires format, minecraft, or loader".into()),
    }
}

fn resolve_loader_version(loader: &str, minecraft: &str, channel: &str) -> Result<String> {
    let transport = UreqTransport::new();
    let value = match loader {
        "fabric" => {
            let bytes = transport.get(packwand_providers::HttpRequest::get(format!(
                "https://meta.fabricmc.net/v2/versions/loader/{minecraft}"
            )))?;
            serde_json::from_slice::<serde_json::Value>(&bytes)?
        }
        "quilt" => {
            let bytes = transport.get(packwand_providers::HttpRequest::get(format!(
                "https://meta.quiltmc.org/v3/versions/loader/{minecraft}"
            )))?;
            serde_json::from_slice::<serde_json::Value>(&bytes)?
        }
        "forge" => {
            let bytes = transport.get(packwand_providers::HttpRequest::get(
                "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json",
            ))?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)?;
            let promos = value
                .get("promos")
                .and_then(serde_json::Value::as_object)
                .ok_or("Forge promotions response has no promos")?;
            let key = format!(
                "{minecraft}-{}",
                if channel == "recommended" {
                    "recommended"
                } else {
                    "latest"
                }
            );
            return promos
                .get(&key)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| format!("Forge has no {channel} loader for {minecraft}").into());
        }
        "neoforge" => {
            let bytes = transport.get(packwand_providers::HttpRequest::get(
                "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge",
            ))?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)?;
            let versions = value
                .get("versions")
                .and_then(serde_json::Value::as_array)
                .ok_or("NeoForge version response has no versions")?;
            let prefix = neoforge_version_prefix(minecraft);
            return versions
                .iter()
                .filter_map(serde_json::Value::as_str)
                .rfind(|version| {
                    prefix
                        .as_ref()
                        .is_none_or(|prefix| version.starts_with(prefix))
                })
                .map(str::to_owned)
                .ok_or_else(|| format!("NeoForge has no loader for {minecraft}").into());
        }
        _ => return Err(format!("unsupported loader {loader:?}").into()),
    };
    let releases = value
        .as_array()
        .ok_or_else(|| format!("{loader} loader response is not an array"))?;
    releases
        .iter()
        .filter(|release| {
            channel != "recommended"
                || release
                    .get("loader")
                    .and_then(|loader| loader.get("stable"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true)
        })
        .find_map(|release| {
            release
                .get("loader")
                .and_then(|loader| loader.get("version"))
                .and_then(serde_json::Value::as_str)
        })
        .map(str::to_owned)
        .ok_or_else(|| format!("{loader} has no {channel} loader for {minecraft}").into())
}

fn neoforge_version_prefix(minecraft: &str) -> Option<String> {
    let mut parts = minecraft.split('.');
    let major = parts.next()?;
    let minor = parts.next()?;
    let patch = parts.next();
    if major == "1" {
        Some(match patch {
            Some(patch) => format!("{minor}.{patch}."),
            None => format!("{minor}."),
        })
    } else {
        Some(format!("{major}."))
    }
}

#[derive(Serialize)]
struct UpdateRecord {
    path: String,
    name: String,
    provider: String,
    old_filename: String,
    new_filename: String,
    changed: bool,
    applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn update_command(args: &ArgMatches) -> Result {
    if args.get_one::<String>("report").is_some() && !args.get_flag("all") {
        return Err("--report requires --all".into());
    }
    let root = std::env::current_dir()?;
    let records = update_workspace(
        &root,
        args.get_one::<String>("name").map(String::as_str),
        args.get_flag("all"),
        args.get_flag("dry-run"),
    )?;
    let failures = records
        .iter()
        .filter(|record| record.error.is_some() && record.error.as_deref() != Some("pinned"))
        .count();
    let json = serde_json::to_vec_pretty(&records)?;
    if let Some(report) = args.get_one::<String>("report") {
        let path = absolute(report)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &json)?;
    }
    if args.get_flag("json") {
        println!("{}", String::from_utf8(json)?);
    } else {
        print_update_records(&records);
    }
    if failures > 0 {
        Err(format!("{failures} update(s) failed").into())
    } else {
        Ok(())
    }
}

fn update_workspace(
    root: &Path,
    selected: Option<&str>,
    all: bool,
    dry_run: bool,
) -> Result<Vec<UpdateRecord>> {
    let mut workspace = Workspace::open(root.to_path_buf())?;
    let paths = if all {
        workspace
            .index()
            .files
            .iter()
            .filter(|entry| entry.metafile && entry.alias.is_none())
            .map(|entry| entry.file.clone())
            .collect::<Vec<_>>()
    } else if let Some(name) = selected {
        vec![metadata_path(&workspace, name)?]
    } else {
        return Err("provide a metadata name or use --all".into());
    };
    let mut records = Vec::new();
    for path in paths {
        let source = root.join(path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let metadata: Mod = toml::from_str(&fs::read_to_string(&source)?)?;
        let name = metadata.name.clone();
        let old_filename = metadata.filename.clone();
        if metadata.pin {
            records.push(UpdateRecord {
                path,
                name,
                provider: String::new(),
                old_filename: old_filename.clone(),
                new_filename: old_filename,
                changed: false,
                applied: false,
                error: Some("pinned".into()),
            });
            continue;
        }
        let (provider, mut request, instance) = match update_request(&metadata, workspace.pack()) {
            Ok(value) => value,
            Err(error) => {
                records.push(UpdateRecord {
                    path,
                    name,
                    provider: String::new(),
                    old_filename: old_filename.clone(),
                    new_filename: old_filename,
                    changed: false,
                    applied: false,
                    error: Some(error.to_string()),
                });
                continue;
            }
        };
        request.channels = vec![
            ReleaseChannel::Release,
            ReleaseChannel::Beta,
            ReleaseChannel::Alpha,
        ];
        match resolve_provider(provider, &request, instance) {
            Ok(resolved) => {
                let new_filename = resolved.version.file.filename.clone();
                let changed = installed_version(&metadata, provider)
                    .is_none_or(|current| current != resolved.version.id);
                let applied = changed && !dry_run;
                let error = if applied {
                    workspace
                        .update_resolved(&path, resolved)
                        .err()
                        .map(|error| error.to_string())
                } else {
                    None
                };
                records.push(UpdateRecord {
                    path,
                    name,
                    provider: provider.name().into(),
                    old_filename,
                    new_filename,
                    changed,
                    applied: applied && error.is_none(),
                    error,
                });
            }
            Err(error) => records.push(UpdateRecord {
                path,
                name,
                provider: provider.name().into(),
                old_filename: old_filename.clone(),
                new_filename: old_filename,
                changed: false,
                applied: false,
                error: Some(error.to_string()),
            }),
        }
    }
    Ok(records)
}

fn print_update_records(records: &[UpdateRecord]) {
    for record in records {
        if let Some(error) = &record.error {
            println!("{}: skipped ({error})", record.name);
        } else if record.changed {
            println!(
                "{}: {} -> {}{}",
                record.name,
                record.old_filename,
                record.new_filename,
                if record.applied { "" } else { " (dry run)" }
            );
        } else {
            println!("{}: up to date", record.name);
        }
    }
}

fn update_request(
    metadata: &Mod,
    pack: &packwand_pack::Pack,
) -> Result<(ProviderKind, ResolveRequest, Option<String>)> {
    let (provider, table, project) = if let Some(table) = metadata.update.get("modrinth") {
        (
            ProviderKind::Modrinth,
            table,
            table
                .get("mod-id")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .to_owned(),
        )
    } else if let Some(table) = metadata.update.get("curseforge") {
        (
            ProviderKind::CurseForge,
            table,
            table
                .get("project-id")
                .and_then(toml::Value::as_integer)
                .map(|value| value.to_string())
                .unwrap_or_default(),
        )
    } else if let Some(table) = metadata.update.get("github") {
        (
            ProviderKind::GitHub,
            table,
            table
                .get("slug")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .to_owned(),
        )
    } else if let Some(table) = metadata.update.get("forgejo") {
        (
            ProviderKind::Forgejo,
            table,
            table
                .get("slug")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .to_owned(),
        )
    } else if let Some(table) = metadata.update.get("gitlab") {
        (
            ProviderKind::GitLab,
            table,
            table
                .get("slug")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .to_owned(),
        )
    } else {
        return Err("metadata has no supported update provider".into());
    };
    if project.is_empty() {
        return Err(format!("{} update metadata has no project id", provider.name()).into());
    }
    let mut request = ResolveRequest::new(project);
    request.game_versions = pack
        .versions
        .get("minecraft")
        .cloned()
        .into_iter()
        .collect();
    request.loaders = pack
        .versions
        .keys()
        .filter(|key| key.as_str() != "minecraft")
        .cloned()
        .collect();
    request.branch = table
        .get("branch")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    request.asset_pattern = table
        .get("regex")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    let instance = table
        .get("instance")
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    Ok((provider, request, instance))
}

fn installed_version(metadata: &Mod, provider: ProviderKind) -> Option<String> {
    let table = metadata.update.get(provider.name())?;
    let key = match provider {
        ProviderKind::Modrinth => "version",
        ProviderKind::CurseForge => "file-id",
        ProviderKind::Forgejo | ProviderKind::GitHub | ProviderKind::GitLab => "tag",
    };
    table.get(key).and_then(|value| {
        value
            .as_str()
            .map(str::to_owned)
            .or_else(|| value.as_integer().map(|value| value.to_string()))
    })
}

fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;
    for character in name.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            separator = false;
        } else if !separator && !slug.is_empty() {
            slug.push('-');
            separator = true;
        }
    }
    slug.trim_end_matches('-').to_owned()
}

fn lint(args: &ArgMatches) -> Result {
    let files = strings(args, "files");
    let report = if files.is_empty() {
        packwand_diagnostics::lint_workspace(std::env::current_dir()?)
    } else {
        let mut report = packwand_diagnostics::ValidationReport::default();
        for file in files {
            report.checked += 1;
            report.issues.extend(packwand_diagnostics::lint_file(file));
        }
        report
    };
    for issue in &report.issues {
        eprintln!("{}: {}", issue.path.display(), issue.message);
    }
    if report.valid() {
        println!("{} file(s) lint clean", report.checked);
        Ok(())
    } else {
        Err(format!(
            "{} of {} file(s) failed lint",
            report.issues.len(),
            report.checked
        )
        .into())
    }
}

fn content_lint_command(args: &ArgMatches) -> Result {
    let current = std::env::current_dir()?;
    let mut roots = strings(args, "pack-dirs")
        .into_iter()
        .map(absolute)
        .collect::<Result<Vec<_>>>()?;
    if args.get_flag("all") {
        roots.extend(
            packwand_workspace::discover(&current)?
                .into_iter()
                .filter(|project| {
                    matches!(project.category.as_str(), "datapacks" | "resourcepacks")
                })
                .map(|project| project.root),
        );
    }
    if roots.is_empty() {
        roots.push(current);
    }
    roots.sort();
    roots.dedup();
    let mut report = packwand_diagnostics::ValidationReport::default();
    for root in roots {
        let next = packwand_diagnostics::content_lint(root);
        report.checked += next.checked;
        report.issues.extend(next.issues);
    }
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for issue in &report.issues {
            eprintln!(
                "{}: {:?}: {}",
                issue.path.display(),
                issue.severity,
                issue.message
            );
        }
        println!(
            "checked {} content file(s); {} issue(s)",
            report.checked,
            report.issues.len()
        );
    }
    if report.valid() {
        Ok(())
    } else {
        Err(format!(
            "content lint found {} error(s)",
            report
                .issues
                .iter()
                .filter(|issue| matches!(issue.severity, packwand_diagnostics::Severity::Error))
                .count()
        )
        .into())
    }
}

#[derive(Debug, Serialize)]
struct PreflightIssue {
    level: &'static str,
    #[serde(skip_serializing_if = "String::is_empty")]
    path: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct PreflightStep {
    name: &'static str,
    errors: usize,
    warnings: usize,
    issues: Vec<PreflightIssue>,
}

#[derive(Debug, Serialize)]
struct PreflightResult {
    dir: String,
    steps: Vec<PreflightStep>,
    errors: usize,
    warnings: usize,
    ok: bool,
}

fn preflight(args: &ArgMatches) -> Result {
    let root = args
        .get_one::<String>("dir")
        .map(absolute)
        .transpose()?
        .unwrap_or(std::env::current_dir()?);
    let report = run_preflight(&root);
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for step in &report.steps {
            println!(
                "{}: {} ({} error(s), {} warning(s))",
                step.name,
                if step.errors == 0 { "PASS" } else { "FAIL" },
                step.errors,
                step.warnings
            );
            for issue in &step.issues {
                let path = if issue.path.is_empty() {
                    String::new()
                } else {
                    format!("{}: ", issue.path)
                };
                eprintln!("  {}: {path}{}", issue.level, issue.message);
            }
        }
    }
    if report.ok {
        Ok(())
    } else {
        Err(format!("preflight failed with {} error(s)", report.errors).into())
    }
}

fn run_preflight(root: &Path) -> PreflightResult {
    let mut steps = Vec::new();

    let manifest_path = if root.join("manifest.json").is_file() {
        root.join("manifest.json")
    } else {
        root.parent()
            .map(|parent| parent.join("manifest.json"))
            .unwrap_or_else(|| root.join("manifest.json"))
    };
    let manifest_issues = match fs::read(&manifest_path) {
        Ok(bytes) => match serde_json::from_slice::<Manifest>(&bytes) {
            Ok(manifest) => {
                let mut issues = Vec::new();
                for (field, value) in [
                    ("id", manifest.id.as_str()),
                    ("name", manifest.name.as_str()),
                    ("type", manifest.project_type.as_str()),
                ] {
                    if value.trim().is_empty() {
                        issues.push(PreflightIssue {
                            level: "error",
                            path: manifest_path.to_string_lossy().into_owned(),
                            message: format!("manifest is missing required field {field:?}"),
                        });
                    }
                }
                if manifest.version.trim().is_empty() {
                    issues.push(PreflightIssue {
                        level: "warning",
                        path: manifest_path.to_string_lossy().into_owned(),
                        message: "manifest has no version".into(),
                    });
                }
                issues
            }
            Err(error) => vec![PreflightIssue {
                level: "error",
                path: manifest_path.to_string_lossy().into_owned(),
                message: error.to_string(),
            }],
        },
        Err(_) => vec![PreflightIssue {
            level: "error",
            path: manifest_path.to_string_lossy().into_owned(),
            message: "no manifest.json found in the pack directory or its parent".into(),
        }],
    };
    steps.push(preflight_step("manifest", manifest_issues));

    let syntax = packwand_diagnostics::lint_workspace(root);
    steps.push(preflight_step(
        "syntax",
        syntax
            .issues
            .into_iter()
            .map(|issue| PreflightIssue {
                level: match issue.severity {
                    packwand_diagnostics::Severity::Error => "error",
                    packwand_diagnostics::Severity::Warning => "warning",
                },
                path: issue.path.to_string_lossy().into_owned(),
                message: issue.message,
            })
            .collect(),
    ));

    let mut reference_issues = packwand_diagnostics::content_lint(root)
        .issues
        .into_iter()
        .map(|issue| PreflightIssue {
            level: match issue.severity {
                packwand_diagnostics::Severity::Error => "error",
                packwand_diagnostics::Severity::Warning => "warning",
            },
            path: issue.path.to_string_lossy().into_owned(),
            message: issue.message,
        })
        .collect::<Vec<_>>();
    if root.join("pack.toml").is_file() {
        match Workspace::open(root.to_path_buf()) {
            Ok(workspace) => {
                for entry in &workspace.index().files {
                    let path = root.join(entry.file.replace('/', std::path::MAIN_SEPARATOR_STR));
                    if !path.is_file() {
                        reference_issues.push(PreflightIssue {
                            level: "error",
                            path: entry.file.clone(),
                            message: "indexed path is missing".into(),
                        });
                    }
                }
            }
            Err(error) => reference_issues.push(PreflightIssue {
                level: "error",
                path: root.join("pack.toml").to_string_lossy().into_owned(),
                message: error.to_string(),
            }),
        }
    }
    if let Err(error) = packwand_diagnostics::build_all_registries(root) {
        reference_issues.push(PreflightIssue {
            level: "error",
            path: root.to_string_lossy().into_owned(),
            message: format!("registry build failed: {error}"),
        });
    }
    steps.push(preflight_step("references", reference_issues));

    let errors = steps.iter().map(|step| step.errors).sum();
    let warnings = steps.iter().map(|step| step.warnings).sum();
    PreflightResult {
        dir: root.to_string_lossy().replace('\\', "/"),
        steps,
        errors,
        warnings,
        ok: errors == 0,
    }
}

fn preflight_step(name: &'static str, issues: Vec<PreflightIssue>) -> PreflightStep {
    PreflightStep {
        name,
        errors: issues.iter().filter(|issue| issue.level == "error").count(),
        warnings: issues
            .iter()
            .filter(|issue| issue.level == "warning")
            .count(),
        issues,
    }
}

fn registry_command(args: &ArgMatches) -> Result {
    use std::str::FromStr as _;

    let root = args
        .get_one::<String>("dir")
        .map(absolute)
        .transpose()?
        .unwrap_or(std::env::current_dir()?);
    let kind = required(args, "kind")?;
    let registries = if kind.eq_ignore_ascii_case("all") {
        packwand_diagnostics::build_all_registries(&root)?
    } else {
        vec![packwand_diagnostics::build_registry(
            &root,
            packwand_diagnostics::RegistryKind::from_str(kind)?,
        )?]
    };
    if args.get_flag("json") {
        if registries.len() == 1 {
            println!("{}", serde_json::to_string_pretty(&registries[0])?);
        } else {
            println!("{}", serde_json::to_string_pretty(&registries)?);
        }
    } else {
        for registry in registries {
            println!(
                "{} registry: {} source(s), {} entries, version {:.12}",
                registry.kind,
                registry.sources.len(),
                registry.entries.len(),
                registry.version
            );
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct LocalCiStage {
    name: &'static str,
    ok: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    message: String,
}

fn ci_local(args: &ArgMatches) -> Result {
    let root = args
        .get_one::<String>("dir")
        .map(absolute)
        .transpose()?
        .unwrap_or(std::env::current_dir()?);
    let preflight = run_preflight(&root);
    let registry = packwand_diagnostics::build_all_registries(&root);
    let stages = vec![
        LocalCiStage {
            name: "preflight",
            ok: preflight.ok,
            message: format!(
                "{} error(s), {} warning(s)",
                preflight.errors, preflight.warnings
            ),
        },
        LocalCiStage {
            name: "registry",
            ok: registry.is_ok(),
            message: registry
                .err()
                .map(|error| error.to_string())
                .unwrap_or_default(),
        },
    ];
    let ok = stages.iter().all(|stage| stage.ok);
    if args.get_flag("json") {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"dir":root.to_string_lossy().replace('\\', "/"),"stages":stages,"ok":ok})
            )?
        );
    } else {
        for stage in &stages {
            println!(
                "{}: {} {}",
                stage.name,
                if stage.ok { "PASS" } else { "FAIL" },
                stage.message
            );
        }
    }
    if ok {
        Ok(())
    } else {
        Err("localized CI failed".into())
    }
}

fn validate(args: &ArgMatches) -> Result {
    if !args.get_flag("all") && strings(args, "manifests").is_empty() {
        return Err("provide manifest path(s) or use --all".into());
    }
    let report = packwand_diagnostics::validate_projects(std::env::current_dir()?)?;
    for issue in &report.issues {
        eprintln!("{}: {}", issue.path.display(), issue.message);
    }
    if report.valid() {
        println!("all {} manifest(s) OK", report.checked);
        Ok(())
    } else {
        Err(format!("{} validation issue(s)", report.issues.len()).into())
    }
}

fn parity(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let reports = if strings(args, "pack-dirs").is_empty() {
        packwand_diagnostics::parity_workspace(root)?
    } else {
        let mut reports = Vec::new();
        for path in strings(args, "pack-dirs") {
            reports.extend(packwand_diagnostics::parity_project(
                &packwand_workspace::read_project(&root, &absolute(path)?)?,
            ));
        }
        reports
    };
    let drifted = reports.iter().filter(|report| report.drifted()).count();
    if args.get_flag("json") {
        println!("{}", serde_json::to_string(&reports)?);
    } else {
        for report in &reports {
            println!(
                "{}/{}: {}",
                report.pack,
                report.variant,
                if report.drifted() {
                    "drifted"
                } else if report.missing_side.is_some() {
                    "single-platform"
                } else {
                    "in sync"
                }
            );
        }
        println!("{drifted} of {} report(s) drifted", reports.len());
    }
    if args.get_flag("strict") && drifted > 0 {
        Err(format!("{drifted} variant pair(s) drifted").into())
    } else {
        Ok(())
    }
}

fn utils(args: &ArgMatches) -> Result {
    match args.subcommand() {
        Some(("commands", sub)) => {
            let mut entries = Vec::new();
            collect_commands(&cli::build(), "", &mut entries);
            if sub.get_flag("json") {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            } else {
                for entry in entries {
                    println!("{}", entry.path);
                }
            }
            Ok(())
        }
        Some(("markdown", sub)) => {
            let directory = PathBuf::from(required(sub, "dir")?);
            fs::create_dir_all(&directory)?;
            let mut entries = Vec::new();
            collect_commands(&cli::build(), "", &mut entries);
            let mut markdown = String::from("# Packwand command reference\n\n");
            for entry in entries {
                markdown.push_str(&format!(
                    "## `packwand {}`\n\n{}\n\n",
                    entry.path, entry.summary
                ));
            }
            fs::write(directory.join("packwand.md"), markdown)?;
            println!("wrote {}", directory.join("packwand.md").display());
            Ok(())
        }
        _ => Err("utils requires commands or markdown".into()),
    }
}

#[derive(Serialize)]
struct CommandEntry {
    path: String,
    use_line: String,
    summary: String,
    runnable: bool,
}

fn collect_commands(parent: &clap::Command, prefix: &str, out: &mut Vec<CommandEntry>) {
    for sub in parent.get_subcommands() {
        if sub.get_name() == "help" {
            continue;
        }
        let path = if prefix.is_empty() {
            sub.get_name().to_owned()
        } else {
            format!("{prefix} {}", sub.get_name())
        };
        out.push(CommandEntry {
            path: path.clone(),
            use_line: sub.get_name().to_owned(),
            summary: sub.get_about().map(ToString::to_string).unwrap_or_default(),
            runnable: !sub.has_subcommands(),
        });
        collect_commands(sub, &path, out);
    }
}

fn required<'a>(args: &'a ArgMatches, name: &str) -> Result<&'a str> {
    args.get_one::<String>(name)
        .map(String::as_str)
        .ok_or_else(|| format!("missing {name}").into())
}

fn strings(args: &ArgMatches, name: &str) -> Vec<String> {
    args.get_many::<String>(name)
        .map(|values| values.cloned().collect())
        .unwrap_or_default()
}

fn comma_values(args: &ArgMatches, name: &str) -> Vec<String> {
    args.get_one::<String>(name)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn print_value(value: &serde_json::Value, json: bool) {
    if json || !value.is_string() {
        println!("{}", value);
    } else if let Some(value) = value.as_str() {
        println!("{value}");
    }
}

fn absolute(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    Ok(if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    })
}

#[cfg(test)]
mod tests {
    use super::requested_group_path;
    use crate::cli;

    #[test]
    fn parent_commands_route_to_their_own_help() {
        let root = cli::build();
        let workspace = root
            .clone()
            .try_get_matches_from(["packwand", "workspace"])
            .unwrap();
        assert_eq!(
            requested_group_path(&root, &workspace),
            Some(vec!["workspace".into()])
        );

        let provider = root
            .clone()
            .try_get_matches_from(["packwand", "workspace", "mr"])
            .unwrap();
        assert_eq!(
            requested_group_path(&root, &provider),
            Some(vec!["workspace".into(), "mr".into()])
        );

        let leaf = root
            .clone()
            .try_get_matches_from(["packwand", "workspace", "status"])
            .unwrap();
        assert_eq!(requested_group_path(&root, &leaf), None);
    }
}
