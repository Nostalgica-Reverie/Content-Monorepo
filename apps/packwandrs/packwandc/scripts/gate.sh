#!/usr/bin/env sh
# The packwandc quality gate.
#
# Runs every static check in one place, cheapest first, so a formatting slip
# fails in a second rather than after a full sanitizer build. CI calls this via
# `just lint-packwandc`; run it locally before pushing.
#
# What this does NOT do: build, run tests, or run sanitizers. Those are
# `just test-packwandc`, because they need a configured build tree and this
# script deliberately does not require one.

set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
root_dir=$(CDPATH='' cd -- "${script_dir}/.." && pwd)
cd "${root_dir}"

failures=0
step() {
    name=$1
    shift
    printf '\n=== %s\n' "${name}"
    if "$@"; then
        return 0
    fi
    printf '!!! %s FAILED\n' "${name}" >&2
    failures=$((failures + 1))
    return 0
}

have() { command -v "$1" >/dev/null 2>&1; }

sources() {
    find kernel modules arch tests -type f \( -name '*.c' -o -name '*.h' -o -name '*.m' \) \
        2>/dev/null | sort
}
headers() { find include -type f -name '*.h' 2>/dev/null | sort; }

# --- 1. formatting ----------------------------------------------------------

check_format() {
    if ! have clang-format; then
        printf 'clang-format not found -- skipping (CI installs it)\n' >&2
        return 0
    fi
    # shellcheck disable=SC2046 # deliberate word split over the file list
    clang-format --dry-run --Werror $(sources) $(headers)
}

# --- 2. generated artifacts are current -------------------------------------

check_generated() { sh scripts/gen-syscalls.sh --check; }

# --- 3. the boundary and banned-construct gates -----------------------------

check_uapi() { sh scripts/gate-uapi.sh; }
check_banned() { sh scripts/gate-banned.sh; }

# --- 4. shell ---------------------------------------------------------------

check_shell() {
    if ! have shellcheck; then
        printf 'shellcheck not found -- skipping (CI installs it)\n' >&2
        return 0
    fi
    shellcheck scripts/*.sh
}

# --- 5. clang-tidy ----------------------------------------------------------
#
# Needs compile_commands.json, which CMake writes into the build tree. If no
# tree has been configured this is skipped locally, but NEVER in CI -- the
# workflow configures first, so a skip there would silently drop the check.

check_tidy() {
    if ! have clang-tidy; then
        printf 'clang-tidy not found -- skipping (CI installs it)\n' >&2
        return 0
    fi

    db=""
    for candidate in build/*/compile_commands.json; do
        [ -f "${candidate}" ] && db=$(dirname "${candidate}") && break
    done

    if [ -z "${db}" ]; then
        if [ "${PWC_GATE_STRICT:-0}" = "1" ]; then
            printf 'no compile_commands.json found and PWC_GATE_STRICT=1\n' >&2
            printf 'configure a build tree first: cmake --preset dev\n' >&2
            return 1
        fi
        printf 'no configured build tree -- skipping clang-tidy\n' >&2
        printf 'run `cmake --preset dev` first to enable it\n' >&2
        return 0
    fi

    # --header-filter is mandatory: clang-tidy silently skips included headers
    # without it, and most of packwandc's contracts live in headers.
    # Keep the gate output concise.
    # shellcheck disable=SC2046 # deliberate word split over the file list
    clang-tidy -p "${db}" --header-filter='.*/packwandc/include/packwandc/.*' \
        --warnings-as-errors='*' $(find kernel -name '*.c' | sort)
}

step "clang-format" check_format
step "generated artifacts up to date" check_generated
step "uapi boundary" check_uapi
step "banned constructs" check_banned
step "shellcheck" check_shell
step "clang-tidy" check_tidy

printf '\n'
if [ "${failures}" -gt 0 ]; then
    printf 'gate: %d step(s) failed\n' "${failures}" >&2
    exit 1
fi
printf 'gate: all checks passed\n'
