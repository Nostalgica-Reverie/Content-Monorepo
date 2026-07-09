## packwand settings acceptable-versions

Manage your pack's acceptable Minecraft versions. This must be a comma separated list of Minecraft versions, e.g. 1.16.3,1.16.4,1.16.5

```
packwand settings acceptable-versions [flags]
```

### Options

```
  -a, --add      Add a version to the list
  -h, --help     help for acceptable-versions
  -r, --remove   Remove a version from the list
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

* [packwand settings](packwand_settings.md)	 - Manage pack settings

