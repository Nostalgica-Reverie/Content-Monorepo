#!/usr/bin/env sh
# Generate the artifacts that leave C, from include/packwandc/uapi/syscalls.def.
#
#   crates/packwandc-sys/src/bindings.rs   the Rust extern block
#   tests/golden/syscalls.txt              the frozen syscall-number ledger
#
# Both are CHECKED IN. CI regenerates and diffs them, so a syscall cannot be
# added, renamed, or renumbered without the change showing up in review.
# The generated bindings mirror the C ABI.
#
# The C artifacts are NOT generated: uapi/pwc_syscall.h and kernel/syscall.c
# include syscalls.def directly as an X-macro, so they cannot drift from it or
# from each other. Only the Rust side needs a translation step, and bindgen is
# deliberately not used for it -- bindgen would drag libclang into the build
# for two dozen lines of trivially derivable FFI.
#
# Usage:
#   scripts/gen-syscalls.sh          write the files
#   scripts/gen-syscalls.sh --check  exit non-zero if they are out of date

set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
root_dir=$(CDPATH='' cd -- "${script_dir}/.." && pwd)

def_file="${root_dir}/include/packwandc/uapi/syscalls.def"
bindings_out="${root_dir}/crates/packwandc-sys/src/bindings.rs"
golden_out="${root_dir}/tests/golden/syscalls.txt"

check_only=0
if [ $# -gt 0 ]; then
    case "$1" in
        --check) check_only=1 ;;
        *)
            printf 'usage: %s [--check]\n' "$0" >&2
            exit 2
            ;;
    esac
fi

if [ ! -f "${def_file}" ]; then
    printf 'gen-syscalls: missing %s\n' "${def_file}" >&2
    exit 1
fi

# --- the generator ----------------------------------------------------------
#
# awk reads syscalls.def, ignores comments and blank lines, and emits either
# the Rust binding or the ledger depending on MODE.

