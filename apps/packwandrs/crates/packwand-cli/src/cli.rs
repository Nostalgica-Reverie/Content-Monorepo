use clap::{Arg, ArgAction, Command, ValueHint};

fn command(name: &'static str, about: &'static str) -> Command {
    Command::new(name).about(about)
}

fn flag(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .long(name)
        .help(help)
        .action(ArgAction::SetTrue)
}

fn option(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).long(name).help(help).value_name("VALUE")
}

fn positional(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).help(help).value_hint(ValueHint::AnyPath)
}

fn many(name: &'static str, help: &'static str) -> Arg {
    positional(name, help)
        .num_args(1..)
        .action(ArgAction::Append)
}

pub fn build() -> Command {
    command(
        "packwand",
        "Minecraft modpack toolchain — packwiz core with multi-pack workspace management",
    )
    .version("26.2.0")
    .disable_version_flag(true)
    .arg(
        option(
            "cache",
            "The directory where Packwand caches downloaded mods",
        )
        .global(true),
    )
    .arg(option("config", "The configuration file to use").global(true))
    .arg(
        option(
            "jobs",
            "Worker count for parallel operations; 0 uses defaults",
        )
        .short('j')
        .value_parser(clap::value_parser!(usize))
        .default_value("0")
        .global(true),
    )
    .arg(option("meta-folder", "Folder where new metadata files are added").global(true))
    .arg(
        option(
            "meta-folder-base",
            "Base directory for resolving the metadata folder",
        )
        .default_value(".")
        .global(true),
    )
    .arg(
        flag(
            "no-refresh",
            "Skip index and pack.toml refresh after modifications",
        )
        .global(true),
    )
    .arg(
        option("pack-file", "Modpack metadata file")
            .default_value("pack.toml")
            .global(true),
    )
    .arg(
        flag("yes", "Accept prompts non-interactively")
            .short('y')
            .global(true),
    )
    .subcommands(top_level_alphabetical())
}

/// Every top-level command, sorted a-z for `--help`. The individual group
/// functions below still exist to keep related commands (and the crate that
/// owns their logic) grouped in the source — see the `new-packwand-cmd`
/// skill — but the tree clap actually builds is a single flattened,
/// alphabetized list, not the source's thematic grouping.
fn top_level_alphabetical() -> Vec<Command> {
    let mut commands = Vec::new();
    commands.extend(pack_management());
    commands.extend(updates());
    commands.extend(build_export());
    commands.extend(batch_commands());
    commands.extend(diagnostics());
    commands.extend(other());
    commands.push(
        command("completion", "Generate shell completion scripts")
            .arg(positional("shell", "bash, elvish, fish, powershell, or zsh").required(true)),
    );
    commands.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    commands
}

fn pack_management() -> Vec<Command> {
    vec![
        command(
            "add",
            "Add a mod to all or a specific pack's platform subdirs",
        )
        .arg(positional("project", "Project slug or URL").required(true))
        .arg(positional("pack", "Pack directory or subdir")),
        curseforge(),
        repository_provider("forgejo", "Forgejo, Gitea, or Codeberg", true, true),
        command("freeze", "Pin mods so workspace updates skip them")
            .arg(positional("pack-subdir", "Pack subdir").required(true))
            .arg(many("mod-slugs", "Mod slugs").required(false))
            .arg(flag("json", "Output JSON")),
        repository_provider("github", "GitHub", false, true),
        repository_provider("gitlab", "GitLab", true, false),
        command(
            "import",
            "Import an .mrpack or CurseForge zip as a new modpack",
        )
        .arg(positional("archive", "Archive path or URL").required(true))
        .arg(option("id", "Override the derived pack ID")),
        init(),
        modrinth(),
        new_project(),
        command("pin", "Pin external metadata files")
            .arg(many("names", "Metadata names").required(false)),
        command("port", "Compare MR and CF subdirs and port missing mods")
            .arg(positional("mr-subdir", "Modrinth subdir").required(true))
            .arg(positional("cf-subdir", "CurseForge subdir").required(true))
            .arg(flag("add", "Add missing CurseForge entries"))
            .arg(flag("no-refresh", "Batch refresh"))
            .arg(flag("json", "Output JSON")),
        command("rehash", "Migrate all hashes to a specific format")
            .arg(positional("format", "sha1, sha256, or sha512")),
        command("remove", "Remove external metadata files")
            .alias("rm")
            .arg(many("names", "Metadata names").required(false)),
        command("side", "Check or fix a mod's side across pack subdirs")
            .arg(positional("pack-dir", "Pack directory").required(true))
            .arg(positional("mod-slug", "Mod slug").required(true))
            .arg(positional("side", "client, server, both, or either")),
        command("unfreeze", "Unpin workspace-frozen mods")
            .arg(positional("pack-subdir", "Pack subdir").required(true))
            .arg(many("mod-slugs", "Mod slugs")),
        command("unpin", "Unpin external metadata files")
            .arg(many("names", "Metadata names").required(false)),
        command("url", "Manage direct-download external files").subcommand(
            command("add", "Add a direct-download external file")
                .arg(positional("name", "Display name").required(true))
                .arg(positional("url", "Download URL").required(true))
                .arg(flag("force", "Allow URLs supported by another provider"))
                .arg(option("meta-name", "Metadata filename")),
        ),
    ]
}

