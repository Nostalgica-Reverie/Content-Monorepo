# Packwand Architecture

Packwand (`apps/packwand`, Go module `git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand`) is a hard fork of packwiz built for mass-concurrent, monorepo-scale handling of Minecraft modpacks. This document describes the package layout, the dependency shape, and the conventions that keep it maintainable. It was promoted from the §4 architecture survey in the repo-root `packwandaudit.md`.

## Package map

| Package | Role | Depended on by |
|---|---|---|
| `core/` | Domain model (`Mod`/`Pack`/`Index`), hashing, downloading, version resolution, concurrency primitives, shared HTTP clients | ~48 files — the shared kernel |
| `cmd/` | Cobra CLI surface, incl. nested `cmd/packwiz-bootstrap` and `cmd/serve-templates` sub-`main`s | ~24 files (via registration inversion) |
| `curseforge/` | CurseForge platform integration + `murmur2/`, `packinterop/` subpackages | — |
| `modrinth/` | Modrinth platform integration | — |
| `forgejo/`, `github/`, `gitlab/` | Git-forge-based mod source/update backends | — |
| `manifest/` | Multi-pack auto-publish manifest loading | ~16 files |
| `content/` | Config lint, preflight, local CI, `.mrpack`/CF zip import | — |
| `registry/` | Schema/type checks (KubeJS, ProbeJS, datapacks, resourcepacks) | ~6 files (mainly `content/`) |
| `workspace/` | Batch scheduler across packs, subprocess fan-out | ~16 files |
| `migrate/` | Config/loader/MC-version migration | — |
| `api/` | Versioned HTTP API backing the GUI | only `gui/gui.go` |
| `build/` | Build/plan/export/publish engine | — |
| `nix/` | Nix packaging helpers | — |
| `cmdshared/`, `clistyle/`, `settings/`, `url/`, `utils/` | Small supporting leaf packages (`clistyle/` is the shared terminal-theming leaf) | — |

## Dependency shape

`core/` is a near-leaf: its only internal import is `curseforge/murmur2` (for the CF hash format), yet nearly every other package depends on it — the expected shape for a shared kernel.

**Registration inversion** is the one layering choice that looks backwards at first read: `curseforge`, `modrinth`, `forgejo`, `github`, `gitlab`, `migrate`, `settings`, `url`, `utils`, `build`, `content`, and `gui` all import `cmd`, never the reverse. `main.go` blank-imports each feature package; each package's `init()` calls `cmd.Add()`/`cmd.AddToGroup()` to register its Cobra commands. This is deliberate — it lets every platform package self-register without `cmd/root.go` knowing any of them exist, and there is no import cycle to untangle. Don't "fix" this into a conventional dependency-injection scheme.

`api/` sits at the top, imported only by `gui/gui.go` — changes there are low-blast-radius within this module. (The standalone `apps/api`/cursorapi host shares logic but is a separate module.)

## Concurrency model

All concurrency limits and `ParallelFor[T]` (the semaphore-bounded generic used instead of `errgroup`, which appears nowhere in this codebase) live in `core/concurrency.go` — the single place any future concurrency-model change starts.

| Function | Env var | Default |
|---|---|---|
| `core.MaxConcurrent()` | `PACKWAND_CONCURRENCY` (legacy `SOMNUS_CONCURRENCY`) | `min(runtime.NumCPU(), 8)` |
| `core.NetworkConcurrent()` | `PACKWAND_NETWORK_CONCURRENCY` | falls back to `MaxConcurrent()` |
| `core.HashConcurrent()` | `PACKWAND_HASH_CONCURRENCY` | falls back to `MaxConcurrent()` |
| `workspace.CacheSlotCount()` | `PACKWAND_CACHE_SLOTS` | FNV-buckets packs into N `cache-slot-*` resources |

**`workspace.Scheduler`** (`workspace/scheduler.go`) is the primitive multi-item operations should use: a resource-keyed scheduler (mutex + `sync.Cond`) running exactly `workers` long-lived goroutines. `Submit(Task)` takes a list of `Resource` keys (e.g. `"subdir:"+dir`, a cache-slot bucket); a task runs only once it holds the head of every resource queue it needs, giving per-resource mutual exclusion without a global lock. `validate --all`, `workspace refresh/update`, and the publish planner all fan out through it (or through `core.ParallelFor` where no resource coordination is needed).

**Subprocess fan-out guard**: `workspace.ConfigureSubprocess` (`workspace/workspace.go`) forces each child `packwand` process's internal concurrency to 1 (`PACKWAND_*_CONCURRENCY=1`) unless explicitly overridden. Without this, a workspace-level `workers=8` fan-out times each child's own `workers=8` would spawn up to 64 concurrent operations. Do not remove it.

## HTTP conventions

Every HTTP client comes from `core/httpclient.go` — never construct a bare `&http.Client{}`:

- `core.NewClient()` — metadata/API calls: 30s timeout, transparent retry (3 attempts, doubling backoff) on network errors/429/5xx.
- `core.NewDownloadClient()` — large file transfers: same retry policy, 10-minute timeout.
- `core.NewUploadClient()` — transfer-scale timeout, no transparent retry, for callers with their own retry/backoff loop (the publish upload path).

A client without a `Timeout` can block a `NetworkConcurrent()` worker slot forever on one hung connection. The retry transport resolves `http.DefaultTransport` per call so `httpmock`-based tests keep working.

## Error handling conventions

- **`os.Exit` is confined to `cmd/` and `cmdshared/`.** Provider and library packages return errors; Cobra command handlers use `RunE` (root has `SilenceUsage` so runtime errors print once, without a usage dump). `just lint-go` enforces this with a grep gate.
- Download integrity is verify-then-commit: `core/download.go` streams into a temp file while hashing and only finalizes into the cache (`CreateFromTemp`) after the hash check passes. Unverified bytes never reach a path anything else reads. Use the same pattern for any new verify-then-commit logic.
- Paths derived from untrusted archive contents must pass `content.safeJoin` (or an equivalent prefix check) before writing — see `extractOverridesPrefix` and `writeImportedToml` in `content/content.go`.
- The `length-bytes` hash format is internal-only (`core.IsInternalHashFormat`) and is rejected on both mod save and load.

## Testing conventions

- `manifest/load_all_test.go` is the template for unit tests: fast, `t.TempDir()`-based, no subprocess, no network.
- Provider API clients are tested with a stub `http.RoundTripper` (see `curseforge/request_test.go`, `github/request_test.go`, `gitlab/request_test.go`).
- `characterization/refresh_test.go` compiles the real binary and drives it against fixture packs — the regression gate for behavior-affecting refactors.
- `curseforge/murmur2` carries golden CurseForge fingerprint vectors; any change to the murmur2 implementation must keep them passing (a silent hash mismatch would break CF fingerprint matching, which is worse than most bugs).

## CLI conventions

- Persistent root flags (`-y`/`--yes`) are the pattern for cross-cutting switches.
- Machine-readable output is `--json` on the command (`doctor`, `list`, `parity`, `automate run`, `workspace status/update`, `update`, docs commands, …); reports that CI consumes also support `--report <path>`.
- Group commands (parents with subcommands and no `Run`) reject unknown subcommands with a non-zero exit via `enforceGroupArgs`.
