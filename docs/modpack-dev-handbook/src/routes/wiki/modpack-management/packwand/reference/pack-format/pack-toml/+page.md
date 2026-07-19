# pack.toml

The main metadata file for a packwand modpack. This is the first file loaded, so that a modpack downloader can download all the files in the modpack.

## `pack-format`

**String, required for new packs.** A version string identifying the pack format. packwand writes `packwand:26` for new packs.

Two families of values are accepted:

- **`packwand:<generation>`** — the packwand format. The suffix is a single integer generation number (currently `26`).
  - Consumers must fail to load the pack if the generation is not a valid integer.
  - Consumers must fail to load the pack if the generation predates the minimum they support; `packwand migrate format` upgrades old packs.
  - Consumers should warn (but continue) if the generation is newer than the version they implement.
- **`packwiz:<semver>`** — the legacy packwiz format, accepted for backward compatibility. The suffix must be valid [semver](https://semver.org/spec/v2.0.0.html); versions matching `~1` are accepted, and packs with a feature version above `1.1` produce an upgrade suggestion. `packwiz:1.0.0` is migrated to `packwiz:1.1.0` automatically on load.

If the field is missing entirely, consumers assume `packwiz:1.1.0` for compatibility with very old packs.

## `name`

**String, required.** The name of the modpack. Displayed in user interfaces to identify the pack; does not need to be unique between packs.

## `author`

**String, optional.** The author(s) of the modpack. Output when exporting to the CurseForge pack format.

## `version`

**String, optional.** The version of the modpack. Output when exporting to CurseForge and Modrinth pack formats. Must not be used to determine whether the modpack is outdated.

## `description`

**String, optional.** A short description of the modpack. Output when exporting to the Modrinth pack format.

## `[index]`

**Table, required.** Information about the [index file](/wiki/modpack-management/packwand/reference/pack-format/index-toml) of this modpack.

| Key | Type | Description |
| --- | --- | --- |
| `file` | path, required | The path to the index file, relative to `pack.toml` (forward slashes). Defaults to `index.toml` when empty. |
| `hash-format` | string, required | The [hash format](#hash-formats) of the index hash. packwand writes `sha512`. |
| `hash` | string | The hash of the generated index file. Omitted from source metadata; `packwand refresh --build` writes it for distribution. |

## `[versions]`

**Table of strings, required.** The versions of components used by this modpack — Minecraft and the mod loader(s). The existence of a component implies it should be installed; tools also use these values to decide which mod versions are compatible.

| Key | Description | Example |
| --- | --- | --- |
| `minecraft` | Required. The Minecraft version, in the format used by version.json files. | `"1.20.1"`, `"26.1.2"` |
| `fabric` | The Fabric loader version. | `"0.16.9"` |
| `forge` | The Forge version, without the Minecraft-version prefix. | `"14.23.5.2838"` |
| `neoforge` | The NeoForge version. | `"21.1.77"` |
| `quilt` | The Quilt loader version. | `"0.27.0"` |
| `liteloader` | The LiteLoader version. | `"1.12.2-SNAPSHOT"` |

Additional string keys are permitted. A pack with `quilt` is also considered compatible with `fabric` mods, and a pack with `neoforge` is also considered compatible with `forge` mods.

## `[options]`

**Table, optional.** Tool configuration read at load time; see [Additional options](/wiki/modpack-management/packwand/reference/additional-options). Keys include `acceptable-game-versions`, `acceptable-game-loaders`, `meta-folder`, `meta-folder-base`, `no-internal-hashes`, and `datapack-folder`.

## `[scripts]`

**Table of strings, optional.** *(packwand extension, not in packwiz.)* Named commands runnable with `packwand run <name>`:

```toml
[scripts]
postbuild = "echo done"
```

## `[export]`

**Table of tables, optional.** Per-platform export configuration, e.g. `[export.curseforge]` and `[export.modrinth]` settings used by the corresponding `export` commands.

## Hash formats

All hash values in the pack are lowercase strings. Consumers must support:

| Format | Notes |
| --- | --- |
| `sha512` | **Default.** Used by packwand for all new files and index entries. |
| `sha256` | Used as the download-cache key format. |
| `sha1` | Legacy; provided by some remote APIs. |
| `md5` | Legacy; provided by some remote APIs. |
| `murmur2` | The CurseForge variant: 32-bit MurmurHash2 (seed 1) with whitespace bytes (9, 10, 13, 32) removed before hashing, stored as an unsigned decimal integer. |

## Example

```toml
name = "My Modpack"
author = "Me"
version = "1.0.0"
pack-format = "packwand:26"

[index]
file = "index.toml"
hash-format = "sha512"
hash = "..."

[versions]
minecraft = "1.20.1"
fabric = "0.16.9"
```