fn curseforge() -> Command {
    command("curseforge", "Manage CurseForge-based mods")
        .alias("cf")
        .subcommand(
            command("add", "Add a CurseForge project")
                .arg(positional("project", "URL, slug, or search"))
                .arg(option("addon-id", "Project ID"))
                .arg(option("file-id", "File ID"))
                .arg(option("game", "Game slug").default_value("minecraft"))
                .arg(option("category", "Category slug"))
                .arg(option("release-channel", "release, beta, or alpha")),
        )
        .subcommand(
            command("update", "Replace a mod with a specific CurseForge file")
                .arg(positional("name", "Existing metadata name").required(true))
                .arg(positional("file", "CurseForge file URL").required(true)),
        )
        .subcommand(command(
            "detect",
            "Detect CurseForge metadata from indexed files",
        ))
        .subcommand(
            command("export", "Export a CurseForge modpack zip")
                .arg(
                    option("side", "Export side")
                        .short('s')
                        .default_value("client"),
                )
                .arg(option("output", "Output archive").short('o')),
        )
        .subcommand(
            command("import", "Import a CurseForge modpack zip")
                .arg(positional("path", "Archive path").required(true)),
        )
        .subcommand(
            command("open", "Open a CurseForge project page")
                .alias("doc")
                .arg(positional("name", "Metadata name").required(true)),
        )
}

fn modrinth() -> Command {
    command("modrinth", "Manage Modrinth-based mods")
        .alias("mr")
        .subcommand(
            command("add", "Add a Modrinth project")
                .arg(positional("project", "URL, slug, or search"))
                .arg(option("project-id", "Project ID"))
                .arg(option("version-id", "Version ID"))
                .arg(option("version-filename", "Version filename")),
        )
        .subcommand(
            command("export", "Export a Modrinth .mrpack")
                .arg(option("output", "Output archive").short('o'))
                .arg(flag(
                    "restrictDomains",
                    "Restrict downloads to Modrinth-approved domains",
                ))
                .arg(flag("verify", "Re-download and verify persisted hashes")),
        )
}

fn repository_provider(
    name: &'static str,
    label: &'static str,
    instance: bool,
    branch: bool,
) -> Command {
    let mut add = command("add", "Add a repository release")
        .arg(positional("project", "Repository URL or owner/name"))
        .arg(option("regex", "Release asset regular expression"));
    if instance {
        add = add.arg(option("instance", "Self-hosted instance hostname"));
    }
    if branch {
        add = add.arg(option("branch", "Release branch"));
    }
    command(name, label).subcommand(add)
}

