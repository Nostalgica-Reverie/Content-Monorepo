## packwand

Minecraft modpack toolchain — packwiz core with multi-pack workspace management

```
packwand [flags]
```

### Options

```
      --cache string              The directory where packwiz will cache downloaded mods (default "C:\\Users\\jmtmm\\AppData\\Local\\packwand\\cache")
      --config string             The config file to use (default: .packwand.toml in your platform config directory)
  -h, --help                      help for packwand
      --meta-folder string        The folder in which new metadata files will be added, defaulting to a folder based on the category (mods, resourcepacks, etc; if the category is unknown the current directory is used)
      --meta-folder-base string   The base folder from which meta-folder will be resolved, defaulting to the current directory (so you can put all mods/etc in a subfolder while still using the default behaviour) (default ".")
      --no-refresh                Skip index and pack.toml refresh after modifications (use 'packwand refresh' to finalize batch operations)
      --pack-file string          The modpack metadata file to use (default "pack.toml")
  -y, --yes                       Accept all prompts with the default or "yes" option (non-interactive mode) - may pick unwanted options in search results
```

### SEE ALSO

* [packwand add](packwand_add.md)	 - Add a mod to all (or a specific) pack's Modrinth and CurseForge subdirs
* [packwand api](packwand_api.md)	 - Run and inspect the Packwand HTTP API
* [packwand automation](packwand_automation.md)	 - Query effective automation settings for a pack
* [packwand build](packwand_build.md)	 - Build modpack exports and zip packs from git-changed targets (CI mode)
* [packwand bump](packwand_bump.md)	 - Bump the manifest version (--configs also updates in-pack version files)
* [packwand cache](packwand_cache.md)	 - Inspect and maintain the shared download cache
* [packwand completion](packwand_completion.md)	 - Generate the autocompletion script for the specified shell
* [packwand content-lint](packwand_content-lint.md)	 - Lint pack content — namespaces, texture/model refs, pack.mcmeta, function tags, duplicate and case-colliding files
* [packwand curseforge](packwand_curseforge.md)	 - Manage curseforge-based mods
* [packwand diff](packwand_diff.md)	 - Show mod additions, removals, and updates between two git refs
* [packwand doctor](packwand_doctor.md)	 - Check that tools, repo root, and manifests are all healthy
* [packwand export](packwand_export.md)	 - Export packs locally (like build but uses 'local' as the SHA suffix)
* [packwand forgejo](packwand_forgejo.md)	 - Manage projects released on Forgejo, Gitea, or Codeberg
* [packwand freeze](packwand_freeze.md)	 - Pin mods so updates skip them (no slugs: list what's frozen)
* [packwand github](packwand_github.md)	 - Manage projects released on GitHub
* [packwand gitlab](packwand_gitlab.md)	 - Manage projects released on GitLab or self-hosted GitLab instances
* [packwand gui](packwand_gui.md)	 - Run the local Packwand web GUI
* [packwand import](packwand_import.md)	 - Import an .mrpack or CurseForge zip as a new modpack
* [packwand init](packwand_init.md)	 - Initialise a packwiz modpack
* [packwand lint](packwand_lint.md)	 - Check JSON and .pw.toml files for syntax errors (no args: lints git-changed files)
* [packwand list](packwand_list.md)	 - List all the mods in the modpack
* [packwand migrate](packwand_migrate.md)	 - Migrate Minecraft/loader versions or pack-format to a newer generation.
* [packwand modlist](packwand_modlist.md)	 - Write a crash-assistant modlist.json from a pack's mods/ directory
* [packwand modrinth](packwand_modrinth.md)	 - Manage modrinth-based mods
* [packwand new](packwand_new.md)	 - Scaffold a new pack (manifest.json, changelog.md, packwiz subdirs)
* [packwand nix](packwand_nix.md)	 - Nix integration (packwiz2nix-compatible outputs)
* [packwand packs](packwand_packs.md)	 - Look up or edit any pack's manifest fields by id
* [packwand pages](packwand_pages.md)	 - Regenerate modlist.md files for all packs (or a single pack) and the projects index
* [packwand pin](packwand_pin.md)	 - Pin a file so it does not get updated automatically
* [packwand port](packwand_port.md)	 - Compare MR and CF subdirs and port missing mods from Modrinth to CurseForge
* [packwand publish](packwand_publish.md)	 - Build, upload, verify, or list publish targets for a pack
* [packwand refresh](packwand_refresh.md)	 - Refresh the index file
* [packwand rehash](packwand_rehash.md)	 - Migrate all hashes to a specific format
* [packwand remove](packwand_remove.md)	 - Remove an external file from the modpack; equivalent to manually removing the file and running packwiz refresh
* [packwand run](packwand_run.md)	 - Execute a user-defined script from the [scripts] section of pack.toml
* [packwand serve](packwand_serve.md)	 - Run a local development server
* [packwand settings](packwand_settings.md)	 - Manage pack settings
* [packwand side](packwand_side.md)	 - Check or fix a mod's side across all subdirs in a pack
* [packwand test](packwand_test.md)	 - Spin up packwand serve and run packwiz-installer against it to validate a pack
* [packwand unfreeze](packwand_unfreeze.md)	 - Unpin mods so updates can apply to them again
* [packwand unpin](packwand_unpin.md)	 - Unpin a file so it receives updates
* [packwand update](packwand_update.md)	 - Update an external file (or all external files) in the modpack
* [packwand url](packwand_url.md)	 - Add external files from a direct download link, for sites that are not directly supported by packwiz
* [packwand utils](packwand_utils.md)	 - Utilities for managing packwiz itself
* [packwand validate](packwand_validate.md)	 - Validate pack manifests — fields, subdirs, changelog, role, automation
* [packwand version](packwand_version.md)	 - Print the packwand version
* [packwand workspace](packwand_workspace.md)	 - Multi-pack workspace operations across all packs

