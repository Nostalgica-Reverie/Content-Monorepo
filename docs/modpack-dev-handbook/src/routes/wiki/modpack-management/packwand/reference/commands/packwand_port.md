## packwand port

Compare MR and CF subdirs and port missing mods from Modrinth to CurseForge

```
packwand port <mr-subdir> <cf-subdir> [flags]
```

### Options

```
      --add          Interactively add missing CurseForge entries via packwand
  -h, --help         help for port
      --json         Output missing list as JSON (dry-run only)
      --no-refresh   Batch the refresh until the end
```

### Options inherited from parent commands

```
      --cache string              The directory where packwiz will cache downloaded mods (default "C:\\Users\\jmtmm\\AppData\\Local\\packwand\\cache")
      --config string             The config file to use (default: .packwand.toml in your platform config directory)
      --meta-folder string        The folder in which new metadata files will be added, defaulting to a folder based on the category (mods, resourcepacks, etc; if the category is unknown the current directory is used)
      --meta-folder-base string   The base folder from which meta-folder will be resolved, defaulting to the current directory (so you can put all mods/etc in a subfolder while still using the default behaviour) (default ".")
      --pack-file string          The modpack metadata file to use (default "pack.toml")
  -y, --yes                       Accept all prompts with the default or "yes" option (non-interactive mode) - may pick unwanted options in search results
```

### SEE ALSO

* [packwand](packwand.md)	 - Minecraft modpack toolchain — packwiz core with multi-pack workspace management

