# Lasting Legacy/LCE-Monorepo
This is the repository hosting all of the different Lasting Legacy modpacks, resource packs, and datapacks.

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
- JSON Linter
- TOML Linter
- PNG Compressor
*for modpacks only
**on publish and build only

### Using Auto Publish
Every project in the repo must have a manifest.json. This manifest.json specifies stuff that our publish.yml then uses to auto publish. Once it is set up, you may simply bump version in the manifest.json and it will update across platforms.

### Using Canary Channels
Our Auto Publish action comes with an additional thing, a Canary channel for projects. To properly utilize this, add in a manifest-experimental.json, and properly configure it according to the schema, and every commit on the pack, it will automatically publish to a dedicated canary channel.

### Using Auto Update & Auto Refresh
Auto Update and Auto Refresh can be very powerful things! It allows you to automatically update packs. Since May 23rd, 2026, the action has now been made opt-out as well.

Auto Update and Auto Refresh will automatically update and validate all mods in every pack that is not opted-out of the feature. To opt out, please add a auto-update-ignore.json, with any reason you'd like. This is purely cosmetic and does not impact anything.

### Using Builds
All builds in the repo occur when a commit happens to their specific subdirectory. These builds are the same as what Auto Publish uses.

Only the pack modified within a commit will be built. So if you modified something in, lets say Simply Legacy, your commit would only build Simply Legacy, and not Re-Console+ or 2000's Edition.

This means builds can be very fast, sometimes taking only 30 seconds.

### Using Linters (currently unavailable)
All linters automatically run on commit, and will fail if the modified JSON/TOML is broken. This is helpful in the case of making a minor mistake in syntax

## Credits
justfile forked from skywardmc, stale.yml forked from JEI. Both licensed under MIT.

# License
As all of these projects are different, the license may vary. Most packs are under GPL-3.0, or MIT. Please check the pack folder or the pages on official sites (Modrinth, CurseForge) for the license.

All* actions are licensed under AGPL-3.0 and written in Rust/Typescript.
