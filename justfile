# Monorepo build tasks. Install just from https://just.systems (or
# `cargo install just`), then run e.g. `just build-installer`. `just --list`
# shows every public recipe. CI (.forgejo/workflows/) drives these same
# recipes so local runs match the build server.
#
# Windows: recipe bodies run through a
# real external `sh`, matching just's default on every platform -- this
# repo does not override `windows-shell`. Git for Windows is already a
# hard requirement for `git` itself, and its usr/bin (sh, awk, ...) is
# exactly what's needed here; no extra dependency is introduced beyond
# what any POSIX-flavored repo tool already assumes. Run `just` from a
# shell that has Git's usr/bin on PATH (a Git Bash shell, or one where
# it's been added manually) -- not a bare PowerShell/cmd prompt.
#
set shell := ["sh", "-cu"]

INSTALLER_DIR := "apps/packwand-installer"
WEBVIEW_DIR := "apps/mod-browser-webview"
GUI_TAURI_DIR := "apps/packwandrs"
BOT_DIR := "apps/bot"
API_DIR := "apps/api"
DOCS_SITES := "docs docs/packwand docs/packwiz"

# just has no built-in equivalent of Task's `{{exeExt}}`; define it explicitly.
EXE_EXT := if os() == "windows" { ".exe" } else { "" }

default:
    @just --list

# — Shared helpers (the "defaults": component recipes call these instead of
#   re-encoding platform quirks) —

[unix]
_gradlew DIR ARGS:
    cd "{{ DIR }}" && ./gradlew {{ ARGS }} --no-daemon

[windows]
_gradlew DIR ARGS:
    cd "{{ DIR }}" && cmd.exe //d //c gradlew.bat {{ ARGS }} --no-daemon