fn init() -> Command {
    command("init", "Initialise a packwiz modpack")
        .arg(option("name", "Pack name"))
        .arg(option("author", "Pack author"))
        .arg(option("version", "Pack version"))
        .arg(option("index-file", "Index filename").default_value("index.toml"))
        .arg(option("mc-version", "Minecraft version"))
        .arg(flag("latest", "Use latest Minecraft version").short('l'))
        .arg(flag("snapshot", "Allow latest snapshot").short('s'))
        .arg(flag("reinit", "Replace an existing pack file").short('r'))
        .arg(option("modloader", "fabric, forge, neoforge, or quilt"))
        .arg(option("fabric-version", "Fabric loader version"))
        .arg(flag("fabric-latest", "Use latest Fabric loader"))
        .arg(option("forge-version", "Forge loader version"))
        .arg(flag("forge-latest", "Use latest Forge loader"))
        .arg(option("neoforge-version", "NeoForge loader version"))
        .arg(flag("neoforge-latest", "Use latest NeoForge loader"))
        .arg(option("quilt-version", "Quilt loader version"))
        .arg(flag("quilt-latest", "Use latest Quilt loader"))
}

fn new_project() -> Command {
    command("new", "Scaffold a new pack")
        .arg(positional("category", "mods, modpacks, datapacks, or resourcepacks").required(true))
        .arg(positional("name", "Project ID/name").required(true))
        .arg(option("mc", "Minecraft version"))
        .arg(option("loader", "Mod loader").default_value("fabric"))
        .arg(flag("base", "Scaffold as a performance base"))
        .arg(option("consumes", "Performance-base project ID"))
        .arg(option("variants", "Comma-separated variant IDs"))
}

fn updates() -> Vec<Command> {
    vec![
        command(
            "migrate",
            "Migrate Minecraft, loader versions, or pack format",
        )
        .subcommand(command("format", "Migrate to the current pack format"))
        .subcommand(
            command("minecraft", "Migrate Minecraft version")
                .arg(positional("version", "Target version")),
        )
        .subcommand(
            command("loader", "Migrate loader version")
                .arg(positional("version", "Version, latest, or recommended")),
        ),
        command("refresh", "Refresh the index file")
            .arg(flag("build", "Generate the distribution pack hash")),
        command("update", "Update external files")
            .arg(positional("name", "Metadata name"))
            .arg(flag("all", "Update every external file").short('a'))
            .arg(flag("dry-run", "Report updates without writing"))
            .arg(option("report", "Write JSON report (requires --all)"))
            .arg(flag("json", "Output JSON report")),
    ]
}

fn build_export() -> Vec<Command> {
    vec![
        command(
            "build",
            "Build changed mod jars, modpack exports, and zip packs",
        )
        .arg(positional("sha", "SHA suffix"))
        .arg(option("pack", "Build a specific project").short('p')),
        command("bump", "Bump a manifest version")
            .arg(positional("pack-dir", "Project directory").required(true))
            .arg(positional("new-version", "New version").required(true))
            .arg(flag(
                "configs",
                "Update in-pack version configs and refresh",
            )),
        command("export", "Export packs locally")
            .arg(positional("pack-name", "Optional project name")),
        command("json", "JSON utilities for pack files").subcommand(
            command("minify", "Minify JSON and .mcmeta files")
                .arg(many("paths", "Files or directories"))
                .arg(flag("check", "Report without rewriting"))
                .arg(flag("strict", "Fail on invalid JSON")),
        ),
        publish(),
    ]
}

fn publish() -> Command {
    command("publish", "Build, upload, verify, or list publish targets")
        .subcommand(
            command("plan", "Plan publish targets from git changes")
                .arg(option("from", "Base git ref"))
                .arg(option("to", "Target git ref").default_value("HEAD"))
                .arg(flag("no-validate", "Skip manifest validation"))
                .arg(option("pack", "Limit to one project directory")),
        )
        .subcommand(
            command("list", "List publish targets").arg(many("manifests", "Manifest files")),
        )
        .subcommand(
            command("build", "Build a publish target")
                .arg(positional("manifest", "Manifest path").required(true))
                .arg(positional("variant", "Variant ID")),
        )
        .subcommand(
            command("upload", "Upload a publish target")
                .arg(positional("manifest", "Manifest path").required(true))
                .arg(positional("variant", "Variant ID"))
                .arg(flag("live", "Actually upload; default is dry-run"))
                .arg(option("changelog-file", "Release notes file")),
        )
        .subcommand(
            command("verify", "Verify a publish target")
                .arg(positional("manifest", "Manifest path").required(true))
                .arg(positional("variant", "Variant ID")),
        )
}

