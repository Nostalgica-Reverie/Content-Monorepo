# Reverie Projects/monorepo
This is the repository hosting all of the different Reverie Projects modpacks, resource packs, and datapacks.

## Notice
Development is (currently) held on [git.nostalgica.net](https://git.nostalgica.net/Lasting-Legacy/Content-Monorepo), Tangled, GitHub and Codeberg are mirrors. Please go to our [GitHub Issues](https://github.com/Nostalgica-Reverie/Content-Monorepo/issues) page to report any issues.

# General
This repository hosts all the source and files for all of our resource packs, data packs, modpacks and more. This readme is primarily intended for internal developer usage.

## Contributing
First, please refer to the CONTRIBUTING.md file in the repository. This will tell you some basics

# Actions
The repository makes usage of Forgejo actions, for CI/CD and general QoL improvements to our dev process.

## Current Functions
- Auto Publish
- Auto Update and Auto Refresh*
- Auto Build
- Bulk Refresh*
- Bulk Update*
- Bulk PNG Optimizer
- JSON Linter
- TOML Linter
- Modpack Sync*
*for modpacks only
**on publish and build only

### Using Auto Publish
Every project in the repo must have a manifest.json. This manifest.json specifies stuff that our publish.yml then uses to auto publish. Once it is set up, you may simply bump version in the manifest.json and it will update across platforms.

Whenever Auto Publish is ran, it will be ran through a Validator. The Validator will fail if something is improperly configured; whether that be the lack of a changelog.md, a malformed manifest.json, or other reasons. If a publish run fails, please look to your manifest and set-up to make sure you are properly set up.

### Using Sync
To address issues regarding our packs being intertwined in content and development, there is now a Sync system implemented. 

Sync will essentially make one pack act as a library for whatever pack needs it. A good example is Simply Optimized Forked; a handful of our modpacks utilize this modpack as its performance base, so that we do not have to reimplement the same optimizations over and over.

In manifest.json, a pack must declare whether it is a ```base```. If it is, then other packs can hook into it to be synced up automatically, with a structure similar to this:

```
"role": {
    "performance_base": {
      "pack": "lce-common",
      "mappings": [
        { "source": "26.1.2-mr", "target": "26.1.2-mr" }
      ]
    }
  }
} 
```

This means that this pack is directly synced with ```lce-common```, benefitting from all of its changes

### Using Canary Channels
Our Auto Publish action comes with an additional thing, a Canary channel for projects. To properly utilize this, add in a manifest-experimental.json, and properly configure it according to the schema, and every commit on the pack, it will automatically publish to a dedicated canary channel.

### Using Auto Update & Auto Refresh
Auto Update and Auto Refresh can be very powerful things! It allows you to automatically update packs. Since May 23rd, 2026, the action has now been made opt-out as well.

Auto Update and Auto Refresh will automatically update and validate all mods in every pack that is not opted-out of the feature. To opt out, please add a auto-update-ignore.json, with any reason you'd like. This is purely cosmetic and does not impact anything.

### Using Builds
All builds in the repo occur when a commit happens to their specific subdirectory. These builds are the same as what Auto Publish uses.

Only the pack modified within a commit will be built. So if you modified something in, lets say Simply Legacy, your commit would only build Simply Legacy, and not Re-Console+ or 2000's Edition.

This means builds can be very fast, sometimes taking only 30 seconds.

### Using Linters
All linters automatically run on commit, and will fail if the modified JSON/TOML is broken. This is helpful in the case of making a minor mistake in syntax

# Somnus
Somnus is a WIP CLI tool written in go that is built directly into the Monorepository. To utilize it, you must install it locally. It can be very helpful to modpack development!

# Installing Somnus
To install Somnus, you must install Somnus, Builder and Maintain. All 3 of these are written in Go, so you can simply navigate to their directories and run ```go install .```.

Afterward, you can run somnus in the CLI and see the current commands.

## Using Somnus
Somnus is primarily a tool to accelerate some hurdles in pack development, regarding exporting with packwiz, initiating new packs under our monorepo structure, and to also allow CI to be ran locally in a better way.

### Somnus Init
```somnus init``` will initiate a new modpack with a manifest.json and a changelog.md. You must do the subdirectory yourself, but there are plans to make it set up packwiz for you as well in the future.

### Somnus Bump
```somnus bump``` will bump manifest via CLI.

### Somnus Export
```somnus export``` will batch export every version of the pack in a .mrpack/.zip format in a non tracked folder for you to use.

### Somnus Sync
```somnus sync``` runs the sync command locally.

### Somnus Modlist
```somnus modlist``` will generate a Crash Assistant mod list derived from the packs ```index.toml```.

### Somnus Test
```somnus test``` will (attempt) to set up an untracked auto-updating MultiMC instance, currently for testing purposes.

## Credits
stale.yml forked from JEI. Both licensed under MIT.

# License
As all of these projects are different, the license may vary. Most packs are under GPL-3.0, or MIT. Please check the pack folder or the pages on official sites (Modrinth, CurseForge) for the license.

All* actions are licensed under AGPL-3.0 and written in Rust/Typescript/Go.
