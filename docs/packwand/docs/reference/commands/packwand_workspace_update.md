## packwand workspace update

Run packwand update --all in every pack subdir (honors auto_update)

```
packwand workspace update [pack-dir] [flags]
```

### Options

```
      --all             Run across all packs even when scoped
      --check           Show what would update without applying (dry-run)
  -h, --help            help for update
      --ignored-only    With --check, check packs opted out of auto-update instead of the normal set
      --json            With --check, output a JSON summary instead of plain text
      --report string   Write an aggregated machine-readable JSON update report to this file
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

* [packwand workspace](packwand_workspace.md)	 - Multi-pack workspace operations across all packs

