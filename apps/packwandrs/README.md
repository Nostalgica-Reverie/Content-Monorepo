# Packwand Rust rewrite

This directory is the isolated replacement workspace for Packwand. The legacy
Go/Gleam application remains unchanged while the Rust engine reaches parity.

The rewrite currently contains seven focused crates:

- `packwand-pack` owns Packwand/Packwiz TOML models and Go-compatible hashing.
- `packwand-providers` resolves Modrinth, CurseForge, Forgejo, GitHub, and GitLab
  projects into a shared model.
- `packwand-ops` applies transactional add, remove, update, and metadata refresh operations.
- `packwand-build` plans and writes Modrinth/CurseForge archives, creates
  standalone content ZIPs, detects `packeater.json` folders for aggressive
  optimization, and safely imports both launcher archive formats.
- `packwand-workspace` owns typed manifests, project discovery, scaffolding,
  lifecycle edits, version bumps, and freeze state.
- `packwand-diagnostics` provides syntax/content linting, content registries,
  manifest/preflight validation, and Modrinth/CurseForge parity reports.
- `packwand-cli` produces the standalone `packwand` 26.2.0 binary. Its Clap
  command tree covers the complete legacy top-level and nested command catalog,
  with structural tests preventing commands from disappearing during the port.

The conservative command-by-command completion ledger is in
[`docs/phase3-parity.md`](docs/phase3-parity.md). A feature is not marked
complete merely because its UI or planning model exists.

Phase 3 also includes a Tauri 2 desktop host in `src-tauri/` and a Vue 3.5
frontend in `frontend/`. The frontend uses Tauri IPC exclusively; there is no
loopback application backend or Go process in its runtime path.

Archive builds are transactional, include indexed overrides, and are shared by
the CLI and desktop job engine. Imports validate paths and expanded sizes,
resolve launcher metadata, stage a complete pack, and only then move it into
the workspace. Native CI builds also cover changed-project selection, content
ZIPs, Packeater-marked variants, and Gradle mod variants. Marked builds resolve
the optimizer beside the Packwand executable, from the in-tree Packeater build,
from `PACKEATER_BIN`, or from `PATH`; they fail instead of silently falling back
to a larger plain ZIP when Packeater is missing.

Run the workspace checks from this directory:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p packwand-cli
```

## Install and run

The Windows installer is produced at
`target/release/bundle/nsis/Packwand_26.2.0_x64-setup.exe`. Run it normally;
Packwand installs per-user and includes the packwiz installer used by the
Diagnostics page plus the native `packwand.exe` CLI in its install directory.
Java must be available on `PATH` for installer-driven pack
tests and Gradle is required only when building mod projects.

To use the portable binaries instead, keep `packwand.exe` and
`packwand-gui.exe` beside each other. Run `packwand gui` or start
`packwand-gui.exe` directly.

## Build from source

Prerequisites: Rust stable with Cargo, Bun 1.3 or newer, the Tauri Windows
prerequisites (WebView2 and MSVC Build Tools), and Java for installer tests.

Run the desktop app from this directory:

```sh
bun install
cargo tauri dev
```

Create optimized CLI, desktop, and NSIS installer artifacts:

```sh
bun install --frozen-lockfile
cargo build --release -p packwand-cli
cargo tauri build
```

The CLI is `target/release/packwand.exe`, the portable desktop executable is
`target/release/packwand-gui.exe`, and the installer is under
`target/release/bundle/nsis/`.

Frontend-only checks are available through `bun run check`,
`bun run test:frontend`, and `bun run audit:capabilities`.
