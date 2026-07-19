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
# it's been added manually) -- not a bare PowerShell/cmd prompt. See
# c.md section 3.4.1 for why this was chosen over a PowerShell override.
set shell := ["sh", "-cu"]

INSTALLER_DIR := "apps/packwand-installer"
WEBVIEW_DIR := "apps/mod-browser-webview"
GUI_TAURI_DIR := "apps/packwand/gui/tauri"
BOT_DIR := "apps/bot"
API_DIR := "apps/api"
DOCS_SITES := "docs docs/packwand docs/packwiz"
HANDBOOK_DIR := "docs/modpack-dev-handbook"

# just has no built-in equivalent of Task's `{{exeExt}}`; define it explicitly.
EXE_EXT := if os() == "windows" { ".exe" } else { "" }

# C compiler for tools/hashutil. Override with `CC=clang just build-hashutil`
# if you don't have a `cc`/`gcc` on PATH under that name (this repo's own
# Windows dev environment has `gcc`/`clang` from MinGW but no `cc` alias).
CC := env("CC", "gcc")

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
[working-directory: 'apps/packwand']
lint-go:
    go vet ./...
    ! grep -rn --include='*.go' 'os\.Exit' core/ content/ registry/ workspace/ manifest/ build/ api/ migrate/ modrinth/ curseforge/ github/ gitlab/ forgejo/ url/ settings/ utils/ clistyle/ nix/ 2>/dev/null | grep -v _test.go || { echo 'os.Exit found outside cmd/ — return an error instead' >&2; exit 1; }

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
[working-directory: 'apps/packwand']
audit-go:
    go run golang.org/x/vuln/cmd/govulncheck@latest ./...

# fmt + clippy on the root Cargo workspace (packwand-rs launcher core and packwand-gui)
lint-rust-core:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

# Scan Rust lockfiles for RUSTSEC advisories (requires cargo-audit)
audit-rust:
    cargo audit --file {{ WEBVIEW_DIR }}/Cargo.lock
    cargo audit --file Cargo.lock

# ESLint + Prettier check on the Discord bot (requires bun)
[working-directory: 'apps/bot']
lint-bot:
    bun install --cwd ../.. --frozen-lockfile --filter @reverie/pineapple
    bun run lint

# Repo-wide spell check (requires typos -- cargo install typos-cli); config in _typos.toml
lint-typos:
    typos .

# Lint Forgejo workflows incl. shellcheck of run blocks (requires actionlint)
lint-actions:
    actionlint -config-file .forgejo/actionlint.yaml -ignore 'in invalid format because owner and repo' .forgejo/workflows/*.yml

# Evaluate every Nix flake output without building it (Nix requires Linux, macOS, or WSL)
lint-nix:
    nix flake check --no-build --no-update-lock-file

# clang-format check on tools/hashutil (requires clang-format)
[working-directory: 'tools/hashutil']
lint-hashutil:
    clang-format --dry-run --Werror *.c *.h

# All linters/vetters and vulnerability scans
lint: lint-go lint-cursorapi lint-webview lint-installer lint-mods lint-rust-core lint-bot lint-typos lint-actions lint-hashutil audit-go audit-rust docs-typecheck

# — Test —

# Go tests
[working-directory: 'apps/packwand']
test-go:
    go test ./...

# Run Packwand's manifest/content/registry gate for a pack or subdir
[working-directory: 'apps/packwand']
preflight DIR:
    go run . preflight "{{ DIR }}"

# Run the IDE's CI-equivalent Packwand stages for a pack subdir
[working-directory: 'apps/packwand']
ci-local DIR:
    go run . ci-local "{{ DIR }}"

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

# Cargo tests for the root Cargo workspace (excludes #[ignore]'d real-boot tests needing Java/network)
test-rust-core:
    cargo test --workspace

# Build + run tools/hashutil's self-test against known hash vectors
[working-directory: 'tools/hashutil']
test-hashutil: build-hashutil
    ./hashutil{{ EXE_EXT }} --selftest

# Build all flake checks, including Packwand and the generated modpack inventory
test-nix:
    nix flake check --no-update-lock-file --print-build-logs

# All tests
test: test-go test-cursorapi test-installer test-mods test-webview test-rust-core test-hashutil

# — Build —

# Build the packwand CLI
[working-directory: 'apps/packwand']
build-packwand:
    go build -o packwand{{ EXE_EXT }} .

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
[working-directory: 'apps/packwand']
build-bootstrap:
    go build -o packwiz-bootstrap{{ EXE_EXT }} ./cmd/packwiz-bootstrap

# Build the packwand CLI and stage it as a Tauri external binary (sidecar) named for the host Rust target triple
build-gui-sidecar:
    mkdir -p {{ GUI_TAURI_DIR }}/src-tauri/binaries
    go build -C apps/packwand -o gui/tauri/src-tauri/binaries/packwand-$(rustc -vV | awk '/^host:/ {print $2}'){{ EXE_EXT }} .

# Build the native Packwand GUI app (Tauri v2). Requires cargo tauri-cli; see docs/packwand/docs/development/gui-build.md
[working-directory: 'apps/packwand/gui/tauri']
build-gui: build-gui-sidecar
    cargo tauri build

# Generate/update packwiz2nix checksums.json for every modpack subdir via packwand's internal generator
gen-nix:
    go run -C apps/packwand . nix gen --all

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

# Build tools/hashutil (single .c file, no separate build system -- see c.md section 1.5)
# Targets C23; -std=c2x is the flag name gcc/clang accept for it (see hashutil.c's header comment).
[working-directory: 'tools/hashutil']
build-hashutil:
    {{ CC }} -O2 -Wall -Wextra -std=c2x -o hashutil{{ EXE_EXT }} hashutil.c

# Build everything (CLI, installer, webview, bootstrap, bot, hashutil)
build: build-packwand build-cursorapi build-installer build-mods build-webview build-bootstrap build-bot build-hashutil

# — Docs —

# Type-check all docs sites with their own compiler configuration
docs-typecheck:
    bun install --frozen-lockfile
    for d in {{ DOCS_SITES }}; do (cd "$d" && bun run typecheck) || exit 1; done
    cd {{ HANDBOOK_DIR }} && bun run check

# Build all three VitePress sites and the Svelte handbook
docs-build:
    bun install --frozen-lockfile
    for d in {{ DOCS_SITES }}; do (cd "$d" && bun run docs:build) || exit 1; done
    cd {{ HANDBOOK_DIR }} && bun run build

# Check cross-site links across all three docs sites against their built dist/ output (run after docs-build)
docs-lint-links:
    bun docs/link-lint.mts

# — Frontend —

# Rebuild the Gleam GUI frontend into gui/static
[working-directory: 'apps/packwand/gui/ui']
gui-frontend:
    node build.mts
