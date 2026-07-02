## packwand completion

Generate the autocompletion script for the specified shell

### Synopsis

Generate the autocompletion script for packwand for the specified shell.
See each sub-command's help for details on how to use the generated script.


### Options

```
  -h, --help   help for completion
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

* [packwand](packwand.md)	 - Minecraft modpack toolchain â€” packwiz core with multi-pack workspace management
* [packwand completion bash](packwand_completion_bash.md)	 - Generate the autocompletion script for bash
* [packwand completion fish](packwand_completion_fish.md)	 - Generate the autocompletion script for fish
* [packwand completion powershell](packwand_completion_powershell.md)	 - Generate the autocompletion script for powershell
* [packwand completion zsh](packwand_completion_zsh.md)	 - Generate the autocompletion script for zsh

