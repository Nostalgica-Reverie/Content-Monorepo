#!/usr/bin/env sh
# Header-boundary gate -- packwandc.md 7.6.
#
# packwandc has no privilege ring: it is a userspace static library, so the
# kernel/userland split cannot be enforced by hardware. It is enforced here
# instead, by partitioning the headers and refusing the includes that would
# blur the line. This is the rule that stops "the kernel" quietly becoming a
# pile of mutually-including C files.
#
# Rules:
#   1. include/packwandc/uapi/** may not include include/packwandc/kernel/**
#   2. crates/packwandc-sys may only reference headers under uapi/
#   3. modules/<a>/** may not include modules/<b>/** private headers
#   4. arch/** is reachable only through its module's arch interface header

set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
root_dir=$(CDPATH='' cd -- "${script_dir}/.." && pwd)
cd "${root_dir}"

failures=0

fail() {
    printf '\ngate-uapi: %s\n' "$1" >&2
    printf '%s\n' "$2" | while IFS= read -r line; do
        [ -n "${line}" ] && printf '    %s\n' "${line}" >&2
    done
    failures=$((failures + 1))
}

# --- rule 1: uapi must not reach into kernel --------------------------------

if [ -d include/packwandc/uapi ]; then
    hits=$(grep -rnE '^[ \t]*#[ \t]*include[ \t]*[<"]packwandc/kernel/' include/packwandc/uapi ||
        true)
    if [ -n "${hits}" ]; then
        fail 'a uapi header includes a kernel-internal header' "${hits}"
    fi
fi

# --- rule 2: the Rust FFI crate sees only uapi ------------------------------
#
# packwandc-sys is the single unsafe crate (packwandc.md 6.1). If it can see
# kernel internals, the safe SDK above it is no longer bounded by the public
# ABI and the whole layering argument collapses.

if [ -d crates/packwandc-sys ]; then
    hits=$(grep -rnE 'packwandc/kernel/' crates/packwandc-sys || true)
    if [ -n "${hits}" ]; then
        fail 'packwandc-sys references a kernel-internal header' "${hits}"
    fi
fi

# --- rule 3: modules are opaque to each other -------------------------------
#
# A module's public surface is its uapi header. Anything else under its
# directory is private. Cross-module traffic goes through the kernel.

if [ -d modules ]; then
    for dir in modules/*/; do
        [ -d "${dir}" ] || continue
        this=$(basename "${dir}")
        hits=$(grep -rnE '^[ \t]*#[ \t]*include[ \t]*"\.\./' "${dir}" || true)
        if [ -n "${hits}" ]; then
            fail "module '${this}' includes a header from outside its own directory" "${hits}"
        fi
    done
fi

# --- rule 4: arch is private to its module ----------------------------------
#
# arch/<platform>/<module>/ is reachable only from modules/<module>/. Nothing
# in the kernel or in another module may include it directly.

if [ -d arch ]; then
    hits=$(grep -rnE '^[ \t]*#[ \t]*include[ \t]*[<"].*\barch/' kernel modules 2>/dev/null |
        grep -vE '^modules/([a-z0-9_]+)/.*arch/[a-z0-9_]+/\1/' || true)
    if [ -n "${hits}" ]; then
        fail 'arch/ header included from outside its owning module' "${hits}"
    fi
fi

# --- rule 5: uapi headers are self-contained --------------------------------
#
# Every public header must compile on its own. Callers include one header and
# get a working declaration set; they should not have to discover an ordering.
# Cheap to check and it catches the "works because something else included it
# first" class of breakage before a consumer does.

if [ -d include/packwandc/uapi ] && command -v clang >/dev/null 2>&1; then
    for hdr in include/packwandc/uapi/*.h; do
        [ -f "${hdr}" ] || continue
        if ! clang -fsyntax-only -std=c2x -I include -x c "${hdr}" 2>/dev/null; then
            # Re-run without suppression so the diagnostic reaches the log.
            out=$(clang -fsyntax-only -std=c2x -I include -x c "${hdr}" 2>&1 || true)
            fail "uapi header is not self-contained: ${hdr}" "${out}"
        fi
    done
fi

if [ "${failures}" -gt 0 ]; then
    printf '\ngate-uapi: %d rule(s) violated\n' "${failures}" >&2
    exit 1
fi

printf 'gate-uapi: ok\n'
