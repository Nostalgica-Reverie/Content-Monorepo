# mod.pw.toml

A metadata file which references an external file from a URL (or a metadata-based downloader). This allows for side-only mods, optional mods, and pinning, and stores metadata to allow finding updates on Modrinth, CurseForge, GitHub, GitLab, and Forgejo. The "mod" terminology is used a lot here, but this works for any file — resource packs, shader packs, datapacks, and plain files.

Metadata files use the `.pw.toml` extension and are marked with `metafile = true` in the [index](/reference/pack-format/index-toml).

## `name`

**String, required.** The name of the mod, displayed in user interfaces. Does not need to be unique.

## `filename`

**Path, required.** The destination filename of the downloaded file, relative to this metadata file.

## `side`

**String, default `"both"`.** The physical Minecraft side this file should be installed on: `"client"` (client and integrated server), `"server"` (dedicated server), or `"both"`. An empty string is equivalent to `"both"`.

## `pin`

**Boolean, default `false`.** _(packwand extension.)_ When true, the file is pinned: `packwand update` skips it until it is unpinned (`packwand pin <mod>` / `packwand unpin <mod>`).

## `[download]`

**Table, required.** How to obtain the file.

| Key           | Type             | Description                                                                                                                                                                                                                                                    |
| ------------- | ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `url`         | string           | The URL to download from. Required when `mode` is `"url"` or omitted.                                                                                                                                                                                          |
| `mode`        | string           | The download mode. `"url"` (or omitted/empty) downloads from `url`. `"metadata:curseforge"` resolves the download through the CurseForge API using the `[update.curseforge]` metadata — required by CurseForge's distribution rules; such files have no `url`. |
| `hash-format` | string, required | The [hash format](/reference/pack-format/pack-toml#hash-formats) of `hash`. packwand writes `sha512` where the source provides it.                                                                                                                             |
| `hash`        | string, required | The hash of the file, used for integrity verification.                                                                                                                                                                                                         |

## `[option]`

**Table, optional.** The optional state of this file. When absent, the file is not optional.

| Key           | Type                               | Description                                                                                                                                                                    |
| ------------- | ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `optional`    | boolean, required, default `false` | Whether the file is optional.                                                                                                                                                  |
| `description` | string                             | Shown to the user when selecting optional mods; should explain why they might want it.                                                                                         |
| `default`     | boolean, default `false`           | Whether the file is enabled by default. If a target pack format does not support optional mods but supports disabled mods, files defaulting to disabled are exported disabled. |

## `[update]`

**Table, optional.** How tools may update this file. If absent or empty, the file is never auto-updated. Each sub-table is one update source; if several are defined, the tool chooses one (which one is implementation-defined — do not rely on the order).

Consumers must fail to load a metadata file that declares an update source they do not recognise.

### `[update.curseforge]`

| Key          | Type              | Description                                                                                                                                |
| ------------ | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `project-id` | integer, required | The CurseForge project ID. Updating retrieves the latest valid file for this project (matching game version, release channel, and loader). |
| `file-id`    | integer, required | The currently-installed file ID.                                                                                                           |

### `[update.modrinth]`

| Key       | Type             | Description                         |
| --------- | ---------------- | ----------------------------------- |
| `mod-id`  | string, required | The Modrinth project ID.            |
| `version` | string, required | The currently-installed version ID. |

### `[update.github]`

_(packwand extension.)_ Updates from GitHub release assets.

| Key      | Type             | Description                                                       |
| -------- | ---------------- | ----------------------------------------------------------------- |
| `slug`   | string, required | The repository, as `owner/repo`.                                  |
| `tag`    | string           | The currently-installed release tag.                              |
| `branch` | string           | Restrict updates to releases targeting this branch.               |
| `regex`  | string           | A regular expression an asset filename must match to be selected. |

### `[update.gitlab]`

_(packwand extension.)_ Updates from GitLab release assets.

| Key        | Type             | Description                                                       |
| ---------- | ---------------- | ----------------------------------------------------------------- |
| `instance` | string           | The GitLab instance hostname; defaults to `gitlab.com`.           |
| `slug`     | string, required | The project path, as `owner/repo`.                                |
| `tag`      | string           | The currently-installed release tag.                              |
| `regex`    | string           | A regular expression an asset filename must match to be selected. |

### `[update.forgejo]`

_(packwand extension.)_ Updates from Forgejo/Gitea release assets (including Codeberg).

| Key        | Type             | Description                                                       |
| ---------- | ---------------- | ----------------------------------------------------------------- |
| `instance` | string           | The Forgejo/Gitea instance hostname; defaults to `codeberg.org`.  |
| `slug`     | string, required | The repository, as `owner/repo`.                                  |
| `tag`      | string           | The currently-installed release tag.                              |
| `branch`   | string           | Restrict updates to releases targeting this branch.               |
| `regex`    | string           | A regular expression an asset filename must match to be selected. |

## Example

```toml
name = "Borderless Mining"
filename = "borderless-mining-1.1.5+1.19.jar"
side = "client"

[download]
url = "https://cdn.modrinth.com/data/kYq5qkSL/versions/1.1.5+1.19/borderless-mining-1.1.5%2B1.19.jar"
hash-format = "sha512"
hash = "..."

[update]
[update.modrinth]
mod-id = "kYq5qkSL"
version = "gqoXgtxO"
```

A CurseForge metadata-mode file:

```toml
name = "Example Mod"
filename = "examplemod-1.0.jar"

[download]
mode = "metadata:curseforge"
hash-format = "murmur2"
hash = "1234567890"

[update]
[update.curseforge]
file-id = 3643025
project-id = 327154
```
