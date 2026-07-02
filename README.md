# Reverie Projects/monorepo
This is the repository hosting all of the different Reverie Projects modpacks, resource packs, and datapacks.

## Notice
Development is (currently) held on [git.nostalgica.net](https://git.nostalgica.net/Lasting-Legacy/Content-Monorepo), Tangled, GitHub and Codeberg are mirrors. Please go to our [GitHub Issues](https://github.com/Nostalgica-Reverie/Content-Monorepo/issues) page to report any issues.

# General
This repository hosts all the source and files for all of our resource packs, data packs, modpacks and documentation. This readme is primarily intended for internal developer usage.

## Contributing
First, please refer to the CONTRIBUTING.md file in the repository. This will tell you some basics.

### How to work on the repo
1. Install Go (1.24 or newer) from https://golang.org/dl/
2. run `cd src\packwand`
3. run `go install`

Now you have our tooling set up.

# Documentation
- Install [Node.js](https://nodejs.org/en/download) version 18 or higher
- Install VitePress through NPM in a terminal
```
npm add -D vitepress@next
```
- Start the local development server
```
npm run docs:dev
```

Any changes made in the docs segment of the repo will now automatically update locally

# Modpacks
To work on modpacks, you must install Packwand, which the download was provided above.

# Actions
The repository makes usage of Forgejo actions, for CI/CD and general QoL improvements to our dev process.

## Current Functions
- Auto Publish
- Auto Update and Auto Refresh*
- Auto Build
- Bulk Refresh*
- Bulk Update*
- Bulk Loader Update*
- Bulk PNG Optimizer
- JSON Linter
- TOML Linter
- Modpack Sync*

*for modpacks only

### Using Auto Publish
Every project in the repo must have a manifest.json. This manifest.json specifies stuff that our publish.yml then uses to auto publish. Once it is set up, you may simply bump version in the manifest.json and it will update across platforms.

Please read tools/manifest/schema.json to understand the manifest.json.

Whenever Auto Publish is ran, it will be ran through a Validator. The Validator will fail if something is improperly configured; whether that be the lack of a changelog.md, a malformed manifest.json, or other reasons. If a publish run fails, please look to your manifest and set-up to make sure you are properly set up.

Auto Publish will also automatically list out every mod updated, added or removed in a modpack, and all commits to the pack. This allows for some time saving as you no longer now have to keep track of constant changes, and they are automatically added to the changelog

### Using Canary Channels
Our Auto Publish action comes with an additional thing, a Canary channel for projects. To properly utilize this, add in a manifest-experimental.json, and properly configure it according to the schema, and every commit on the pack, it will automatically publish to a dedicated canary channel.

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

This means that this pack is directly synced with ```lce-common```, benefitting from all of its changes automatically. It allows for easier development as the packs relying on it are essentially patches on top of the base.

### Using Auto Update & Auto Refresh
Auto Update and Auto Refresh can be very powerful things! It allows you to automatically update packs. Since May 23rd, 2026, the action has now been made opt-out as well.

Auto Update and Auto Refresh will automatically update and validate all mods in every pack that is not opted-out of the feature. To opt out, please add a auto-update-ignore.json, with any reason you'd like. This is purely cosmetic and does not impact anything.

### Using Builds
All builds in the repo occur when a commit happens to their specific subdirectory. These builds are the same as what Auto Publish uses.

Only the pack modified within a commit will be built. So if you modified something in, lets say Simply Legacy, your commit would only build Simply Legacy, and not Re-Console+ or 2000's Edition.

This means builds can be very fast, sometimes taking only 30 seconds.

### Using Linters
All linters automatically run on commit, and will fail if the modified JSON/TOML is broken. This is helpful in the case of making a minor mistake in syntax

### Using Bulk Actions
Bulk Actions allow you to a lot of one thing, in a single button push via our ForgeJo. This allows for bulk PNG compression, a bulk refresh (which will fix any broken modpacks), and a bulk update (does the same as auto update and auto refresh!)

# Packwand
To address issues found in packwiz, we forked it and merged it with our existing tooling. For more info, please read the readme in src/packwand.

## Credits
stale.yml forked from JEI, licensed under MIT. Packwand and Packwand Documentation forked from Packwiz, licensed under MIT. Packwand Spec forked from Packwiz Spec, licensed under CC0-1.0. All credits to their original authors

# License
As all of these projects are different, the license may vary. Most packs are under GPL-3.0, or MIT. Please check the pack folder or the pages on official sites (Modrinth, CurseForge) for the license.

All* actions are licensed under AGPL-3.0 and written in Rust/Typescript/Go.
