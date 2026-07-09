# packwand

Minecraft modpack toolchain - packwiz core with multi-pack workspace management.

Instead of handling downloaded JARs directly, packwand keeps modpacks in TOML-backed metadata that is version-controlled and exportable.

## Usage

```text
packwand [flags]
packwand [command]
```

## Pack Management

- `add` Add a mod to all or a specific pack's Modrinth and CurseForge subdirs
- `curseforge` Manage CurseForge-based mods
- `forgejo` Manage projects released on Forgejo, Gitea, or Codeberg
- `freeze` Pin mods so updates skip them
- `github` Manage projects released on GitHub
- `gitlab` Manage projects released on GitLab or self-hosted GitLab instances
- `import` Import an `.mrpack` or CurseForge zip as a new modpack
- `init` Initialise a packwiz modpack
- `modrinth` Manage Modrinth-based mods
- `new` Scaffold a new pack
- `pin` Pin a file so it does not get updated automatically
- `port` Compare Modrinth and CurseForge subdirs and port missing mods
- `rehash` Migrate all hashes to a specific format
- `remove` Remove an external file from the modpack
- `side` Check or fix a mod's side across all subdirs in a pack
- `unfreeze` Unpin mods so updates can apply to them again
- `unpin` Unpin a file so it receives updates
- `url` Add external files from a direct download link

## Updates & Refresh

- `migrate` Migrate Minecraft, loader, or pack-format generations
- `refresh` Refresh the index file
- `update` Update an external file or all external files in the modpack

## Build & Export

- `build` Build modpack exports and zip packs from git-changed targets
- `bump` Bump the manifest version
- `export` Export packs locally
- `publish` Build, upload, verify, or list publish targets for a pack

## Workspace

- `packs` Look up or edit any pack's manifest fields by id
- `workspace` Multi-pack workspace operations across all packs

## Diagnostics

- `content-lint` Lint pack content
- `doctor` Check that tools, repo root, and manifests are healthy
- `lint` Check JSON and `.pw.toml` files for syntax errors
- `list` List all the mods in the modpack
- `test` Spin up packwand serve and validate a pack with packwiz-installer
- `validate` Validate pack manifests
- `version` Print the packwand version

## Other

- `api` Run and inspect the Packwand HTTP API
- `automation` Query effective automation settings for a pack
- `cache` Inspect and maintain the shared download cache
- `diff` Show mod additions, removals, and updates between two git refs
- `gui` Run the local Packwand web GUI
- `modlist` Write a crash-assistant `modlist.json` from a pack's `mods/` directory
- `nix` Nix integration
- `pages` Regenerate `modlist.md` files and the projects index
- `run` Execute a user-defined script from `pack.toml`
- `serve` Run a local development server
- `settings` Manage pack settings
- `utils` Utilities for managing packwiz itself

## Flags

- `--cache` Override the shared download cache directory
- `--config` Select the Packwand config file
- `--meta-folder` Change where new metadata files are written
- `--meta-folder-base` Resolve `--meta-folder` relative to another base directory
- `--no-refresh` Skip index and `pack.toml` refresh after modifications
- `--pack-file` Select the pack metadata file
- `-y`, `--yes` Accept default prompts in non-interactive mode

## Getting started

- [Install packwand](/wiki/modpack-management/packwand/installation)
- [Create your first modpack](/wiki/modpack-management/packwand/tutorials/creating/getting-started)
- [Command reference](/wiki/modpack-management/packwand/reference/commands/packwand)
- [Pack format reference](/wiki/modpack-management/packwand/reference/pack-format/pack-toml)
