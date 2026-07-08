## packwand completion zsh

Generate the autocompletion script for zsh

### Synopsis

Generate the autocompletion script for the zsh shell.

If shell completion is not already enabled in your environment you will need
to enable it.  You can execute the following once:

	echo "autoload -U compinit; compinit" >> ~/.zshrc

To load completions in your current shell session:

	source <(packwand completion zsh)

To load completions for every new session, execute once:

#### Linux:

	packwand completion zsh > "${fpath[1]}/_packwand"

#### macOS:

	packwand completion zsh > $(brew --prefix)/share/zsh/site-functions/_packwand

You will need to start a new shell for this setup to take effect.


```
packwand completion zsh [flags]
```

### Options

```
  -h, --help              help for zsh
      --no-descriptions   disable completion descriptions
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

* [packwand completion](packwand_completion.md)	 - Generate the autocompletion script for the specified shell

