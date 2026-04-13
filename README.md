# Lasting Legacy/LCE-Monorepo
This is the repository hosting all of the different Lasting Legacy modpacks, resource packs, and datapacks.

## Notice
Development is held on [git.nostalgica.net](https://git.nostalgica.net/Lasting-Legacy/Content-Monorepo), GitHub and Codeberg are mirrors. Please go to our [GitHub Issues](https://github.com/Nostalgica-Reverie/Content-Monorepo/issues) page to report any issues.

# General
This repository hosts all the source and files for all of our resource packs, data packs, modpacks and more. This readme is primarily intended for internal developer usage.

## Contributing
First, please refer to the CONTRIBUTING.md file in the repository. This will tell you some basics

## Actions
The repository makes usage of Forgejo actions, for CI/CD and general QoL improvements to our dev process.

## Current Functions
- Auto Publish
- Auto Update and Auto Refresh*
- Auto Build
- Datapack and Resourcepack validator
- JSON Linter
- Resource Pack Compressor**
*for modpacks only
**on publish and build only

### Using Auto Publish (for devs)
Every project in the repo must have a manifest.json. This manifest.json specifies stuff that our publish.yml then uses to auto publish. Once it is set up, you may simply bump version in the manifest.json and it will update across platforms.

### Incremental Builds
Only the pack modified within a commit will be built. So if you modified something in, lets say Simply Legacy, your commit would only build Simply Legacy, and not Re-Console or 2000's Edition.
License

### Credits
justfile from skywardmc, stale.yml from JEI. Licensed under MIT

# License
As all of these projects are different, the license may vary. Most packs are under GPL-3.0, or MIT. Please check the pack folder or the pages on official sites (Modrinth, CurseForge) for the license.

All* actions are licensed under AGPL-3.0 and written in Rust.
