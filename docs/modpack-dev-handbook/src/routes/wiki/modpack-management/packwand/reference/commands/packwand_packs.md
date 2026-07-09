## packwand packs

Look up or edit any pack's manifest fields by id

```
packwand packs [flags]
```

### Options

```
  -h, --help   help for packs
      --json   Output as JSON
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

* [packwand](packwand.md)	 - Minecraft modpack toolchain — packwiz core with multi-pack workspace management
* [packwand packs get](packwand_packs_get.md)	 - Print a pack's manifest (or a single field)
* [packwand packs index](packwand_packs_index.md)	 - Regenerate derived projects.json index files
* [packwand packs list](packwand_packs_list.md)	 - List all registered packs
* [packwand packs set](packwand_packs_set.md)	 - Set a simple manifest field for a pack

