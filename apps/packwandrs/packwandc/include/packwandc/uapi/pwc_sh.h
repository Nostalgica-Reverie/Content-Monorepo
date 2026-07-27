/* pw4shell -- the packwand command language (packwandc.md 5.8).
 *
 * THE LANGUAGE
 *
 * One command per line. A command is a verb, then arguments, then flags:
 *
 *     list --side client
 *     add sodium
 *     preflight modpacks/lce-common
 *     packwand doctor
 *     echo "quoted argument"     # trailing comment
 *
 * That is the whole grammar. No variables, no pipelines, no substitution, no
 * control flow. The restraint is the point: this is a console for driving
 * packwand, not a general-purpose shell, and every construct left out is one
 * that cannot be used to reach somewhere it should not.
 *
 * WHAT IT CAN DO
 *
 * Kernel built-ins only. The C module never spawns a process. A command either
 * resolves to a built-in the kernel implements or returns parsed argv to its
 * host. The desktop host may invoke only Packwand's fixed, bundled CLI; it does
 * not hand the line to an OS shell or select a program from the input.
 *
 * WHERE COMMANDS LIVE
 *
 * Split across the FFI boundary, because the operations live on both sides:
 *
 *   - The kernel owns the *language* -- tokenising, quoting, validation -- and
 *     the built-ins that need kernel state (trace, version, status, fs).
 *   - Packwand CLI operations live in the Rust crates and cannot be called
 *     from C. For those, pwc_sh_exec parses and validates the line,
 *     reports PWC_ENOSYS, and hands back the parsed argv for the host to
 *     dispatch.
 *
 * So a caller always gets a *parsed, validated* command even when the kernel
 * cannot run it, and the quoting rules are defined in exactly one place rather
 * than reimplemented in TypeScript.
 */
#ifndef PACKWANDC_UAPI_PWC_SH_H
#define PACKWANDC_UAPI_PWC_SH_H

#include "packwandc/uapi/pwc_abi.h"
#include "packwandc/uapi/pwc_status.h"

PWC_BEGIN_DECLS

enum {
    /* Longest accepted input line, in bytes. */
    PWC_SH_MAX_LINE = 1024,
    /* Maximum words in one command, verb included. */
    PWC_SH_MAX_ARGS = 16,
    /* Longest single word, NUL included. */
    PWC_SH_MAX_ARG = 128,
};

PWC_ABI_PACKED_BEGIN
typedef struct pwc_sh_command {
    uint32_t struct_size; /* sizeof(pwc_sh_command); forward compatibility */
    uint32_t argc;        /* words present; 0 for a blank or comment-only line */
    uint32_t arglen[PWC_SH_MAX_ARGS];
    /* Each word is NUL-terminated as well as length-carrying: the length is
     * authoritative, and the terminator is there so a C consumer cannot read
     * off the end by treating it as a plain string. */
    uint8_t argv[PWC_SH_MAX_ARGS][PWC_SH_MAX_ARG];
} pwc_sh_command;
PWC_ABI_PACKED_END

static_assert(sizeof(pwc_sh_command) ==
                  8u + (4u * PWC_SH_MAX_ARGS) + ((size_t) PWC_SH_MAX_ARGS * PWC_SH_MAX_ARG),
              "pwc_sh_command is part of the wire ABI");

PWC_END_DECLS

#endif /* PACKWANDC_UAPI_PWC_SH_H */
