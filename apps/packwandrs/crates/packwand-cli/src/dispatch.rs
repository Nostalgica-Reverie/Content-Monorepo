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

mod automation;
mod batch;
mod cache;
mod diagnostics;
mod diff;
mod inspect;
mod json_tools;
mod metadata;
mod migration;
mod providers;
mod publish;

use automation::automation;
use batch::{batch_command, packs};
use cache::cache_command;
use diagnostics::{
    ci_local, content_lint_command, lint, parity, preflight, registry_command, validate,
};
use diff::diff_command;
use inspect::{deps, explain};
use json_tools::{json, modlist};
use metadata::{
    add_workspace, import_archive, init_pack, list, metadata_path, new_project, pin, port, refresh,
    rehash, remove,
};
use migration::{migrate, resolve_loader_version};
use providers::{platform_command, provider_command};
use publish::{export_local, publish_command};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub fn run() -> Result {
    let mut root = cli::build();
    let matches = root.clone().get_matches();
    // `--jobs` is global, so it is recorded once here rather than threaded
    // through every handler. 0 means "decide for me".
    packwand_parallel::configure(packwand_parallel::Jobs::new(
        matches.get_one::<usize>("jobs").copied().unwrap_or(0),
    ));
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
        Some(("batch", args)) => batch_command(args),
        Some(("automation", args)) => automation(args),
        Some(("cache", args)) => cache_command(args, &matches),
        Some(("api", args)) => crate::api_cmd::run(args),
        Some(("doctor", args)) => doctor(args),
        Some(("content-lint", args)) => content_lint_command(args),
        Some(("ci-local", args)) => ci_local(args),
        Some(("deps", args)) => deps(args),
        Some(("explain", args)) => explain(args),
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

fn doctor(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let mut checks = Vec::new();
    checks.push(DoctorCheck::new(
        "repo-root",
        root.join(".git").exists() || root.join("modpacks").is_dir(),
        root.display().to_string(),
    ));

    checks.push(tool_check(
        "git",
        true,
        "change detection, changelogs, sync anchoring",
    ));
    checks.push(DoctorCheck::new(
        "packwand",
        true,
        std::env::current_exe()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|error| error.to_string()),
    ));
    for (tool, why) in [
        ("java", "only needed for 'packwand test'"),
        ("zip", "datapack/resourcepack builds via the publisher"),
        (
            "packeater",
            "optimized resource pack builds (plain zip used when absent)",
        ),
    ] {
        checks.push(tool_check(tool, false, why));
    }

    // Walked per category rather than via discover(): doctor has to report a
    // broken manifest, and discover gives up on the first one it cannot
    // parse.
    let mut warnings = Vec::new();
    let mut total = 0usize;
    for category in ["mods", "modpacks", "datapacks", "resourcepacks"] {
        let Ok(entries) = fs::read_dir(root.join(category)) else {
            continue;
        };
        let mut project_roots: Vec<PathBuf> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.join("manifest.json").is_file())
            .collect();
        project_roots.sort();
        let mut count = 0usize;
        for project_root in &project_roots {
            count += 1;
            let manifest_path = project_root.join("manifest.json");
            let manifest = match fs::read(&manifest_path)
                .map_err(|error| error.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<Manifest>(&bytes).map_err(|error| error.to_string())
                }) {
                Ok(manifest) => manifest,
                Err(error) => {
                    checks.push(DoctorCheck::new(
                        format!("manifest/{}", slash_display(project_root, &root)),
                        false,
                        format!("unparsable manifest: {error}"),
                    ));
                    continue;
                }
            };
            if let Some(lifecycle) = manifest.lifecycle.as_deref()
                && !lifecycle.is_empty()
                && !LIFECYCLES.contains(&lifecycle)
            {
                checks.push(DoctorCheck::new(
                    format!("lifecycle/{}", manifest.id),
                    false,
                    format!(
                        "invalid lifecycle {lifecycle:?} (want one of {})",
                        LIFECYCLES.join(", ")
                    ),
                ));
            }
            warnings.extend(legacy_file_warnings(project_root));
            warnings.extend(freeze_drift_warnings(project_root, &manifest));
        }
        if count > 0 {
            total += count;
            checks.push(DoctorCheck::new(
                format!("projects/{category}"),
                true,
                format!("{count} project(s)"),
            ));
        }
    }
    checks.push(DoctorCheck::new(
        "manifests",
        total > 0,
        format!("{total} project(s)"),
    ));
    // Legacy layouts still work, so they are advice rather than failures.
    for warning in warnings {
        checks.push(DoctorCheck::new("advice", true, warning));
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
    name: String,
    ok: bool,
    detail: String,
}