fn batch_commands() -> Vec<Command> {
    vec![
        command("packs", "Look up or edit manifest fields by ID")
            .arg(flag("json", "Output JSON").global(true))
            .subcommand(command("list", "List manifest projects"))
            .subcommand(
                command("get", "Get a manifest or field")
                    .arg(positional("id", "Project ID").required(true))
                    .arg(positional("field", "Field name")),
            )
            .subcommand(
                command("set", "Set a manifest field")
                    .arg(positional("id", "Project ID").required(true))
                    .arg(positional("field", "Field name").required(true))
                    .arg(positional("value", "New value").required(true)),
            )
            .subcommand(command("index", "Generate the projects index")),
        batch(),
    ]
}

fn batch() -> Command {
    command("batch", "Multi-pack batch operations")
        .subcommand(command("status", "Report workspace status").arg(flag("json", "Output JSON")))
        .subcommand(
            command("export", "Export pack targets")
                .arg(positional("pack-dir", "Project directory"))
                .arg(flag("all", "Run across all packs")),
        )
        .subcommand(
            command("mr", "Modrinth workspace operations").subcommand(
                command("add", "Add projects across Modrinth subdirs")
                    .arg(many("projects", "Slugs or URLs"))
                    .arg(flag("all", "Run across all packs")),
            ),
        )
        .subcommand(
            command("cf", "CurseForge workspace operations").subcommand(
                command("add", "Add projects across CurseForge subdirs")
                    .arg(many("projects", "Slugs or URLs"))
                    .arg(flag("all", "Run across all packs")),
            ),
        )
        .subcommand(
            command("update", "Update workspace packs")
                .arg(positional("pack-dir", "Project directory"))
                .arg(flag("all", "Run across all packs"))
                .arg(flag("check", "Dry-run update check"))
                .arg(flag("json", "Output JSON with --check"))
                .arg(flag("ignored-only", "Check opted-out packs"))
                .arg(option("report", "Write aggregated JSON report")),
        )
        .subcommand(
            command("refresh", "Refresh workspace packs")
                .arg(positional("pack-dir", "Project directory"))
                .arg(flag("all", "Run across all packs"))
                .arg(flag("dry-run", "List targets without refreshing")),
        )
        .subcommand(
            command("loader-update", "Update loaders across the workspace")
                .arg(positional("version", "latest or recommended"))
                .arg(positional("pack-dir", "Project directory")),
        )
        .subcommand(command("migrate", "Migrate workspace packs").arg(many(
            "migration",
            "format, loader [version], or minecraft [version]",
        )))
        .subcommand(
            command("sync", "Synchronize performance-base consumers")
                .arg(flag("dry-run", "Show changes without writing")),
        )
}

fn diagnostics() -> Vec<Command> {
    vec![
        command(
            "ci-local",
            "Run CI-equivalent validation stages for a subdir",
        )
        .arg(positional("dir", "Pack subdir"))
        .arg(flag("json", "Output JSON")),
        command(
            "content-lint",
            "Lint pack namespaces, references, metadata, and collisions",
        )
        .arg(many("pack-dirs", "Pack directories").required(false))
        .arg(flag("all", "Lint all content projects"))
        .arg(flag("json", "Output JSON")),
        command(
            "deps",
            "Report Modrinth required-dependency coverage for a pack subdir",
        )
        .alias("graph")
        .arg(positional(
            "pack-dir",
            "Pack subdir (defaults to the current directory)",
        ))
        .arg(flag("json", "Output JSON")),
        command("doctor", "Check tools, repository root, and manifests")
            .arg(flag("json", "Output JSON")),
        command(
            "explain",
            "Show everything Packwand knows about one installed mod",
        )
        .arg(positional("mod-slug", "Metadata name or slug").required(true))
        .arg(positional(
            "pack-dir",
            "Pack subdir (defaults to the current directory)",
        ))
        .arg(flag("json", "Output JSON")),
        command("lint", "Check JSON and .pw.toml files for syntax errors")
            .arg(many("files", "Files to lint").required(false)),
        command("list", "List mods in the current pack")
            .arg(flag("version", "Print name and version").short('v'))
            .arg(option("side", "Filter by side").short('s'))
            .arg(flag("json", "Output JSON")),
        command("parity", "Report MR/CF variant drift")
            .arg(many("pack-dirs", "Project directories").required(false))
            .arg(flag("json", "Output JSON"))
            .arg(flag("strict", "Fail when variants drift")),
        command("preflight", "Run the pre-launch validation gate")
            .arg(positional("dir", "Pack directory"))
            .arg(flag("json", "Output JSON")),
        command("registry", "Build content registries")
            .arg(
                positional("kind", "datapack, config, resourcepack, kubejs, or all").required(true),
            )
            .arg(positional("dir", "Content directory"))
            .arg(flag("json", "Output JSON")),
        command("test", "Serve a pack and run packwiz-installer validation")
            .arg(positional("pack-subdir", "Pack subdir").required(true)),
        command("validate", "Validate pack manifests")
            .alias("check-manifest")
            .arg(many("manifests", "Manifest paths").required(false))
            .arg(flag("all", "Validate all manifests")),
        command("version", "Print the Packwand version"),
    ]
}

