## packwand curseforge add

Add a project from a CurseForge URL, slug, ID or search

```
packwand curseforge add [URL|slug|search] [flags]
```

### Options

```
      --addon-id uint32   The CurseForge project ID to use
      --category string   The category to add files from (slug, as stored in URLs); the category in the URL takes precedence
      --file-id uint32    The CurseForge file ID to use
      --game string       The game to add files from (slug, as stored in URLs); the game in the URL takes precedence (default "minecraft")
  -h, --help              help for add
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

* [packwand curseforge](packwand_curseforge.md)	 - Manage curseforge-based mods

