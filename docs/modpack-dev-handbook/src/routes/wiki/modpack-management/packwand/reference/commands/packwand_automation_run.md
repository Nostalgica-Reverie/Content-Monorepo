## packwand automation run

Run the unattended release pipeline for a full_auto-enabled pack (update, refresh, validate, tests, docs, bump)

### Synopsis

Runs update -> refresh -> validate -> tests -> docs -> bump for a single pack that has opted in via manifest.json "automation": { "full_auto": { "enabled": true } }. Stops after bumping the manifest in the working tree — it never commits, builds, or publishes. Committing and pushing the result is left to the caller (CI); pushing a version-bumped manifest to main is what the existing 'packwand publish plan' / publish.yml pipeline already reacts to.

```
packwand automation run <pack-dir> [flags]
```

### Options

```
      --dry-run         Run update/refresh/validate/tests/docs but skip the version bump
  -h, --help            help for run
      --json            Print the run report as JSON instead of text
      --report string   Write the JSON run report to this file
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

* [packwand automation](packwand_automation.md)	 - Query effective automation settings for a pack

