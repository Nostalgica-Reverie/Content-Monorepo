#!/usr/bin/env sh
# Banned-construct scanner -- packwandc.md 7.5.
#
# This is the C equivalent of Rust's #![forbid(unsafe_code)]: a mechanical
# refusal of the constructs that make C unsafe, rather than a convention
# nobody enforces. It scans kernel/, modules/ and arch/ only. tests/ is exempt
# (a test may legitimately construct a bad case) and so is scripts/.
#
# Every rule here traces to a line in packwandc.md 7.5. If a rule needs an
# exception, the exception goes in this file with a written reason -- the same
# standard .clang-tidy suppressions are held to.

set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
root_dir=$(CDPATH='' cd -- "${script_dir}/.." && pwd)
cd "${root_dir}"

failures=0

# Directories under the gate. Missing ones are fine: modules/ and arch/ do not
# exist until phase 2 (packwandc.md 10).
scan_dirs=""
for d in kernel modules arch; do
    [ -d "$d" ] && scan_dirs="${scan_dirs} $d"
done

if [ -z "${scan_dirs}" ]; then
    printf 'gate-banned: nothing to scan\n' >&2
    exit 1
fi

# shellcheck disable=SC2086 # scan_dirs is a deliberate word-split list
sources=$(find ${scan_dirs} -type f \( -name '*.c' -o -name '*.h' -o -name '*.m' \) | sort)

if [ -z "${sources}" ]; then
    printf 'gate-banned: no sources found under%s\n' "${scan_dirs}" >&2
    exit 1
fi

# NOTE: hits are passed as an ARGUMENT, never piped in. A `printf ... | report`
# pipeline runs report in a subshell, so `failures=$((failures + 1))` is
# discarded when the subshell exits -- the gate would print every violation and
# still exit 0, which is strictly worse than having no gate at all.
report() {
    # rule, explanation, hits ("file:line:text" per line; may be empty)
    rule=$1
    why=$2
    hits=$3

    [ -z "${hits}" ] && return 0

    printf '\ngate-banned: %s\n  %s\n' "${rule}" "${why}" >&2
    printf '%s\n' "${hits}" | while IFS= read -r hit; do
        [ -n "${hit}" ] && printf '    %s\n' "${hit}" >&2
    done
    failures=$((failures + 1))
    return 0
}

# Strip comments and string literals before matching, so a banned name
# mentioned in a comment (as several are, in this very tree) is not a hit.
# Not a full C lexer -- it drops // to end of line, /* */ on one line, and
# "..." contents -- but it is enough to keep the false-positive rate at zero
# for the patterns below.
strip() {
    sed -e 's://.*::' -e 's:/\*[^*]*\*/::g' -e 's:"[^"]*"::g' "$1"
}

scan() {
    # pattern, rule, why
    pattern=$1
    rule=$2
    why=$3
    hits=""
    for f in ${sources}; do
        matched=$(strip "$f" | grep -nE "${pattern}" | grep -v -- "->" | grep -vE "=.*\\[" | sed "s|^|${f}:|" || true)
        [ -n "${matched}" ] && hits="${hits}${matched}
"
    done
    report "${rule}" "${why}" "${hits}"
}

# --- unbounded and unsafe libc ---------------------------------------------

scan '\b(strcpy|strcat|sprintf|vsprintf|gets|scanf|sscanf|atoi|atol|atof|system|tmpnam|mktemp|strtok)[ \t]*\(' \
    'unbounded or unsafe libc function' \
    'These have no length checking or no error reporting. Use the bounded form, or a pwc_ helper.'

scan '\balloca[ \t]*\(' \
    'alloca' \
    'Unbounded stack growth with no failure mode. Use a kernel arena (packwandc.md 3.4).'

# --- allocation -------------------------------------------------------------
#
# Exception, with reason: kernel/arena.c and kernel/slab.c ARE the allocator,
# so they are the one place the raw functions may appear. Everything else goes
# through pwc_arena_* / pwc_slab_*.

alloc_hits=""
for f in ${sources}; do
    case "$f" in
        kernel/arena.c | kernel/slab.c) continue ;;
    esac
    matched=$(strip "$f" | grep -nE '\b(malloc|calloc|realloc|free|aligned_alloc|strdup)[ \t]*\(' |
        sed "s|^|${f}:|" || true)
    [ -n "${matched}" ] && alloc_hits="${alloc_hits}${matched}
"
done
report 'direct allocation outside the kernel allocator' \
    'Only kernel/arena.c and kernel/slab.c may call these. See packwandc.md 3.4.' \
    "${alloc_hits}"

# --- language constructs ----------------------------------------------------

# A VLA is also caught by -Wvla at compile time; this catches it in headers
# that no translation unit happens to include yet.
scan '\[[ \t]*[a-z_][a-zA-Z0-9_]*[ \t]*\][ \t]*;' \
    'possible variable-length array' \
    'Array bounds must be constant expressions. See packwandc.md 3.4.'

