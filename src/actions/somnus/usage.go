package main

var verbUsage = map[string]string{
	"init": `usage: somnus init <category> <name> [--mc <version>] [--loader fabric|forge|neoforge|quilt] [--base | --consumes <id>] [--variants a,b,c]
  sets up a fresh pack folder with a manifest, changelog, packwiz subdirs, and a .packwizignore
  category: modpacks | datapacks | resourcepacks
  e.g. somnus init modpacks re-console-lite --mc 1.21.1 --loader fabric`,

	"bump": `usage: somnus bump <pack-dir> <new-version> [--configs]
  bumps the version field in the manifest — add --configs to also update in-pack version files and refresh
  e.g. somnus bump modpacks/re-console-plus 26.06.1`,

	"packs": `usage: somnus packs list | get <id> [field] | set <id> <field> <value> | index
  look up or edit any pack's manifest fields by id — index regenerates the derived project.json files
  e.g. somnus packs list
       somnus packs set re-console-plus release_type beta`,

	"freeze": `usage: somnus freeze <pack-subdir> [mod-slugs...]
  pins mods in one subdir (e.g. modpacks/x/26.1.2-mr) so 'somnus update' skips them
  no slugs: lists that subdir's frozen mods
  recorded in the pack manifest under automation.freeze — doctor flags drift
  e.g. somnus freeze modpacks/re-console-plus/26.1.2-mr sodium iris`,

	"unfreeze": `usage: somnus unfreeze <pack-subdir> <mod-slugs...>
  unpins previously frozen mods so updates can apply to them again`,

	"export": `usage: somnus export [pack]
  builds changed packs locally (or one named pack) — artifacts land in artifacts/
  e.g. somnus export re-console-plus`,

	"build": `usage: somnus build <short-sha> | somnus build --pack <name> <short-sha>
  ci entry — builds git-changed packs (or one named pack) and tags artifacts with the sha
  e.g. somnus build a1b2c3d`,

	"sync": `usage: somnus sync [--dry-run]
  copies performance base content into consumer packs per the manifest mappings
  --dry-run  shows what would be copied, pruned, and refreshed without touching anything`,

	"update": `usage: somnus update [pack-dir] [--all]
  runs packwiz update --all in every pack subdir (honors automation.auto_update)
  with a pack-dir: just that pack (explicit beats the opt-out)
  run from inside a pack: scopes to it automatically — --all forces everything`,

	"refresh": `usage: somnus refresh [pack-dir] [--all]
  runs packwiz refresh in every pack subdir — same scoping rules as update`,

	"loader-update": `usage: somnus loader-update [latest|recommended] [pack-dir] [--all]
  migrates loaders across packs (honors automation.auto_update) — same scoping rules as update`,

	"modlist": `usage: somnus modlist <pack-subdir>
  writes the crash-assistant modlist.json from the subdir's .pw.toml files
  e.g. somnus modlist modpacks/re-console-plus/26.1.2-mr`,

	"pages": `usage: somnus pages [pack]   (alias: docs)
  writes modlist.md in every mod subdir — full runs (no pack arg) also emit projects.json for the docs site`,

	"test": `usage: somnus test <pack-subdir>   (alias: instance)
  spins up packwiz serve and installs into a local test instance
  requires $PACKWIZ_INSTALLER_JAR — honors $SOMNUS_TEST_INSTANCE
  e.g. somnus test modpacks/re-console-plus/26.1.2-mr`,

	"lint": `usage: somnus lint [files...]
  checks json and pw.toml files for syntax errors — no args: lints git-changed files`,

	"port": `usage: somnus port <mr-subdir> <cf-subdir> [--add] [--no-refresh] [--json]
  shows which mr mods are missing on the cf side — --add ports them interactively
  --json        emits {mr_total, cf_matched, missing[]} instead of plain text (dry-run only)
  --no-refresh  skips per-mod index rebuilds during --add and refreshes once at the end
  e.g. somnus port modpacks/rc-plus/26.1.2-mr modpacks/rc-plus/26.1.2-cf`,

	"import": `usage: somnus import <url-or-mrpack-file> [--id <pack-id>]
  pulls a modrinth mrpack into modpacks/ as a ready pack ({mc}-mr subdir, manifest, changelog)
  update metadata is reconstructed from cdn.modrinth.com urls so imported mods stay updatable
  e.g. somnus import https://cdn.modrinth.com/data/1KVo5zza/versions/g5RAIwpP/Fabulously.Optimized-v13.2.2.mrpack`,

	"side": `usage: somnus side <pack-dir> <mod-slug> [client|server|both]
  no side: shows the mod's current side per subdir
  with a side: rewrites it in every subdir's .pw.toml and refreshes (good for mislabeled mods)
  e.g. somnus side modpacks/re-console-plus some-mislabeled-mod client`,

	"publish": `usage: somnus publish <list|build|upload|verify> <manifest...> [variant] [--live]
  the release pipeline — go port of the rust publisher with the same outputs and artifact names
  list <manifests...>          prints matrix entries as json (multiple manifests concatenate)
  build <manifest> [variant]   exports artifacts into <pack>/artifacts/ and writes GITHUB_OUTPUT metadata
  upload <manifest> [variant]  sends artifacts to modrinth/curseforge — dry run unless --live
  verify <manifest> [variant]  asserts the version actually landed on modrinth (post-upload ci check)`,

	"packwiz": `usage: somnus packwiz build [--output <path>]
  clones packwiz at the pinned sha, applies all patches in patches/, and builds the binary
  default output: ./packwiz-bin/packwiz (./packwiz-bin/packwiz.exe on windows)
  set PACKWIZ_BIN to the output path so somnus commands pick up the patched binary
  e.g. somnus packwiz build --output ./bin/packwiz`,

	"doctor": `usage: somnus doctor   (alias: check)
  checks that tools (git, packwiz, java, zip), repo root, manifests, and legacy opt-out files are all healthy`,

	"validate": `usage: somnus validate <path/to/manifest.json> [more...] | somnus validate --all
  validates pack manifests — required fields, type, platform subdirs, changelog, role, automation
  --all  discovers and validates every manifest under modpacks/, datapacks/, resourcepacks/
  e.g. somnus validate modpacks/re-console-plus/manifest.json`,

	"automation": `usage: somnus automation get <pack-dir>
  spits out the effective automation settings for a pack as json
  merges manifest.json automation field with legacy opt-out.json if one is present
  e.g. somnus automation get modpacks/re-console-plus`,

	"add": `usage: somnus add <slug> [subdir-or-pack-dir] [--no-refresh]
  adds a mod by modrinth/curseforge slug to one subdir, one pack, or all packs
  routing is by subdir suffix: -mr → modrinth, -cf → curseforge
  if a slug only exists on one platform those dirs will fail gracefully (expected)
  e.g. somnus add sodium
       somnus add sodium modpacks/re-console-plus
       somnus add sodium modpacks/re-console-plus/26.1.2-mr`,

	"status": `usage: somnus status [--json]
  dashboard of all packs — version, mc version, loader, mod counts, frozen mods, auto-update flag
  --json  emits the same data as a json array instead`,

	"diff": `usage: somnus diff <old-ref> <new-ref> [path-prefix]
  shows added, removed, and updated mods (.pw.toml files) between two git refs
  path-prefix filters to a specific pack or subdir
  e.g. somnus diff HEAD~1 HEAD
       somnus diff v26.05.0 v26.06.0 modpacks/re-console-plus`,

	"completion": `usage: somnus completion bash|fish|zsh
  prints a shell completion script — eval it in your rc to get tab completion
  bash:  eval "$(somnus completion bash)"   (add to ~/.bashrc)
  fish:  somnus completion fish > ~/.config/fish/completions/somnus.fish
  zsh:   eval "$(somnus completion zsh)"   (add to ~/.zshrc)`,

	"help": `usage: somnus help [verb]
  shows the full verb list, or detailed usage for one verb`,

	"version": `usage: somnus version
  prints the somnus version`,
}
