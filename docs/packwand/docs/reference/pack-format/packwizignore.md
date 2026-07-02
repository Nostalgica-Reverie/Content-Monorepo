# .packwizignore

`.packwizignore` is an optional file at the root of a pack that excludes files from the pack index, using the [same format as gitignore](https://git-scm.com/docs/gitignore). Place patterns in it (one per line) and run `packwand refresh`; matching files are not added to `index.toml` and are not distributed with the pack.

The pack file (`pack.toml`), the index file, and `.packwizignore` itself are always excluded.

## Default rules

The following defaults are always applied, whether or not a `.packwizignore` file exists. They can be overridden with a negating pattern (preceded with `!`):

```txt
# Git metadata
.git/**
.gitattributes
.gitignore

# macOS metadata
.DS_Store

# Exported CurseForge zip files
/*.zip

# Exported Modrinth packs
*.mrpack

# Tool binaries placed in the pack folder
packwiz.exe
packwiz
packwand.exe
packwand
```
