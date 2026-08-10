# packwand

packwand is a desktop application and command line tool for creating and launching Minecraft modpacks. Instead of committing downloaded JAR files, packwand creates TOML metadata that can be version-controlled and shared with Git. Its built-in launcher installs that content with the native Packwand installer, or you can [export it to a CurseForge or Modrinth modpack](/tutorials/hosting/curseforge).

packwand is a fork of [packwiz](https://github.com/packwiz/packwiz) extended with multi-pack workspace management for the Lasting Legacy monorepo. It remains compatible with packwiz packs and the packwiz-installer ecosystem.

packwand is great for...

- Distributing private modpacks for servers
- Creating modpacks for CurseForge and Modrinth
- Managing many related packs at once (workspaces, base/consumer pack sync, bulk updates)
- Installing and launching development instances without a separate launcher

## Features

- Git-friendly TOML-based metadata format (`packwand:26` pack format; accepts legacy `packwiz:1.x` packs)
- SHA-512 hashing by default for new files and index entries
- Built-in Minecraft instance launcher backed by the native, hash-verifying Packwand installer
- Legacy Java installer compatibility for existing MultiMC/Prism, ATLauncher, and server workflows
- Pack distribution with HTTP servers, with a built-in local server for testing (`packwand serve`)
- Easy installation and updating of multiple mods at once from CurseForge and Modrinth
- Mods from GitHub releases, GitLab, and Forgejo/Gitea/Codeberg instances
- Exporting to CurseForge and Modrinth packs
- Importing from CurseForge packs
- Server-only and Client-only mod handling
- Multi-pack workspaces: `packwand workspace status/sync/refresh/update` across every pack in a repository
- A native desktop GUI (`packwand gui`)
- User-defined scripts in `pack.toml` run with `packwand run <name>`

## Getting started

- [Install packwand](/installation)
- [Create your first modpack](/tutorials/creating/getting-started)
- [Command reference](/reference/commands/packwand)
- [Pack format reference](/reference/pack-format/pack-toml)
