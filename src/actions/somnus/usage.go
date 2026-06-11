package main

var verbUsage = map[string]string{
	"init": `usage: somnus init <category> <name> [--mc <version>] [--loader fabric|forge|neoforge|quilt] [--base | --consumes <id>] [--variants a,b,c]
  category: modpacks | datapacks | resourcepacks
  e.g. somnus init modpacks re-console-lite --mc 1.21.1 --loader fabric`,

	"bump": `usage: somnus bump <pack-dir> <new-version> [--configs]
  e.g. somnus bump modpacks/re-console-plus 26.06.1
  --configs  also update in-pack version configs (main menu credits, loader dependency overrides) and refresh`,

	"export": `usage: somnus export [pack]
  build changed packs locally (or one named pack); artifacts land in artifacts/
  e.g. somnus export re-console-plus`,

	"build": `usage: somnus build <short-sha> | somnus build --pack <name> <short-sha>
  CI entry: builds git-changed packs (or one named pack), tagging artifacts with the sha
  e.g. somnus build a1b2c3d`,

	"sync": `usage: somnus sync [--dry-run]
  propagate performance bases into consumers per manifest mappings
  --dry-run  show what would be copied, pruned, and refreshed without changing anything`,

	"update": `usage: somnus update
  packwiz update --all in every pack subdir (honors opt-out.json auto_update)`,

	"refresh": `usage: somnus refresh
  packwiz refresh in every pack subdir`,

	"loader-update": `usage: somnus loader-update [latest|recommended]
  migrate loaders across all packs (honors opt-out.json auto_update); default: latest`,

	"modlist": `usage: somnus modlist <pack-subdir>
  write crash-assistant modlist.json derived from the subdir's .pw.toml files
  e.g. somnus modlist modpacks/re-console-plus/26.1.2-mr`,

	"pages": `usage: somnus pages [pack]   (alias: docs)
  write modlist.md in every mod subdir; full runs (no pack argument) also emit projects.json for the docs site`,

	"test": `usage: somnus test <pack-subdir>   (alias: instance)
  packwiz serve + packwiz-installer into a local test instance
  requires $PACKWIZ_INSTALLER_JAR; honors $SOMNUS_TEST_INSTANCE
  e.g. somnus test modpacks/re-console-plus/26.1.2-mr`,

	"lint": `usage: somnus lint [files...]
  syntax-lint JSON / .pw.toml files; with no arguments, lints git-changed files`,

	"port": `usage: somnus port <mr-subdir> <cf-subdir> [--add] [--no-refresh]
  diff MR mods against the CF side; --add ports missing ones interactively
  --no-refresh  defer index rebuilds during --add, refresh once at the end (requires packwiz-tx via PACKWIZ_BIN)
  e.g. somnus port modpacks/rc-plus/26.1.2-mr modpacks/rc-plus/26.1.2-cf`,

	"doctor": `usage: somnus doctor   (alias: check)
  verify tools (git, packwiz, java, zip), repo root, manifest health, and legacy opt-out files`,

	"help": `usage: somnus help [verb]
  show the full verb list, or detailed usage for one verb`,

	"version": `usage: somnus version
  print the somnus version`,
}
