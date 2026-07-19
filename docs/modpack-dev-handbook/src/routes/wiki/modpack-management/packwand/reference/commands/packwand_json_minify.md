## packwand json minify

Minify .json/.mcmeta files in place (recurses into directories); invalid JSON is skipped with a warning

### Synopsis

Strips insignificant whitespace from JSON files so built and published artifacts ship smaller. Directories are walked recursively for .json and .mcmeta files; .git and node_modules are never entered. Files that do not parse as strict JSON (e.g. JSON5/commented configs) are skipped with a warning unless --strict is set. Key order and number formatting are preserved.

```
packwand json minify <path...> [flags]
```

### Options

```
      --check    Report files that would shrink and exit 1 instead of rewriting them
  -h, --help     help for minify
      --strict   Fail on files that are not valid JSON instead of skipping them
```

### Options inherited from parent commands

```
      --cache string              The directory where packwiz will cache downloaded mods (default: your platform cache directory)
      --config string             The config file to use (default: .packwand.toml in your platform config directory)
      --meta-folder string        The folder in which new metadata files will be added, defaulting to a folder based on the category (mods, resourcepacks, etc; if the category is unknown the current directory is used)
      --meta-folder-base string   The base folder from which meta-folder will be resolved, defaulting to the current directory (so you can put all mods/etc in a subfolder while still using the default behaviour) (default ".")
      --no-refresh                Skip index and pack.toml refresh after modifications (use 'packwand refresh' to finalize batch operations)
      --pack-file string          The modpack metadata file to use (default "pack.toml")
  -y, --yes                       Accept all prompts with the default or "yes" option (non-interactive mode) - may pick unwanted options in search results
```

### SEE ALSO

* [packwand json](packwand_json.md)	 - JSON utilities for pack files

