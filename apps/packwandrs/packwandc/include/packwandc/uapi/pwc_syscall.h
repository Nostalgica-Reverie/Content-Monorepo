/* packwandc syscall surface -- see packwandc.md 4.
 *
 * NOT A GENERATED FILE. It consumes syscalls.def directly as an X-macro, so
 * the number enum and the prototypes are derived from the same text at compile
 * time and cannot fall out of step with each other or with the dispatch table
 * in kernel/syscall.c, which consumes the same file.
 *
 * Only the artifacts that leave C -- the Rust binding and the golden ledger --
 * are generated, by scripts/gen-syscalls.sh, and CI diffs them.
 */
#ifndef PACKWANDC_UAPI_PWC_SYSCALL_H
#define PACKWANDC_UAPI_PWC_SYSCALL_H

#include "packwandc/uapi/pwc_abi.h"
#include "packwandc/uapi/pwc_handle.h"
#include "packwandc/uapi/pwc_sh.h"
#include "packwandc/uapi/pwc_status.h"
#include "packwandc/uapi/pwc_trace.h"

PWC_BEGIN_DECLS

typedef struct pwc_waitent {
    pwc_handle_t h;
    uint32_t events;
    uint32_t revents;
} pwc_waitent;

/* --- syscall numbers --------------------------------------------------- */

#define PWC_SYSCALL(nr, name, module, ret, ...) PWC_SYS_##name = (nr),
enum pwc_syscall_nr {
#include "packwandc/uapi/syscalls.def"
};
#undef PWC_SYSCALL

/* --- prototypes --------------------------------------------------------
 *
 * Each syscall is also an ordinary C function. There is no `pwc_syscall(nr,
 * ...)` trampoline in the public surface: a numbered dispatch entry point buys
 * nothing in a statically linked userspace library, and it would throw away
 * every argument type the compiler could otherwise check. The numbers exist
 * for the ledger, for tracing, and for the Rust binding -- not for dispatch.
 */

#define PWC_SYSCALL(nr, name, module, ret, ...) PWC_API PWC_NODISCARD ret name(__VA_ARGS__);
#include "packwandc/uapi/syscalls.def"
#undef PWC_SYSCALL

/* --- table metadata ---------------------------------------------------- */

/* Number of live syscalls, via the enum-counting idiom: one throwaway
 * enumerator per entry, then a final one that lands on the count because
 * enumerators start at zero.
 *
 * The obvious alternative -- `#define PWC_SYSCALL(...) +1` summed from a
 * leading 0 -- expands to a macro whose replacement list cannot be
 * parenthesised, which clang-tidy flags as bugprone-macro-parentheses. The
 * check is right in general, so the idiom changes rather than the check being
 * suppressed. */
#define PWC_SYSCALL(nr, name, module, ret, ...) PWC_SYSCALL_COUNTER_##name,
enum pwc_syscall_counter {
#include "packwandc/uapi/syscalls.def"
    PWC_SYSCALL_COUNT
};
#undef PWC_SYSCALL

static_assert(PWC_SYSCALL_COUNT > 0, "the syscall table must not be empty");

PWC_END_DECLS

#endif /* PACKWANDC_UAPI_PWC_SYSCALL_H */
