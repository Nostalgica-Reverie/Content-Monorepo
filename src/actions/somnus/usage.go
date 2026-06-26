package main

var verbUsage = map[string]string{
	"init": `usage: somnus init <category> <name> [--mc <version>] [--loader fabric|forge|neoforge|quilt] [--base | --consumes <id>] [--variants a,b,c]
  category: modpacks | datapacks | resourcepacks
  e.g. somnus init modpacks re-console-lite --mc 1.21.1 --loader fabric`,

	"bump": `usage: somnus bump <pack-dir> <new-version> [--configs]
  e.g. somnus bump modpacks/re-console-plus 26.06.1
  --configs  also update in-pack version configs (main menu credits, loader dependency overrides) and refresh`,

	"packs": `usage: somnus packs list | get <id> [field] | set <id> <field> <value> | index
  the pack registry: every manifest, addressable as an object
  'packs index' regenerates the derived <category>/Project.json files (also done by full 'pages' runs)
  e.g. somnus packs list
       somnus packs set re-console-plus release_type beta`,

	"freeze": `usage: somnus freeze <pack-subdir> [mod-slugs...]
  pin mods in ONE subdir (e.g. modpacks/x/26.1.2-mr) so 'somnus update' skips them
  with no slugs: list that subdir's frozen mods
  recorded in the pack manifest (automation.freeze); doctor flags drift
  e.g. somnus freeze modpacks/re-console-plus/26.1.2-mr sodium iris`,

	"unfreeze": `usage: somnus unfreeze <pack-subdir> <mod-slugs...>
  unpin previously frozen mods so updates apply to them again`,

	"export": `usage: somnus export [pack]
  build changed packs locally (or one named pack); artifacts land in artifacts/
  e.g. somnus export re-console-plus`,

	"build": `usage: somnus build <short-sha> | somnus build --pack <name> <short-sha>
  CI entry: builds git-changed packs (or one named pack), tagging artifacts with the sha
  e.g. somnus build a1b2c3d`,

	"sync": `usage: somnus sync [--dry-run]
  propagate performance bases into consumers per manifest mappings
  --dry-run  show what would be copied, pruned, and refreshed without changing anything`,

	"update": `usage: somnus update [pack-dir] [--all]
  packwiz update --all in every pack subdir (honors automation.auto_update)
  with a pack-dir: just that pack (overrides its opt-out — explicit beats config)
  run from inside a pack: scopes to it automatically; --all forces everything`,

	"refresh": `usage: somnus refresh [pack-dir] [--all]
  packwiz refresh in every pack subdir; same scoping rules as update`,

	"loader-update": `usage: somnus loader-update [latest|recommended] [pack-dir] [--all]
  migrate loaders (honors automation.auto_update); same scoping rules as update`,

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

	"import": `usage: somnus import <url-or-mrpack-file> [--id <pack-id>]
  import a Modrinth .mrpack into modpacks/ as a ready pack ({mc}-mr subdir, manifest, changelog)
  update metadata is reconstructed from cdn.modrinth.com URLs, so imported mods stay updatable
  e.g. somnus import https://cdn.modrinth.com/data/1KVo5zza/versions/g5RAIwpP/Fabulously.Optimized-v13.2.2.mrpack`,

	"side": `usage: somnus side <pack-dir> <mod-slug> [client|server|both]
  with no side: show the mod's current side per subdir
  with a side: rewrite it in every subdir's .pw.toml and refresh (fixes mislabeled mods)
  e.g. somnus side modpacks/re-console-plus some-mislabeled-mod client`,

	"publish": `usage: somnus publish <list|build|upload|verify> <manifest...> [variant] [--live]
  the release pipeline (Go port of the Rust publisher; same outputs and artifact names)
  list <manifests...>          print matrix entries as JSON (multiple manifests concatenate)
  build <manifest> [variant]   export artifacts into <pack>/artifacts/, write GITHUB_OUTPUT metadata
  upload <manifest> [variant]  send artifacts to Modrinth/CurseForge — DRY RUN unless --live
  verify <manifest> [variant]  assert the version landed on Modrinth (post-upload CI check)`,

	"packwiz": `usage: somnus packwiz build [--output <path>]
  clone packwiz at the pinned SHA, apply all patches in patches/, and build the binary
  default output: ./packwiz-bin/packwiz (./packwiz-bin/packwiz.exe on Windows)
  set PACKWIZ_BIN to the output path so somnus commands pick up the patched binary
  e.g. somnus packwiz build --output ./bin/packwiz`,

	"doctor": `usage: somnus doctor   (alias: check)
  verify tools (git, packwiz, java, zip), repo root, manifest health, and legacy opt-out files`,

	"validate": `usage: somnus validate <path/to/manifest.json> [more...] | somnus validate --all
  validate pack manifest(s): required fields, type, platform subdirs, changelog, role, automation
  --all  discover and validate every manifest under modpacks/, datapacks/, resourcepacks/
  e.g. somnus validate modpacks/re-console-plus/manifest.json`,

	"automation": `usage: somnus automation get <pack-dir>
  output the effective automation settings for a pack as JSON
  merges manifest.json automation field with legacy opt-out.json (if present)
  e.g. somnus automation get modpacks/re-console-plus`,

	"help": `usage: somnus help [verb]
  show the full verb list, or detailed usage for one verb`,

	"version": `usage: somnus version
  print the somnus version`,
}