generate() {
    awk -v mode="$1" '
        function fail(msg) {
            printf("gen-syscalls: %s:%d: %s\n", FILENAME, FNR, msg) > "/dev/stderr"
            exit_code = 1
            exit 1
        }

        # Map a C type to its Rust spelling. Unknown types are a hard error
        # rather than a guess -- a silently wrong FFI signature is exactly the
        # class of bug this whole layer exists to avoid.
        function rust_type(c,    t) {
            # Normalise by removing every space, so "const char *",
            # "const char*" and "constchar *" all collapse to one spelling and
            # the table below needs a single entry per type.
            t = c
            gsub(/[ \t]/, "", t)

            if (t == "void")            return ""
            if (t == "pwc_status")      return "i32"
            if (t == "uint8_t")         return "u8"
            if (t == "uint16_t")        return "u16"
            if (t == "uint32_t")        return "u32"
            if (t == "uint64_t")        return "u64"
            if (t == "int32_t")         return "i32"
            if (t == "int64_t")         return "i64"
            if (t == "size_t")          return "usize"
            if (t == "size_t*")         return "*mut usize"
            if (t == "bool")            return "bool"
            if (t == "uint8_t*")        return "*mut u8"
            if (t == "uint32_t*")       return "*mut u32"
            if (t == "uint64_t*")       return "*mut u64"
            if (t == "constuint8_t*")   return "*const u8"
            if (t == "char*")           return "*mut core::ffi::c_char"
            if (t == "constchar*")      return "*const core::ffi::c_char"
            if (t == "void*")           return "*mut core::ffi::c_void"
            if (t == "constvoid*")      return "*const core::ffi::c_void"
            if (t == "pwc_handle_t")    { uses_handle = 1; return "PwcHandle" }
            if (t == "pwc_handle_t*")   { uses_handle = 1; return "*mut PwcHandle" }
            if (t == "pwc_waitent*") return "*mut core::ffi::c_void"
            if (t == "constpwc_error_detail*") {
                uses_error_detail = 1
                return "*const PwcErrorDetail"
            }
            if (t == "pwc_trace_record*") {
                uses_trace_record = 1
                return "*mut PwcTraceRecord"
            }
            if (t == "pwc_sh_command*") {
                uses_sh_command = 1
                return "*mut PwcShCommand"
            }

            fail("unmapped C type \"" c "\" -- add it to rust_type() in gen-syscalls.sh")
        }

        # snake_case -> UpperCamelCase, for the syscall-number enum variants.
        function camel(s,    parts, n, i, out) {
            n = split(s, parts, "_")
            out = ""
            for (i = 1; i <= n; i++) {
                if (parts[i] == "") continue
                out = out toupper(substr(parts[i], 1, 1)) substr(parts[i], 2)
            }
            return out
        }

        BEGIN { count = 0; exit_code = 0 }

        # Strip block comments spanning whole lines, plus blank lines.
        /^[ \t]*\/\*/  { in_comment = 1 }
        in_comment     { if (/\*\//) in_comment = 0; next }
        /^[ \t]*$/     { next }
        /^[ \t]*\/\//  { next }

        /^[ \t]*PWC_SYSCALL[ \t]*\(/ {
            line = $0
            if (line !~ /\)[ \t]*$/) {
                fail("PWC_SYSCALL entries must be a single line ending in )")
            }
            sub(/^[ \t]*PWC_SYSCALL[ \t]*\(/, "", line)
            sub(/\)[ \t]*$/, "", line)

            n = split(line, f, ",")
            if (n < 4) fail("PWC_SYSCALL needs at least nr, name, module, return type")

            nr = f[1]; name = f[2]; module = f[3]; ret = f[4]
            gsub(/^[ \t]+|[ \t]+$/, "", nr)
            gsub(/^[ \t]+|[ \t]+$/, "", name)
            gsub(/^[ \t]+|[ \t]+$/, "", module)

            if (nr !~ /^[0-9]+$/)               fail("syscall number must be a decimal integer")
            if (nr + 0 <= 0)                    fail("syscall numbers start at 1")
            if (nr + 0 <= last_nr)              fail("syscall numbers must be strictly ascending (append-only)")
            if (name !~ /^pwc_[a-z0-9_]+$/)     fail("syscall names must match pwc_[a-z0-9_]+")
            if (seen_name[name])                fail("duplicate syscall name " name)
            seen_name[name] = 1
            last_nr = nr + 0

            count++
            nrs[count] = nr; names[count] = name; modules[count] = module

            # Build the Rust parameter list.
            params = ""
            for (i = 5; i <= n; i++) {
                p = f[i]
                gsub(/^[ \t]+|[ \t]+$/, "", p)
                if (p == "void" || p == "") continue

                # Split "uint32_t *out_major" into type and identifier: the
                # identifier is the trailing [A-Za-z_][A-Za-z0-9_]* run.
                if (match(p, /[A-Za-z_][A-Za-z0-9_]*$/) == 0) {
                    fail("parameter \"" p "\" has no identifier")
                }
                pname = substr(p, RSTART, RLENGTH)
                ptype = substr(p, 1, RSTART - 1)
                if (ptype ~ /^[ \t]*$/) fail("parameter \"" p "\" has no type")

                if (params != "") params = params ", "
                params = params pname ": " rust_type(ptype)
            }

            rret = rust_type(ret)
            # A doc comment per binding: the packwandrs workspace sets
            # missing_docs = "deny", and these are public items.
            sigs[count] = "    /// Syscall " nr ", `" name "` (" module " module).\n"
            sigs[count] = sigs[count] "    pub fn " name "(" params ")"
            if (rret != "") sigs[count] = sigs[count] " -> " rret
            sigs[count] = sigs[count] ";"
            next
        }

        # Anything else at column zero that is not a comment is a format error.
        /^[^ \t\/*#]/ { fail("unrecognised line in syscalls.def") }

        END {
            if (exit_code != 0) exit exit_code
            if (count == 0) {
                printf("gen-syscalls: syscalls.def defines no syscalls\n") > "/dev/stderr"
                exit 1
            }

            if (mode == "golden") {
                for (i = 1; i <= count; i++) printf("%s %s %s\n", nrs[i], names[i], modules[i])
                exit 0
            }

            print "// @generated by scripts/gen-syscalls.sh -- DO NOT EDIT."
            print "//"
            print "// Source: include/packwandc/uapi/syscalls.def"
            print "// Regenerate with `just gen-packwandc`; CI diffs this file against a fresh"
            print "// run, so an edit here fails the build rather than silently diverging from"
            print "// the C ABI."
            print ""
            # Emitted only when a syscall actually takes or returns a handle;
            # an unconditional import would be an unused-import warning until
            # the first handle syscall lands in phase 1.
            if (uses_handle) {
                print "use crate::PwcHandle;"
            }
            if (uses_error_detail) {
                print "use crate::PwcErrorDetail;"
            }
            if (uses_trace_record) {
                print "use crate::PwcTraceRecord;"
            }
            if (uses_sh_command) {
                print "use crate::PwcShCommand;"
            }
            if (uses_handle || uses_error_detail || uses_trace_record || uses_sh_command) {
                print ""
            }
            print "/// Syscall numbers. Append-only and frozen: see tests/golden/syscalls.txt."
            print "#[repr(i32)]"
            print "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]"
            print "pub enum SyscallNr {"
            for (i = 1; i <= count; i++) {
                printf("    /// `%s`, provided by the %s module.\n", names[i], modules[i])
                printf("    %s = %s,\n", camel(names[i]), nrs[i])
            }
            print "}"
            print ""
            printf("/// Number of live syscalls. Must equal `PWC_SYSCALL_COUNT` in C.\n")
            printf("pub const SYSCALL_COUNT: usize = %d;\n", count)
            print ""
            print "unsafe extern \"C\" {"
            for (i = 1; i <= count; i++) print sigs[i]
            print "}"
            exit 0
        }
    ' "${def_file}"
}

emit() {
    # mode, destination
    tmp="${2}.tmp.$$"
    mkdir -p "$(dirname -- "$2")"

    # Check explicitly rather than relying on `set -e`: emit is called from a
    # `|| rc=1` list, which suspends errexit for everything it runs. Without
    # this the generator could fail and the truncated output would still be
    # moved into place.
    if ! generate "$1" > "${tmp}"; then
        rm -f "${tmp}"
        return 1
    fi

    if [ "${check_only}" -eq 1 ]; then
        if [ ! -f "$2" ]; then
            printf 'gen-syscalls: %s does not exist; run scripts/gen-syscalls.sh\n' "$2" >&2
            rm -f "${tmp}"
            return 1
        fi
        if ! diff -u "$2" "${tmp}" >&2; then
            printf 'gen-syscalls: %s is out of date; run scripts/gen-syscalls.sh and commit\n' "$2" >&2
            rm -f "${tmp}"
            return 1
        fi
        rm -f "${tmp}"
        return 0
    fi

    mv -f "${tmp}" "$2"
    printf 'gen-syscalls: wrote %s\n' "$2"
}

rc=0
emit rust "${bindings_out}" || rc=1
emit golden "${golden_out}" || rc=1
exit "${rc}"



