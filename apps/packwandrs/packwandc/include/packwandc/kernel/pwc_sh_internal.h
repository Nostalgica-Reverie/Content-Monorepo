/* pw4shell internals -- shared between the tokeniser and the dispatcher.
 *
 * Kernel-internal: Rust reaches pw4shell through the syscalls in
 * uapi/syscalls.def, never through this header. scripts/gate-uapi.sh enforces
 * that split.
 */
#ifndef PACKWANDC_KERNEL_PWC_SH_INTERNAL_H
#define PACKWANDC_KERNEL_PWC_SH_INTERNAL_H

#include "packwandc/uapi/pwc_handle.h"
#include "packwandc/uapi/pwc_sh.h"
#include "packwandc/uapi/pwc_status.h"

/* Tokenise one line into words. Defined in modules/pwsh/lexer.c.
 *
 * Also reachable as a syscall, because the frontend needs the same quoting
 * rules the kernel enforces -- reimplementing them in TypeScript is how a
 * console ends up disagreeing with itself about what `"a b"` means. */
pwc_status pwc_sh_parse(const uint8_t *line, size_t length, pwc_sh_command *out);

/* Where a built-in writes its output. Lines are sent as individual framed
 * messages on the port, so a consumer reading one frame gets one line. */
typedef struct pwsh_sink {
    pwc_handle_t port;
    bool has_port; /* false discards output; used by tests and by dry parses */
} pwsh_sink;

/* Emit one output line. Never fails the command it is reporting on: a full
 * port loses the line rather than turning a successful command into a failed
 * one, which matches ktrace's drop-rather-than-stall rule. */
void pwsh_emit(const pwsh_sink *sink, const char *text);

/* Compare a parsed word against a literal. */
bool pwsh_word_is(const pwc_sh_command *cmd, size_t index, const char *literal);

#endif /* PACKWANDC_KERNEL_PWC_SH_INTERNAL_H */
