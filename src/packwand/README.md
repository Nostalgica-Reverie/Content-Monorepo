# packwand
packwand is a go based command line tool for creating Minecraft modpacks. It is a hard fork of packwiz designed for mass concurrency, monorepo handling, batch commands and general quality of life on top of packwiz. It comes with publishing, auto updating, linting, modlist generation, better pack initialization, porting from modrinth to curseforge and vice versa, importing a modpack from zip/mrpack or a url, batch refresh, batch update, version bumping, etc etc.

packwand is great for...

- Creating modpacks for CurseForge and Modrinth
- Creating many modpacks with shared featuresets
- Publishing modpacks automatically
- Utilizing CI/CD for automation

packwand is not so great for...

- Managing downloaded mod files (use [Curse/GDLauncher or another CLI](https://gist.github.com/comp500/13ae6f058221196077fb19953ac608c7))

## Features
packwand comes with:
- Auto-publishing 
- Acceptable loaders being able to have multiple modloaders - very useful for modpacks using Sinytra Connector
- Arbitrary command support
- Batch updating
- Batch refreshing
- Batch exporting
- Better erroring
- Build tools
- ForgeJo/Codeberg/Gitea support (similar to packwiz gh add!)
- GUI
- Higher security via mandatory sha-512
- Inter-platform porting (from  modrinth to curseforge and vice versa)
- Linting of config files
- Modpack importing from .mrpack or .zip (curseforge format)
- Mod List generation
- Manifest based systems (which runs auto publish and other things)
- Opt-in automations like auto changelog generation and hard validation to ensure pack compliance
- Pack initialization
- Pack siding (Client, Server, Either/Or, or both)
- Pack syncing; a system where modpacks can become a set of shared utilities, where a performance pack you use can be automatically mirrored to any pack it needs
- Proper CI/CD support, where the CI is the tool
- --no-refresh flag implemented to save on speed on batch updates, at the cost of potential breakage

Plus everything in packwiz:
- Git-friendly TOML-based metadata format
- MultiMC pack installer/updater, with support for optional mods and fast automatic updates - perfect for servers!
- Pack distribution with HTTP servers, with a built in local server for testing
- Easy installation and updating of multiple mods at once from CurseForge and Modrinth
- Exporting to CurseForge and Modrinth packs
- Importing from CurseForge packs
- Server-only and Client-only mod handling
- Creation of remote file metadata from JAR files for CurseForge mods

## Installation
Prebuilt binaries are not currently available.

To install manually;

1. Install Go (1.24 or newer) from https://golang.org/dl/
2. run `cd src\packwand`
3. Run `go install`. This may take a while.

## Documentation
Packwand lacks any current documentation, please check back later.

## License
All packwand new features are under AGPL-3.0 or later. All original packwiz code remains under MIT license.