# NOTE: these two are exclusions, not matches, and grep -E has no negative
# lookahead -- an ERE like (?!once) matches nothing and would leave the rule
# silently dead. Both are therefore written as match-then-filter.

scan_except() {
    # pattern, exclude_pattern, rule, why
    pattern=$1
    exclude=$2
    rule=$3
    why=$4
    hits=""
    for f in ${sources}; do
        matched=$(strip "$f" | grep -nE "${pattern}" | grep -vE "${exclude}" | sed "s|^|${f}:|" || true)
        [ -n "${matched}" ] && hits="${hits}${matched}
"
    done
    report "${rule}" "${why}" "${hits}"
}

scan_except '^[ \t]*goto[ \t]+[a-zA-Z_]' '^[0-9]+:[ \t]*goto[ \t]+(cleanup|fail|done)[ \t]*;' \
    'goto to a label other than cleanup/fail/done' \
    'Only the forward-jump-to-cleanup idiom is permitted (packwandc.md 7.5).'

scan_except '^[ \t]*#[ \t]*pragma[ \t]+' '^[0-9]+:[ \t]*#[ \t]*pragma[ \t]+once' \
    'non-once #pragma' \
    'Compiler-specific pragmas belong in arch/ behind a documented shim.'

# Floating point has no place in the kernel: it complicates the ABI, some
# platforms save FP state lazily, and nothing here does arithmetic that needs
# it. Modules may use it if a platform API demands it.
fp_hits=""
for f in ${sources}; do
    case "$f" in
        kernel/*) ;;
        *) continue ;;
    esac
    matched=$(strip "$f" | grep -nE '\b(float|double)\b' | sed "s|^|${f}:|" || true)
    [ -n "${matched}" ] && fp_hits="${fp_hits}${matched}
"
done
report 'floating point in kernel/' \
    'The kernel is integer-only. See packwandc.md 7.5.' \
    "${fp_hits}"

# --- third-party includes ---------------------------------------------------
#
# The zero-dependency rule (packwandc.md 1.1, 8.4). An #include <...> is only
# allowed for the C standard library and for platform SDK headers, which live
# in arch/.

inc_hits=""
allowed_std='stddef|stdint|stdbool|stdarg|stdlib|string|stdio|limits|errno|assert|inttypes|stdalign|stdnoreturn|iso646|time|signal|threads|stdatomic'
for f in ${sources}; do
    case "$f" in
        arch/*) continue ;; # platform SDK headers are the point of arch/
    esac
    matched=$(grep -nE '^[ \t]*#[ \t]*include[ \t]*<' "$f" |
        grep -vE "<(${allowed_std})\.h>" | sed "s|^|${f}:|" || true)
    [ -n "${matched}" ] && inc_hits="${inc_hits}${matched}
"
done
report 'non-standard system include outside arch/' \
    'packwandc has no third-party dependencies. Platform headers belong in arch/.' \
    "${inc_hits}"

# --- mutable global state ---------------------------------------------------
#
# Kernel state lives in the pwc_kernel struct, threaded through explicitly, so
# that lifetime and locking are visible at every call site.

glob_hits=""
for f in ${sources}; do
    case "$f" in
        kernel/boot.c) continue ;; # owns the single kernel instance
    esac
    matched=$(strip "$f" |
        grep -nE '^[a-zA-Z_][a-zA-Z0-9_ ]*[ \t*]+[a-zA-Z_][a-zA-Z0-9_]*[ \t]*(=[^=]|;)' |
        grep -vE '^\s*[0-9]+:\s*(static|const|typedef|extern|struct|union|enum)\b' |
        grep -vE '\(' | sed "s|^|${f}:|" || true)
    [ -n "${matched}" ] && glob_hits="${glob_hits}${matched}
"
done
report 'non-static file-scope variable' \
    'Mutable globals hide lifetime and locking. Thread state through pwc_kernel.' \
    "${glob_hits}"

# --- function length --------------------------------------------------------
#
# clang-tidy caps cognitive complexity; it has no line-count check, so the
# 80-line cap from packwandc.md 7.5 is enforced here.

len_hits=$(awk '
    /^[a-zA-Z_].*\)[ \t]*\{[ \t]*$/ { infn = 1; start = FNR; name = $0; len = 0 }
    infn { len++ }
    /^\}/ {
        if (infn && len > 80) {
            printf("%s:%d: function is %d lines (cap 80)\n", FILENAME, start, len)
        }
        infn = 0
    }
' ${sources} || true)
report 'function exceeds the 80-line cap' \
    'Split it. See packwandc.md 7.5.' \
    "${len_hits}"

# --- result ------------------------------------------------------------------

if [ "${failures}" -gt 0 ]; then
    printf '\ngate-banned: %d rule(s) violated\n' "${failures}" >&2
    exit 1
fi

printf 'gate-banned: ok (%d files)\n' "$(printf '%s\n' "${sources}" | wc -l | tr -d ' ')"


