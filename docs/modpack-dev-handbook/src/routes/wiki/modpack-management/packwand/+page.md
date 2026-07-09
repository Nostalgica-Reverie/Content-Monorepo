<script>
  import { siteConfig } from '$lib/site';
</script>

# packwand

packwand is a Minecraft modpack toolchain that keeps the packwiz metadata format but adds repository-aware workflows, multi-pack workspace management, publishing, and diagnostics.

Instead of handling downloaded JARs directly, packwand keeps modpacks in TOML-backed metadata that is version-controlled, reviewable, and exportable.

## When to reach for packwand

Use packwand when you need one or more of these:

- A single source of truth in Git for mods, configs, scripts, and exports
- Multiple related packs in one repository with shared content or synchronized updates
- First-class publishing workflows for Modrinth, CurseForge, and internal targets
- Diagnostics such as diffing, validation, content linting, and test installs
- A local GUI or HTTP API on top of the manifest-driven workflow

If you only need the smaller original CLI for a single pack, [packwiz](/wiki/modpack-management/packwiz) is still a good fit.

## What packwand adds on top of packwiz

- Workspace operations across many packs in the same repository
- Publishing commands that plan, build, upload, and verify release artifacts
- Repository-aware commands such as `diff`, `pages`, `workspace status`, and `workspace sync`
- Extra automation surfaces: HTTP API, local GUI, automation plans, and richer diagnostics
- A broader installer/export/testing story for teams maintaining long-lived packs

## Typical repository flow

1. Create or enter a pack repository.
2. Run `packwand init` for a single pack or `packwand new` when you want packwand's scaffolding.
3. Add mods from Modrinth, CurseForge, or forge-hosted releases with metadata commands instead of dropping JARs into `mods/`.
4. Commit the resulting manifest changes to Git.
5. Use `packwand refresh`, `validate`, `content-lint`, and `test` as quality gates.
6. Build and publish from the same metadata when the pack is ready.

## Single-pack vs multi-pack use

### Single pack

packwand still works well for one pack when you want publishing, validation, a local GUI, or a more opinionated CLI than packwiz offers.

### Multi-pack repository

packwand becomes more compelling when you maintain variants such as:

- client/server splits
- loader ports
- long-term support branches
- regional or platform-specific releases
- "base pack" content reused by consumer packs

In these cases, `workspace`, `packs`, `diff`, and `publish` remove a lot of manual repository work.

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
- `--config` Select the packwand config file
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
- <a href={siteConfig.packwand.repoUrl}>Repository</a>
- <a href={siteConfig.packwand.releasesUrl}>Releases</a>