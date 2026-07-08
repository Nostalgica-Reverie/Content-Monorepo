## packwand migrate

Migrate Minecraft/loader versions or pack-format to a newer generation.

```
packwand migrate [minecraft|loader|format] [flags]
```

### Options

```
  -h, --help   help for migrate
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
* [packwand migrate format](packwand_migrate_format.md)	 - Upgrade pack-format from packwiz:1.1.0 to packwand:26
* [packwand migrate loader](packwand_migrate_loader.md)	 - Migrate every configured modloader to a newer version.
* [packwand migrate minecraft](packwand_migrate_minecraft.md)	 - Migrate your Minecraft version to a newer version.