impl DoctorCheck {
    fn new(name: impl Into<String>, ok: bool, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ok,
            detail: detail.into(),
        }
    }
}

/// Resolves a program against PATH the way a shell would, without running it.
/// Probing with `--version` would be wrong here: some of these tools exit
/// non-zero for it, and doctor must not execute a binary just to learn it
/// exists.
fn on_path(name: &str) -> Option<PathBuf> {
    let search = std::env::var_os("PATH")?;
    let extensions: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".into())
            .split(';')
            .filter(|extension| !extension.is_empty())
            .map(str::to_lowercase)
            .collect()
    } else {
        Vec::new()
    };
    for directory in std::env::split_paths(&search) {
        let candidate = directory.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        for extension in &extensions {
            let candidate = directory.join(format!("{name}{extension}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// A required tool missing is a failure; an optional one missing only
/// degrades a specific feature, so it stays `ok` and explains itself.
fn tool_check(name: &str, required: bool, why: &str) -> DoctorCheck {
    match on_path(name) {
        Some(path) => DoctorCheck::new(name, true, path.display().to_string()),
        None if required => DoctorCheck::new(name, false, format!("missing: {why}")),
        None => DoctorCheck::new(name, true, format!("not found (optional: {why})")),
    }
}

const LIFECYCLES: [&str; 4] = ["active", "maintenance", "archived", "eol"];

/// Names a path relative to the workspace root using forward slashes, so
/// check names read the same on every platform.
fn slash_display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Legacy sidecar files that have been folded into manifest.json's
/// "automation" block. They still work, so they are reported as advice
/// rather than failures.
fn legacy_file_warnings(project_root: &Path) -> Vec<String> {
    let mut warnings = Vec::new();
    for legacy in ["opt-out.json", "auto-update-ignore.json"] {
        if project_root.join(legacy).is_file() {
            warnings.push(format!("legacy {legacy} in {}", project_root.display()));
        }
    }
    if let Ok(entries) = fs::read_dir(project_root) {
        let mut subdirs: Vec<PathBuf> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect();
        subdirs.sort();
        for subdir in subdirs {
            if subdir.join("sync-exclude.json").is_file() {
                warnings.push(format!("legacy sync-exclude.json in {}", subdir.display()));
            }
        }
    }
    warnings
}

/// Mods a manifest declares frozen that are not actually pinned in their
/// metadata — the freeze would silently not hold.
fn freeze_drift_warnings(project_root: &Path, manifest: &Manifest) -> Vec<String> {
    let mut warnings = Vec::new();
    let Some(automation) = manifest.automation.as_ref() else {
        return warnings;
    };
    for (subdir, slugs) in &automation.freeze {
        for slug in slugs {
            let metafile = project_root
                .join(subdir)
                .join("mods")
                .join(format!("{slug}.pw.toml"));
            let pinned = fs::read_to_string(&metafile)
                .ok()
                .and_then(|text| toml::from_str::<Mod>(&text).ok())
                .is_some_and(|metadata| metadata.pin);
            if !pinned {
                warnings.push(format!(
                    "freeze drift: {} declared frozen but not pinned",
                    metafile.display()
                ));
            }
        }
    }
    warnings
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
        // Scoped to the pack collection directory, not the whole repository:
        // a repo-wide walk also picks up documentation examples and test
        // fixtures whose index references metafiles that were never checked
        // in, and those are not workspace packs.
        let modpacks = root.join(modpacks_dir());
        if !modpacks.is_dir() {
            return Err(format!(
                "{} is not a directory; run --all from the repository root",
                modpacks.display()
            )
            .into());
        }
        let mut pack_roots = Vec::new();
        for entry in walkdir::WalkDir::new(&modpacks)
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
        // One unusable pack must not abandon the other 48. Failures are
        // reported per pack and the command fails at the end if any did.
        let mut failed = Vec::new();
        for pack_root in &pack_roots {
            let label = pack_root.strip_prefix(&root).unwrap_or(pack_root).display();
            if let Err(error) = write_nix_checksums(pack_root, output) {
                eprintln!("warning: {label}: nix gen failed: {error}");
                failed.push(pack_root.clone());
            }
        }
        println!(
            "generated checksums for {} of {} pack(s)",
            pack_roots.len() - failed.len(),
            pack_roots.len()
        );
        if !failed.is_empty() {
            return Err(format!("nix gen failed for {} pack(s)", failed.len()).into());
        }
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

/// The pack collection directory, overridable for the same reason Go allows
/// it: alternate workspace layouts in CI.
fn modpacks_dir() -> String {
    std::env::var("MODPACKS_DIR")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "modpacks".to_owned())
}

fn write_nix_checksums(root: &Path, output: &str) -> Result<usize> {
    let workspace = Workspace::open(root.to_path_buf())?;
    let mut checksums = std::collections::BTreeMap::new();
    // Hashing a mod means downloading the jar itself, so this needs the
    // transfer-scale client rather than the API one.
    let transport = UreqTransport::for_downloads();
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
    let bytes =
        UreqTransport::for_downloads().get(packwand_providers::HttpRequest::get(raw_url))?;
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

/// The CLI's wire shape for an update record. Deliberately distinct from
/// `packwand_ops::UpdateRecord`, which serializes camelCase for the desktop
/// IPC; the CLI's JSON has always been snake_case and scripts depend on it.
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

impl From<packwand_ops::UpdateRecord> for UpdateRecord {
    fn from(record: packwand_ops::UpdateRecord) -> Self {
        Self {
            path: record.path,
            name: record.name,
            provider: record.provider,
            old_filename: record.old_filename,
            new_filename: record.new_filename,
            changed: record.changed,
            applied: record.applied,
            error: record.error,
        }
    }
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
    let failures = records.iter().filter(|record| is_failure(record)).count();
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
        if records.is_empty() && args.get_flag("all") {
            eprintln!(
                "warning: no mod metadata found in index — if mods exist on disk, run `packwand refresh` first"
            );
        }
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
    if !all && selected.is_none() {
        return Err("provide a metadata name or use --all".into());
    }
    // Delegates to packwand-ops rather than repeating the resolve/apply loop:
    // that is where the bulk Modrinth lookup lives, so `--all` costs a handful
    // of requests instead of one per mod.
    Ok(packwand_ops::update_latest(root, selected, all, dry_run)?
        .into_iter()
        .map(UpdateRecord::from)
        .collect())
}

/// Whether an update record represents something that actually went wrong.
///
/// A pinned file and a file with no `[update]` block are both deliberately
/// manually managed — packwiz's way of saying "leave this alone" — so neither
/// is a failed check. Counting them as failures made every workspace report
/// look broken and turned `--check` into a permanent non-zero exit.
fn is_failure(record: &UpdateRecord) -> bool {
    match record.error.as_deref() {
        None | Some("pinned") => false,
        Some(message) => !message.contains("no supported update provider"),
    }
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
        let batch = root
            .clone()
            .try_get_matches_from(["packwand", "batch"])
            .unwrap();
        assert_eq!(
            requested_group_path(&root, &batch),
            Some(vec!["batch".into()])
        );

        let provider = root
            .clone()
            .try_get_matches_from(["packwand", "batch", "mr"])
            .unwrap();
        assert_eq!(
            requested_group_path(&root, &provider),
            Some(vec!["batch".into(), "mr".into()])
        );

        let leaf = root
            .clone()
            .try_get_matches_from(["packwand", "batch", "status"])
            .unwrap();
        assert_eq!(requested_group_path(&root, &leaf), None);
    }
}