fn other() -> Vec<Command> {
    vec![
        command("api", "Run and inspect the Packwand HTTP API").subcommand(
            command("serve", "Run the headless Packwand HTTP API")
                .arg(option("bind", "Bind address").default_value("127.0.0.1:0"))
                .arg(option("token-file", "Bearer token file"))
                .arg(flag("generate-token", "Generate a missing token file"))
                .arg(option("print-port-file", "Write selected server URL")),
        ),
        automation(),
        command("cache", "Inspect and maintain the shared download cache").subcommand(
            command("prune", "Remove unreferenced cache entries")
                .arg(flag("dry-run", "Only list removals"))
                .arg(flag("json", "Output JSON")),
        ),
        command("diff", "Show mod changes between two git refs")
            .arg(positional("old-ref", "Old git ref").required(true))
            .arg(positional("new-ref", "New git ref").required(true))
            .arg(positional("path-prefix", "Optional path prefix"))
            .arg(flag("json", "Output JSON")),
        command("gui", "Run the native Packwand desktop app"),
        command("modlist", "Write crash-assistant modlist.json")
            .arg(positional("subdir", "Pack subdir").required(true))
            .arg(option("subdir-option", "Pack subdir override").short('s'))
            .arg(flag("json", "Output JSON")),
        command("nix", "Nix integration").subcommand(
            command("gen", "Generate packwiz2nix checksums")
                .arg(option("output", "Output path").default_value("checksums.json"))
                .arg(flag("all", "Generate for every pack subdir")),
        ),
        command("pages", "Regenerate modlist pages and projects index")
            .alias("docs")
            .arg(positional("pack-dir", "Project directory"))
            .arg(option("pack", "Project directory").short('p'))
            .arg(flag("json", "Output JSON")),
        command("run", "Execute a user-defined pack script")
            .arg(positional("script", "Script name").required(true)),
        command(
            "script",
            "Generate a context-aware .pw4 script under the repository pw4 folder",
        )
        .arg(option("name", "Output filename, with or without .pw4").default_value("workspace"))
        .arg(
            option("preset", "build, ci, or project")
                .default_value("build")
                .value_parser(["build", "ci", "project"]),
        )
        .arg(option(
            "project",
            "Project ID; inferred from the current directory when possible",
        ))
        .arg(
            option("kind", "modpack, datapack, resourcepack, or mod")
                .default_value("modpack")
                .value_parser(["modpack", "datapack", "resourcepack", "mod"]),
        )
        .arg(
            option("loader", "fabric, forge, neoforge, or quilt")
                .value_parser(["fabric", "forge", "neoforge", "quilt"]),
        )
        .arg(flag("force", "Replace an existing generated script"))
        .arg(flag("json", "Output the generated script report as JSON")),
        command("serve", "Run a local development server")
            .alias("server")
            .arg(option("port", "Port").short('p').default_value("8080"))
            .arg(
                Arg::new("refresh")
                    .long("refresh")
                    .short('r')
                    .action(ArgAction::Set)
                    .default_value("true"),
            )
            .arg(flag("basic", "Serve all files without refreshing")),
        settings(),
        command("utils", "Utilities for managing Packwiz")
            .subcommand(
                command("commands", "Print the command catalog").arg(flag("json", "Output JSON")),
            )
            .subcommand(
                command("markdown", "Generate Markdown command docs")
                    .arg(option("dir", "Destination directory").default_value(".")),
            ),
    ]
}