_mod_gradlew_all ARGS:
    found=0; for d in mods/*; do [ -f "$d/gradlew" ] || continue; found=1; just _gradlew "$d" "{{ ARGS }}" || exit 1; done; [ "$found" -eq 1 ] || { echo "no Gradle mod projects found under mods/" >&2; exit 1; }

# — Lint —

# Vet the Go module; also guard that os.Exit stays confined to cmd/ and cmdshared/
# Vet the cursorapi Go module
[working-directory: 'apps/api']
lint-cursorapi:
    go vet ./...

# Clippy on mod-browser-webview
[working-directory: 'apps/mod-browser-webview']
lint-webview:
    cargo clippy --all-targets -- -D warnings

# Compile-check packwiz-installer (Gradle has no separate lint; compilation surfaces diagnostics)
lint-installer: (_gradlew INSTALLER_DIR "classes bootstrap:classes")

# Run every mod's Gradle verification lifecycle
lint-mods: (_mod_gradlew_all "check")

# Scan the Go module for known vulnerabilities
# fmt + clippy on the packwandrs workspace (CLI, launcher core, desktop shell)
[working-directory: 'apps/packwandrs']
lint-rust-core:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

# Scan Rust lockfiles for RUSTSEC advisories (requires cargo-audit)
audit-rust:
    cargo audit --file {{ WEBVIEW_DIR }}/Cargo.lock
    cargo audit --file apps/packwandrs/Cargo.lock
    cargo audit --file Cargo.lock

# ESLint + Prettier check on the Discord bot (requires bun)
[working-directory: 'apps/bot']
lint-bot:
    bun install --cwd ../.. --frozen-lockfile --filter @reverie/pineapple
    bun run lint

# Repo-wide spell check (requires typos -- cargo install typos-cli); config in _typos.toml
lint-typos:
    typos .

# Not in the `lint` aggregate yet: the repo has not been through `just fmt-js`
# once, so this reports ~460 files and would fail every run. Add it to `lint`
# in the same commit that lands the reformat.

# Prettier check; config in prettier.config.mjs, indentation from .editorconfig
lint-fmt:
    bun install --frozen-lockfile
    bunx prettier --check .

# Rewrite everything Prettier owns (lint reports, it does not edit)
fmt-js:
    bunx prettier --write .

# Turbo is wired underneath just, not beside it: CI calls the recipes, and the
# recipes call turbo. Telemetry is off by default -- opt in per-machine with
# `bunx turbo telemetry enable` if you want it.

# Fan a Turborepo task across the JS/TS workspaces, e.g. `just turbo typecheck`
turbo TASK *ARGS:
    TURBO_TELEMETRY_DISABLED=1 bunx turbo run {{ TASK }} {{ ARGS }}

# Run one of the scripts in scripts/ by name, e.g. `just scripts lint-changelogs --json`
scripts NAME *ARGS:
    bun scripts/run.ts {{ NAME }} {{ ARGS }}

# Not part of `just lint` yet -- the resourcepacks have no changelogs today.
# Check every changelog.md names its manifest's version and says something
lint-changelogs:
    bun scripts/run.ts lint-changelogs

# Build release notes for a project, folding in its performance base's changelog
release-notes MANIFEST *ARGS:
    bun scripts/run.ts build-release-notes "{{ MANIFEST }}" {{ ARGS }}

# Lint Forgejo workflows incl. shellcheck of run blocks (requires actionlint)
lint-actions:
    actionlint -config-file .forgejo/actionlint.yaml -ignore 'in invalid format because owner and repo' .forgejo/workflows/*.yml

# Evaluate every Nix flake output without building it (Nix requires Linux, macOS, or WSL)
lint-nix:
    nix flake check --no-build --no-update-lock-file

# fmt + clippy on packeater_cli (now a packwandrs workspace member; mirrors ci-packeater.yml)
[working-directory: 'apps/packwandrs']
lint-packeater:
    cargo fmt --package packeater_cli -- --check
    cargo clippy -p packeater_cli --all-targets -- -D warnings

# All linters/vetters and vulnerability scans
lint: lint-cursorapi lint-webview lint-installer lint-mods lint-rust-core lint-bot lint-typos lint-actions lint-packeater audit-rust docs-typecheck

# — Test —

# Run Packwand's manifest/content/registry gate for a pack or subdir
preflight DIR:
    cargo run --manifest-path apps/packwandrs/Cargo.toml -p packwand-cli -- preflight "{{ DIR }}"

# Run the IDE's CI-equivalent Packwand stages for a pack subdir
ci-local DIR:
    cargo run --manifest-path apps/packwandrs/Cargo.toml -p packwand-cli -- ci-local "{{ DIR }}"

# Time Packwand's hot stages (PACKWAND_TIMINGS spans) against a real mr and cf pack subdir
bench-packwand MR_DIR CF_DIR:
    cargo build --release --manifest-path apps/packwandrs/Cargo.toml -p packwand-cli
    cd "{{ MR_DIR }}" && "{{ justfile_directory() }}/apps/packwandrs/target/release/packwand{{ EXE_EXT }}" update --all --dry-run
    cd "{{ MR_DIR }}" && "{{ justfile_directory() }}/apps/packwandrs/target/release/packwand{{ EXE_EXT }}" refresh
    cd "{{ MR_DIR }}" && "{{ justfile_directory() }}/apps/packwandrs/target/release/packwand{{ EXE_EXT }}" modrinth export -o bench-export.mrpack && rm -f bench-export.mrpack
    cd "{{ CF_DIR }}" && "{{ justfile_directory() }}/apps/packwandrs/target/release/packwand{{ EXE_EXT }}" curseforge export -o bench-export.zip && rm -f bench-export.zip

# Test the cursorapi Go module
[working-directory: 'apps/api']
test-cursorapi:
    go test ./...

# Gradle tests (installer + bootstrap)
test-installer: (_gradlew INSTALLER_DIR "test")

# Run unit/integration tests for every Stonecutter mod project
test-mods: (_mod_gradlew_all "test")

# Cargo tests
[working-directory: 'apps/mod-browser-webview']
test-webview:
    cargo test

# Cargo tests for the packwandrs workspace (excludes #[ignore]'d real-boot tests needing Java/network)
[working-directory: 'apps/packwandrs']
test-rust-core:
    cargo test --workspace

# Build all flake checks, including Packwand and the generated modpack inventory
test-nix:
    nix flake check --no-update-lock-file --print-build-logs

# All tests
test: test-cursorapi test-installer test-mods test-webview test-rust-core

# — Build —

# Build the Rust Packwand CLI
[working-directory: 'apps/packwandrs']
build-packwand:
    cargo build --release -p packwand-cli

# Build the cursorapi HTTP server
[working-directory: 'apps/api']
build-cursorapi:
    go build -o cursorapi{{ EXE_EXT }} ./cursorapi

# Build packwiz-installer (and the legacy Java bootstrap) via Gradle
build-installer: (_gradlew INSTALLER_DIR "build -x test")

# Build distributable jars for every Stonecutter mod project
build-mods: (_mod_gradlew_all "build -x test")

# Build mod-browser-webview (release). Linux requires webkit2gtk; Windows requires the WebView2 runtime at run time.
[working-directory: 'apps/mod-browser-webview']
build-webview:
    cargo build --release

# Build the Go packwiz-bootstrap wrapper
# Build the native Rust/Vue Packwand GUI app (Tauri v2).
[working-directory: 'apps/packwandrs']
build-gui: build-packwand
    cargo tauri build

# Generate/update packwiz2nix checksums.json for every modpack subdir via packwand's internal generator
gen-nix:
    cargo run --manifest-path apps/packwandrs/Cargo.toml -p packwand-cli -- nix gen --all

# Build the Packwand CLI and Cursor API through Nix without creating result symlinks
build-nix:
    nix build --no-link --no-update-lock-file --print-build-logs .#packwand .#cursorapi

# Regenerate the webview third-party licenses page (embedded at build time)
[working-directory: 'apps/mod-browser-webview']
gen-licenses:
    cargo install --locked cargo-about
    cargo about generate about.hbs -o src/licenses.html

# Typecheck + bundle smoke-test the Discord bot (requires bun; Bun runs the TS sources directly in production)
[working-directory: 'apps/bot']
build-bot:
    bun install --cwd ../.. --frozen-lockfile --filter @reverie/pineapple
    bun run typecheck
    bun build src/index.ts --target=bun --outdir=dist

# Build everything (CLI, installer, webview, bootstrap, bot)
build: build-packwand build-cursorapi build-installer build-mods build-webview build-bot

# — Docs —

# Type-check all docs sites with their own compiler configuration
docs-typecheck:
    bun install --frozen-lockfile
    for d in {{ DOCS_SITES }}; do (cd "$d" && bun run typecheck) || exit 1; done

# Build all three VitePress sites (in parallel — they're independent) and the Svelte handbook
docs-build:
    #!/usr/bin/env sh
    set -eu
    bun install --frozen-lockfile
    pids=""
    for d in {{ DOCS_SITES }}; do
        echo "building $d in the background..."
        (cd "$d" && bun run docs:build) >"$d/.docs-build.log" 2>&1 &
        pids="$pids $!"
    done
    status=0
    for p in $pids; do wait "$p" || status=1; done
    for d in {{ DOCS_SITES }}; do
        echo "----- $d -----"
        cat "$d/.docs-build.log"
        rm -f "$d/.docs-build.log"
    done
    [ "$status" -eq 0 ]

# Check cross-site links across all three docs sites against their built dist/ output (run after docs-build)
docs-lint-links:
    bun docs/link-lint.mts

# — Frontend —

# Rebuild the Gleam GUI frontend into gui/static
[working-directory: 'apps/packwandrs']
gui-frontend:
    bun run build
