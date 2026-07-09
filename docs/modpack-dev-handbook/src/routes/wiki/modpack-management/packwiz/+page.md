<script>
  import { siteConfig } from '$lib/site';
</script>

# packwiz

packwiz is a command line tool for authoring Minecraft modpacks as TOML metadata rather than as a checked-in folder of downloaded JARs.

It is the core manifest format that packwand builds on. If you want a smaller, established CLI for a single pack, packwiz is often the simplest place to start.

## Where packwiz fits best

packwiz is well suited to:

- Single-pack repositories
- Private packs for friends, servers, or internal testing
- Creator workflows where the manifest format matters more than repository automation
- Teams that want a stable, Git-friendly format without the packwand-specific surfaces

## Where packwiz is intentionally smaller

packwiz is not trying to be a repository orchestration tool. It is lighter on:

- multi-pack workspace management
- release planning and verification workflows
- repository diffing and diagnostics
- local GUI or API surfaces

If you want those higher-level workflows, move up to [packwand](/wiki/modpack-management/packwand).

## How packwiz relates to the rest of this section

- `packwiz` is the authoring CLI and metadata format.
- `packwiz-installer` is the runtime updater players and servers execute.
- The [bootstrap](/wiki/modpack-management/packwiz/components/bootstrap) is the tiny launcher-facing shim that updates and starts the installer.
- [packwand](/wiki/modpack-management/packwand) uses the same general pack format but adds more repository-aware tooling on top.

## Recommended author workflow

1. Create a clean repository for the pack.
2. Initialize the manifest with packwiz.
3. Add mods through metadata commands rather than copying JARs into source control.
4. Commit the resulting manifest and config changes to Git.
5. Test distribution through a local server, export, or installer flow before publishing.

## Features

- Git-friendly TOML-based metadata format
- Java-based pack installer/updater (works with MultiMC and ATLauncher), with support for optional mods and fast automatic updates
- Pack distribution with HTTP servers, with a built-in local server for testing
- Easy installation and updating of multiple mods at once from CurseForge and Modrinth
- Exporting to CurseForge and Modrinth packs
- Importing from CurseForge packs
- Server-only and client-only mod handling
- Creation of remote file metadata from JAR files for CurseForge mods

## Useful links

- <a href={siteConfig.packwiz.repoUrl}>packwiz repository</a>
- <a href={siteConfig.packwiz.examplePackUrl}>example pack</a>
- <a href={siteConfig.packwiz.guiUrl}>third-party GUI project</a>
- <a href={siteConfig.packwiz.discordUrl}>upstream Discord</a>