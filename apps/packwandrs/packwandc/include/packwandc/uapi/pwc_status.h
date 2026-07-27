/* packwandc status codes -- see packwandc.md 3.1.
 *
 * Errno-shaped: PWC_OK is zero, every failure is negative, and no function
 * ever returns a positive status. Callers test `< 0`, or better, use the
 * PWC_TRY macro.
 */
#ifndef PACKWANDC_UAPI_PWC_STATUS_H
#define PACKWANDC_UAPI_PWC_STATUS_H

#include "packwandc/uapi/pwc_abi.h"

PWC_BEGIN_DECLS

typedef int32_t pwc_status;

typedef struct pwc_error_detail {
    const char *module;
    const char *message;
    int32_t platform_code;
} pwc_error_detail;

/* The status table is an X-macro so that the enum and the name mapping in
 * kernel/status.c cannot drift -- the same problem syscalls.def solves for the
 * syscall table, and the same solution. */
#define PWC_STATUS_LIST(X)                                                                                   \
    X(PWC_OK, 0, "success")                                                                                  \
    X(PWC_EINVAL, -1, "invalid argument")                                                                    \
    X(PWC_ENOENT, -2, "no such object")                                                                      \
    X(PWC_EPERM, -3, "operation not permitted by handle rights")                                             \
    X(PWC_EAGAIN, -4, "would block, retry")                                                                  \
    X(PWC_ENOMEM, -5, "allocation failed or arena exhausted")                                                \
    X(PWC_EBADF, -6, "unknown handle")                                                                       \
    X(PWC_ESTALE, -7, "handle generation mismatch, object was freed")                                        \
    X(PWC_ENOSYS, -8, "syscall not implemented on this platform")                                            \
    X(PWC_EIO, -9, "platform I/O failure")                                                                   \
    X(PWC_ETIMEDOUT, -10, "deadline expired")                                                                \
    X(PWC_ECANCELED, -11, "operation cancelled")                                                             \
    X(PWC_EOVERFLOW, -12, "value or buffer too large")

#define PWC_STATUS_ENUM_ENTRY(name, value, desc) name = (value),
enum pwc_status_code { PWC_STATUS_LIST(PWC_STATUS_ENUM_ENTRY) };
#undef PWC_STATUS_ENUM_ENTRY

/* Propagate a failing status unchanged. The kernel and modules use this rather
 * than hand-written `if (st < 0) return st;`, which is where single-exit C
 * historically grows bugs.
 *
 * Wrapped in do/while(0) with a unique-ish local so it composes inside if/else
 * without braces surprises. */
#define PWC_TRY(expr)                                                                                        \
    do {                                                                                                     \
        const pwc_status pwc__try_st = (expr);                                                               \
        if (pwc__try_st < PWC_OK) {                                                                          \
            return pwc__try_st;                                                                              \
        }                                                                                                    \
    } while (0)

/* True when a status indicates success. Present so call sites read as prose;
 * `st < 0` remains correct and is not discouraged. */
#define PWC_OKAY(st) ((st) >= PWC_OK)

/* Stable identifier for a status, e.g. "PWC_EINVAL". Never NULL: an unknown
 * code yields "PWC_EUNKNOWN". This is the direct-call form; the same thing is
 * reachable as syscall 2 (pwc_sys_status_name). */
PWC_API PWC_NODISCARD const char *pwc_status_name(pwc_status status);

/* Short human-readable description, e.g. "invalid argument". Never NULL.
 * For logs and test failure messages -- not user-facing copy. */
PWC_API PWC_NODISCARD const char *pwc_status_describe(pwc_status status);

PWC_API PWC_NODISCARD const pwc_error_detail *pwc_last_error(void);

PWC_END_DECLS

#endif /* PACKWANDC_UAPI_PWC_STATUS_H */
