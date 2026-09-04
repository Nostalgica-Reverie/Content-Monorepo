# Monorepo build tasks. Install just from https://just.systems (or
# `cargo install just`), then run e.g. `just lint-mods`. `just --list`
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

# Run every mod's Gradle verification lifecycle
lint-mods: (_mod_gradlew_all "check")

# Repo-wide spell check (requires typos -- cargo install typos-cli); config in _typos.toml
lint-typos:
    typos .

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
# NOTE: broken since packages/packwand2nix (the flake's Nix-generation helper
# library) was removed along with the other non-mc projects -- flake.nix
# cannot currently evaluate. Left in place pending a decision on whether the
# Nix packaging story is rebuilt or retired.
lint-nix:
    nix flake check --no-build --no-update-lock-file

# All linters/vetters and vulnerability scans
lint: lint-mods lint-fmt lint-typos lint-actions docs-typecheck

# — Test —

# Run Packwand's manifest/content/registry gate for a pack or subdir.
# Requires a `packwand` (or `bundle`) binary on PATH -- see header comment.
preflight DIR:
    packwand preflight "{{ DIR }}"

ci-local DIR:
    packwand ci-local "{{ DIR }}"

# Run unit/integration tests for every Stonecutter mod project
test-mods: (_mod_gradlew_all "test")

# Build all flake checks, including the generated modpack inventory
# NOTE: broken, see lint-nix.
test-nix:
    nix flake check --no-update-lock-file --print-build-logs

# All tests
test: test-mods

# — Build —

# Build distributable jars for every Stonecutter mod project
build-mods: (_mod_gradlew_all "build -x test")

# Generate/update packwiz2nix checksums.json for every modpack subdir via packwand's internal generator
# NOTE: broken, see lint-nix -- packwand2nix is gone.
gen-nix:
    packwand nix gen --all

# Build the Nix outputs without creating result symlinks
# NOTE: broken, see lint-nix.
build-nix:
    nix build --no-link --no-update-lock-file --print-build-logs .#default

# Build everything (mods)
build: build-mods

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
