# packwand

packwand is a command line tool for creating Minecraft modpacks. Instead of managing JAR files directly, packwand creates TOML metadata files which can be easily version-controlled and shared with git. You can then [export it to a CurseForge or Modrinth modpack](/tutorials/hosting/curseforge), or [use packwiz-installer](/tutorials/installing/packwiz-installer) for an auto-updating instance.

packwand is a fork of [packwiz](https://github.com/packwiz/packwiz) extended with multi-pack workspace management for the Lasting Legacy monorepo. It remains compatible with packwiz packs and the packwiz-installer ecosystem.

packwand is great for...

- Distributing private modpacks for servers
- Creating modpacks for CurseForge and Modrinth
- Managing many related packs at once (workspaces, base/consumer pack sync, bulk updates)

packwand is not so great for...

- Managing downloaded mod files (use a launcher for that)

## Features

- Git-friendly TOML-based metadata format (`packwand:26` pack format; accepts legacy `packwiz:1.x` packs)
- SHA-512 hashing by default for new files and index entries
- Java-based pack installer/updater ([packwiz-installer](/tutorials/installing/packwiz-installer), works with MultiMC/Prism and ATLauncher), with support for optional mods and fast automatic updates - perfect for servers!
- Pack distribution with HTTP servers, with a built-in local server for testing (`packwand serve`)
- Easy installation and updating of multiple mods at once from CurseForge and Modrinth
- Mods from GitHub releases, GitLab, and Forgejo/Gitea/Codeberg instances
- Exporting to CurseForge and Modrinth packs
- Importing from CurseForge packs
- Server-only and Client-only mod handling
- Multi-pack workspaces: `packwand workspace status/sync/refresh/update` across every pack in a repository
- A local web GUI (`packwand gui`)
- User-defined scripts in `pack.toml` run with `packwand run <name>`

## Getting started

- [Install packwand](/installation)
- [Create your first modpack](/tutorials/creating/getting-started)
- [Command reference](/reference/commands/packwand)
- [Pack format reference](/reference/pack-format/pack-toml)
