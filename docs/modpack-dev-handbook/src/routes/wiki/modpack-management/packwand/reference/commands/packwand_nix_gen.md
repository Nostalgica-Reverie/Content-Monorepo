## packwand nix gen

Generate a packwiz2nix checksums.json for this pack (or --all packs)

### Synopsis

Generates the checksums.json consumed by lib/packwiz2nix (mkPackwizPackages,
mkModLinks, mkMultiMCPack) from the currently loaded pack. Mod files are
resolved through packwand's download cache, so already-fetched files are
hashed for free and new files are downloaded and verified against their
metadata hashes first.

Only URL-mode mods in the mods/ folder are included; CurseForge metadata-mode
files have no static URL and are skipped with a warning.

```
packwand nix gen [flags]
```

### Options

```
      --all             Generate for every pack subdir in the workspace (run from the repo root)
  -h, --help            help for gen
      --output string   Path to write the checksums file to, relative to the pack directory (default "checksums.json")
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

* [packwand nix](packwand_nix.md)	 - Nix integration (packwiz2nix-compatible outputs)