fn automation() -> Command {
    command("automation", "Query and run effective automation settings")
        .subcommand(
            command("get", "Get effective automation for a project")
                .arg(positional("pack-dir", "Project directory").required(true)),
        )
        .subcommand(
            command("run", "Run the unattended release pipeline")
                .arg(positional("pack-dir", "Project directory").required(true))
                .arg(flag("dry-run", "Skip version bump"))
                .arg(option("report", "Write JSON run report"))
                .arg(flag("json", "Output JSON")),
        )
        .subcommand(command("list-full-auto", "List full-auto-enabled projects"))
}

fn settings() -> Command {
    command("settings", "Manage pack settings")
        .subcommand(
            command("acceptable-loaders", "Manage acceptable loader names")
                .arg(positional("loader", "Loader name"))
                .arg(flag("add", "Add loader").short('a'))
                .arg(flag("remove", "Remove loader").short('r')),
        )
        .subcommand(
            command(
                "acceptable-versions",
                "Manage acceptable Minecraft versions",
            )
            .arg(positional("version", "Minecraft version"))
            .arg(flag("add", "Add version").short('a'))
            .arg(flag("remove", "Remove version").short('r')),
        )
}

#[cfg(test)]
mod tests {
    use super::build;

    const TOP_LEVEL: &[&str] = &[
        "add",
        "api",
        "automation",
        "batch",
        "build",
        "bump",
        "cache",
        "ci-local",
        "completion",
        "content-lint",
        "curseforge",
        "deps",
        "diff",
        "doctor",
        "explain",
        "export",
        "forgejo",
        "freeze",
        "github",
        "gitlab",
        "gui",
        "import",
        "init",
        "json",
        "lint",
        "list",
        "migrate",
        "modlist",
        "modrinth",
        "new",
        "nix",
        "packs",
        "pages",
        "parity",
        "pin",
        "port",
        "preflight",
        "publish",
        "refresh",
        "registry",
        "rehash",
        "remove",
        "run",
        "script",
        "serve",
        "settings",
        "side",
        "test",
        "unfreeze",
        "unpin",
        "update",
        "url",
        "utils",
        "validate",
        "version",
    ];

    #[test]
    fn command_tree_is_internally_valid_and_complete() {
        let command = build();
        command.clone().debug_assert();
        let names = command
            .get_subcommands()
            .map(clap::Command::get_name)
            .collect::<Vec<_>>();
        // TOP_LEVEL is itself kept a-z, so this also pins `--help`'s display
        // order: exact equality catches both a missing/extra command and a
        // command landing out of alphabetical order.
        assert_eq!(names, TOP_LEVEL, "top-level commands must list a-z");
    }

    #[test]
    fn required_nested_commands_are_present() {
        let command = build();
        for path in [
            "curseforge add",
            "curseforge detect",
            "curseforge export",
            "curseforge import",
            "modrinth add",
            "modrinth export",
            "migrate format",
            "migrate minecraft",
            "publish plan",
            "publish list",
            "publish build",
            "publish upload",
            "publish verify",
            "packs list",
            "packs get",
            "packs set",
            "packs index",
            "batch status",
            "batch mr add",
            "batch cf add",
            "automation get",
            "automation run",
            "cache prune",
            "json minify",
            "nix gen",
            "settings acceptable-loaders",
            "settings acceptable-versions",
            "utils commands",
            "utils markdown",
            "api serve",
        ] {
            assert!(find(&command, path), "missing nested command {path}");
        }
    }

    fn find(root: &clap::Command, path: &str) -> bool {
        let mut command = root;
        for part in path.split(' ') {
            let Some(next) = command
                .get_subcommands()
                .find(|child| child.get_name() == part)
            else {
                return false;
            };
            command = next;
        }
        true
    }
}
