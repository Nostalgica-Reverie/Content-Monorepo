## packwand modrinth add

Add a project from a Modrinth URL, slug/project ID or search

```
packwand modrinth add [URL|slug|search] [flags]
```

### Options

```
  -h, --help                      help for add
      --project-id string         The Modrinth project ID to use
      --version-filename string   The Modrinth version filename to use
      --version-id string         The Modrinth version ID to use
```

### Options inherited from parent commands

```
      --cache string              The directory where packwiz will cache downloaded mods (default "C:\\Users\\jmtmm\\AppData\\Local\\packwand\\cache")
      --config string             The config file to use (default: .packwand.toml in your platform config directory)
      --meta-folder string        The folder in which new metadata files will be added, defaulting to a folder based on the category (mods, resourcepacks, etc; if the category is unknown the current directory is used)
      --meta-folder-base string   The base folder from which meta-folder will be resolved, defaulting to the current directory (so you can put all mods/etc in a subfolder while still using the default behaviour) (default ".")
      --no-refresh                Skip index and pack.toml refresh after modifications (use 'packwand refresh' to finalize batch operations)
      --pack-file string          The modpack metadata file to use (default "pack.toml")
  -y, --yes                       Accept all prompts with the default or "yes" option (non-interactive mode) - may pick unwanted options in search results
```

### SEE ALSO

* [packwand modrinth](packwand_modrinth.md)	 - Manage modrinth-based mods

