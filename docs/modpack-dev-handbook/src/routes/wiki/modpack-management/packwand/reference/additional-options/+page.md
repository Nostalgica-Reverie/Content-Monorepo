# Additional options

Additional options can be configured in the `[options]` section of `pack.toml`, as follows:

- `acceptable-game-versions` A list of additional Minecraft versions to accept when installing or updating mods (see [Adding mods](/wiki/modpack-management/packwand/tutorials/creating/adding-mods))
- `acceptable-game-loaders` A list of additional mod loaders to accept when installing or updating mods, beyond those implied by the pack's `[versions]` (quilt already accepts fabric mods, and neoforge accepts forge mods)
- `meta-folder` The folder in which new metadata files will be added, defaulting to a folder based on the category (mods, resourcepacks, etc; if the category is unknown the current directory is used)
  - `mods-folder` is deprecated; aliased to `meta-folder`
- `meta-folder-base` The base folder from which meta-folder will be resolved, defaulting to the current directory (so you can put all mods/etc in a subfolder while still using the default behaviour)
- `no-internal-hashes` If this is set to true, packwand will not generate hashes of local files, to prevent merge conflicts and inconsistent hashes when using git/etc.
  - `packwand refresh --build` can be used in this mode to generate internal hashes for distributing the pack with [packwiz-installer](/wiki/modpack-management/packwand/tutorials/installing/packwiz-installer)
- `datapack-folder` The folder in which datapacks are to be added; specific to the datapack loader mod you use, and must be set to add datapacks (that are not bundled as mods)

## Scripts

Packs can define runnable scripts in a `[scripts]` section of `pack.toml`, executed with `packwand run <name>`:

```toml
[scripts]
lint = "packwand refresh && packwand validate manifest.json"
```

## Global configuration

These are set in packwand's own config file (`.packwand.toml` in your platform config directory) or via flags/environment, not in `pack.toml`:

- `cache.directory` Overrides the download cache location (also the `--cache` global flag)
- `github.token` A GitHub API token, to avoid rate limits when installing/updating GitHub mods
- `gitlab.token` / `gitlab.<instance>.token` GitLab API token(s)
- `forgejo.token` / `forgejo.<instance>.token` Forgejo/Gitea/Codeberg API token(s)

## Environment variables

- `PACKWAND_CONCURRENCY` Cap on parallel workers for workspace operations (`SOMNUS_CONCURRENCY` is still honored for existing automation)
- `PACKWAND_NETWORK_CONCURRENCY` Cap on parallel API/download requests
- `PACKWAND_HASH_CONCURRENCY` Cap on parallel local hashing
- `PACKWAND_CACHE_SLOTS` Cap on concurrent export operations against the pack cache
- `PACKWAND_BIN` Path to the packwand binary, used by tooling that shells out to packwand (`PACKWIZ_BIN` is deprecated but still honored)
- `MODPACKS_DIR` Overrides the workspace pack root (default `modpacks`)
