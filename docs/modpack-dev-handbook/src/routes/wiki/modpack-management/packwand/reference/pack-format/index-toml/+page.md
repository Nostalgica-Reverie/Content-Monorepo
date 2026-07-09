# index.toml

The index file of the modpack, storing references to every file to be downloaded (or verified) in the pack.

## `hash-format`

**String, required.** The default [hash format](/wiki/modpack-management/packwand/reference/pack-format/pack-toml#hash-formats) for every file in the index. If missing, consumers assume `sha512`.

packwand writes `sha512`; when it loads an index using an older format it transparently upgrades the index to `sha512` on the next `packwand refresh`.

## `[[files]]`

**Array of tables, optional (defaults to an empty list).** One entry per file in the pack.

| Key | Type | Description |
| --- | --- | --- |
| `file` | path, required | The path to the file, relative to the index file, in forward-slash format. |
| `hash` | string | The hash of the file, in the index's `hash-format` (or this entry's override). May be omitted when `no-internal-hashes` is enabled. |
| `hash-format` | string | Overrides the index-level hash format for this file only. Omitted when equal to the index's format, to save space. |
| `metafile` | boolean, default `false` | True when this entry points to a `.pw.toml` [metadata file](/wiki/modpack-management/packwand/reference/pack-format/mod-toml), which references a file outside the pack. |
| `preserve` | boolean, default `false` | When true, the file is not overwritten on update if it already exists, preserving user changes. |
| `alias` | string | The name with which this file should be downloaded, instead of the filename in `file`. Not compatible with `metafile`. Multiple entries may share the same `file` with different aliases. |

Entries are sorted by `file` (then `alias`) when packwand writes the index, so diffs stay stable under version control.

## Ignored files

Files matching the pack's [`.packwizignore`](/wiki/modpack-management/packwand/reference/pack-format/packwizignore) rules (or the built-in defaults) are never added to the index. The pack file, the index itself, and `.packwizignore` are always excluded.

## Example

```toml
hash-format = "sha512"

[[files]]
file = "config/mymod.cfg"
hash = "..."

[[files]]
file = "mods/mymod.pw.toml"
hash = "..."
metafile = true
```
