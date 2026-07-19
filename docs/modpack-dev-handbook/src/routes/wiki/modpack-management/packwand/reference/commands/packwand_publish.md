## packwand publish

Build, upload, verify, or list publish targets for a project

```
packwand publish [flags]
```

### Options

```
  -h, --help   help for publish
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

* [packwand](packwand.md)	 - Minecraft modpack toolchain — packwiz core with multi-pack workspace management
* [packwand publish build](packwand_publish_build.md)	 - Build the project artifact(s) for publishing
* [packwand publish list](packwand_publish_list.md)	 - Enumerate all (manifest, variant) publish pairs as JSON (for CI matrix)
* [packwand publish plan](packwand_publish_plan.md)	 - Compute the publish matrix from git changes, with include/skip reasons (JSON on stdout)
* [packwand publish upload](packwand_publish_upload.md)	 - Upload pre-built artifacts to Modrinth and/or CurseForge
* [packwand publish verify](packwand_publish_verify.md)	 - Verify a published version exists live on Modrinth

