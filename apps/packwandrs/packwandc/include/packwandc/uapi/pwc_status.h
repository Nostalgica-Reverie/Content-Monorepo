/* packwandc status codes.
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

/* --- the last-error detail record ---------------------------------------
 *
 * An errno-shaped integer says what class of thing went wrong and nothing
 * about which call, where, or why the platform refused. Every failing path
 * that knows more than its status code records it here,
 * and the Rust SDK folds status + detail into one Error.
 *
 * `status` is carried in the record deliberately. A thread-local "last error"
 * is stale-prone by construction: a caller holding PWC_EIO has no way to know
 * whether the detail it reads describes that failure or an earlier one. Pairing
 * the detail with the status it was recorded for lets the reader check, which
 * is the difference between a diagnostic and a guess.
 *
 * `file` is the __FILE__ of the recording site. -ffile-prefix-map
 * 7.3) has already rewritten it to a repo-relative path, so it is stable across
 * machines and safe to log.
 */
PWC_ABI_PACKED_BEGIN
typedef struct pwc_error_detail {
    uint32_t struct_size;  /* sizeof(pwc_error_detail); forward compatibility */
    int32_t status;        /* the status this record was recorded for */
    int32_t platform_code; /* GetLastError()/errno/D-Bus code, 0 if none */
    uint32_t line;         /* __LINE__ of the recording site */
    const char *module;    /* static, never NULL: "core", "pwfs", "arch/win32" */
    const char *message;   /* static, never NULL */
    const char *file;      /* static, never NULL */
} pwc_error_detail;
PWC_ABI_PACKED_END

static_assert(sizeof(pwc_error_detail) == 40, "pwc_error_detail is part of the wire ABI");

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

/* The calling thread's most recent error detail. Never NULL, and never stale
 * in the dangerous sense: before any failure is recorded on this thread it
 * reports status PWC_OK with a "no error recorded" message.
 *
 * Read it only after a call returned a failing status, and compare its
 * `status` field against the one you were handed -- an unequal pair means the
 * failing call recorded no detail and you are looking at an older record. */
PWC_API PWC_NODISCARD const pwc_error_detail *pwc_last_error(void);

PWC_END_DECLS

#endif /* PACKWANDC_UAPI_PWC_STATUS_H */
