## packwand forgejo add

Add a project from a Forgejo/Gitea/Codeberg repository URL or slug

```
packwand forgejo add [URL|slug] [flags]
```

### Options

```
      --branch string     Repository branch to retrieve releases for
  -h, --help              help for add
      --instance string   Forgejo/Gitea instance hostname (default: codeberg.org)
      --regex string      Regular expression to match release assets against
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

* [packwand forgejo](packwand_forgejo.md)	 - Manage projects released on Forgejo, Gitea, or Codeberg

