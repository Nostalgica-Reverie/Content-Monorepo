# Phase 3 Packwand 26.2.0 parity contract

Phase 3 is complete only when every row below is implemented by Rust code and
available from the desktop application. A page, button, plan, or command stub
does not count as implementation. The application must not execute the Go
Packwand binary or use a loopback HTTP server.

Status meanings: **Complete** is production behavior wired to the GUI and
tested; **Engine** is reusable Rust behavior without its complete GUI flow;
**Partial** covers only some legacy behavior; **Missing** has no production
implementation. This ledger is deliberately conservative.

| Group | Command | Status | Current coverage / remaining work |
|---|---|---:|---|
| Pack management | `add` | Partial | Metadata transactions, provider resolution, and workspace-wide MR/CF fanout exist; interactive search and dependency fanout remain. |
| Pack management | `curseforge` | Partial | Numeric IDs, exact slug/URL search, exact file IDs, archive import/merge, fingerprint detection, and project-page opening are native; interactive search and dependency flow remain. |
| Pack management | `forgejo` | Engine | Release resolution exists; complete UI workflow remains. |
| Pack management | `freeze` / `unfreeze` | Engine | Workspace manifest and metadata pin transaction exist; GUI remains. |
| Pack management | `github` | Engine | Release resolution exists; complete UI workflow remains. |
| Pack management | `gitlab` | Engine | Release resolution exists; complete UI workflow remains. |
| Pack management | `import` | Engine | Safe transactional Modrinth and CurseForge archive import, metadata reconstruction, override extraction, exact CF project/file resolution, and workspace scaffolding are implemented; desktop workflow remains. |
| Pack management | `init` | Engine | Local pack initialization exists; loader discovery/selection remains. |
| Pack management | `modrinth` | Partial | Project/version resolution, explicit version IDs, and filename selection exist; interactive search and dependencies remain. |
| Pack management | `new` | Engine | Project scaffolding and variant creation exist; GUI remains. |
| Pack management | `pin` / `unpin` | Engine | Transactional metadata pin edits exist; batch GUI remains. |
| Pack management | `port` | Partial | MR/CF comparison, JSON reporting, and exact-slug CF add flow exist; dependency-aware guided matching remains. |
| Pack management | `rehash` | Engine | Transactional SHA-1/SHA-256/SHA-512 migration covers index entries and external downloads; GUI workflow and cache reuse remain. |
| Pack management | `remove` | Engine | Transaction-safe metadata removal exists; richer selection UI remains. |
| Pack management | `side` | Engine | Side reads and transactional edits exist; all-subdir drift/fix remains. |
| Pack management | `url` | Engine | Direct URL download, hashing, and metadata creation are available in the Rust CLI; GUI workflow remains. |
| Updates | `migrate` | Engine | Pack-format and explicit Minecraft/loader migrations are implemented; latest/recommended loader discovery remains. |
| Updates | `refresh` | Engine | Metadata/index/pack hashing exists and is GUI-wired. |
| Updates | `update` | Engine | Resolve-one/all, pinned-file skipping, dry-run, JSON/report output, and transactional provider updates exist; desktop workflow remains. |
| Build/export | `build` | Partial | Git-changed target selection, modpack exports, Gradle variant JARs, plain content ZIPs, and `packeater.json` variant discovery with hard-fail aggressive Packeater optimization are native; build-sync, cancellation, and the complete desktop workflow remain. |
| Build/export | `bump` | Engine | Manifest bump exists; in-pack config edits remain. |
| Build/export | `export` | Partial | Real transactional Modrinth and CurseForge archives are built by the CLI and desktop job, with manifest/override tests; side-selection and full golden-corpus parity remain. |
| Build/export | `json` | Engine | Native JSON/mcmeta minification, check, and strict modes are implemented in the Rust CLI; GUI workflow remains. |
| Build/export | `publish` | Complete | Manifest/variant matrices, native mod/pack/content builds, artifact validation, Modrinth and CurseForge uploads, idempotency, bounded retry, CF loader fallback, verification polling, CLI commands, and the desktop release workbench are implemented. Live upload remains operator-confirmed and is intentionally not exercised by tests. |
| Workspace | `packs` | Engine | Typed discovery/get/set/index exists; GUI field editor remains. |
| Workspace | `workspace` | Complete | Native export, provider add, update/check/report, refresh, loader update, migration, and guarded performance-base synchronization fanout are implemented; sync preview/apply is desktop-wired. |
| Diagnostics | `ci-local` | Engine | Native preflight and deterministic registry rebuild stages with JSON output exist; desktop presentation remains. |
| Diagnostics | `content-lint` | Engine | JSON/mcmeta, namespace, asset/model/function reference, duplicate, and case-collision lint exists; more legacy document rules and GUI remain. |
| Diagnostics | `doctor` | Engine | Native tool, repository-root, and project/pack health checks are implemented; deeper legacy checks and GUI presentation remain. |
| Diagnostics | `lint` | Partial | JSON and `.pw.toml` model parsing is GUI-wired; changed-file mode remains. |
| Diagnostics | `list` | Engine | Sorted mod inventory exists; filters and JSON UI export remain. |
| Diagnostics | `parity` | Engine | MR/CF drift reporting and GUI are implemented; golden corpus parity remains. |
| Diagnostics | `preflight` | Engine | Composite manifest, syntax, indexed-path, content-reference, and registry gate exists with structured output; GUI remains. |
| Diagnostics | `registry` | Engine | Deterministic SHA-256 registries cover standalone/bundled datapacks, resource packs, config ownership, KubeJS scripts, and mods; completions and GUI remain. |
| Diagnostics | `test` | Complete | A shared Rust engine starts an ephemeral restricted server, drives the bundled packwiz-installer with Java, validates exit status, and is exposed by both CLI and cancellable desktop job UI. |
| Diagnostics | `validate` | Partial | Manifest structural validation exists; schema, role and automation rules remain. |
| Other | `api` | Complete | The optional CLI host exposes authenticated read-only v1 health/projects/commands/diagnostics routes, while the desktop inspector invokes the same typed data natively over Tauri IPC without a loopback bridge. |
| Other | `automation` | Complete | Effective settings, opt-in checks, compatible provider update fanout, synchronization, validation, configured tests, CalVer planning/bump, JSON reports, CLI dry-run, and cancellable desktop dry-run/apply jobs are implemented. |
| Other | `cache` | Engine | Go-compatible cache index reading, reference scanning, validated prune, dry-run, and JSON reporting exist; inspect/verify and GUI remain. |
| Other | `diff` | Engine | Native git-ref metadata diff with grouped additions/removals/updates and JSON exists; GUI remains. |
| Other | `gui` | Complete | Native bundled Tauri/Vue host; no Go or HTTP bridge. |
| Other | `modlist` | Engine | Crash-assistant JSON writer is implemented in the Rust CLI; GUI workflow remains. |
| Other | `nix` | Engine | Packwiz2nix-compatible URL/SHA-256 checksum generation exists for one or all pack subdirs; GUI remains. |
| Other | `pages` | Engine | Transactional side-grouped mod lists plus docs and category project indexes use the established schema; GUI remains. |
| Other | `run` | Engine | User-defined pack scripts execute natively from the Rust CLI; GUI workflow remains. |
| Other | `serve` | Engine | Native localhost HTTP serving, safe paths, indexed-file restriction, basic mode, and refresh-on-pack request exist; cancellation/UI controls remain. |
| Other | `settings` | Engine | Transactional acceptable-loader/version set/add/remove and desktop settings exist; a pack settings editor remains. |
| Other | `utils` | Engine | Native command catalog and Markdown reference generation are implemented; packwiz binary management remains. |
| Shell | `completion`, `help`, `version` | Complete | Rust 26.2.0 CLI, help tree, PowerShell/Bash/Fish/Zsh/Elvish completion, and structural surface tests exist. |

## Global acceptance gates

1. Every row is **Complete**, not merely present in a menu.
2. Every Go command flag and exit behavior has a Rust characterization test.
3. Golden output parity covers real repository packs and export archives.
4. All long work is cancellable and reports typed progress/log events.
5. Production contains no Go Packwand process, loopback bridge, or remote
   Tauri capability grant.
