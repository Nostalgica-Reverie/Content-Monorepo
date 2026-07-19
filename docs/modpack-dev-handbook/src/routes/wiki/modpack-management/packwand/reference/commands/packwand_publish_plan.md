## packwand publish plan

Compute the publish matrix from git changes, with include/skip reasons (JSON on stdout)

```
packwand publish plan [flags]
```

### Options

```
      --from string   Base git ref to compare manifests against (default: <to>^)
  -h, --help          help for plan
      --no-validate   Skip running 'packwand validate' on included manifests
      --pack string   Limit the plan to one repository-relative pack directory
      --to string     Target git ref whose manifests are candidates for publishing (default "HEAD")
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

* [packwand publish](packwand_publish.md)	 - Build, upload, verify, or list publish targets for a project

