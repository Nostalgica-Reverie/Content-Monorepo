/* Recording side of the last-error detail record.
 *
 * The *reading* side (pwc_last_error) is public uapi: Rust and any other
 * consumer may look at the record. Recording is kernel-internal, so that the
 * only things that can claim "here is why that failed" are the kernel, its
 * modules, and the arch backends. scripts/gate-uapi.sh enforces the split.
 *
 * Use the macros, not pwc_error_record directly -- they capture __FILE__ and
 * __LINE__ at the failure site, which is the part a caller cannot reconstruct.
 */
#ifndef PACKWANDC_KERNEL_PWC_ERROR_H
#define PACKWANDC_KERNEL_PWC_ERROR_H

#include "packwandc/uapi/pwc_status.h"

PWC_BEGIN_DECLS

/* Record a failure detail for the calling thread and return `status` unchanged,
 * so a failing path reads as one expression:
 *
 *     return PWC_FAIL(PWC_EIO, "arch/win32", "CredWriteW rejected the blob");
 *
 * `module`, `message` and `file` must be static storage -- the record holds the
 * pointers, it does not copy. Passing a stack buffer is a dangling read, which
 * is why there is no printf-style variant: formatting would need an allocation
 * this layer does not have.
 *
 * Recording a non-failing status is a no-op: it is never useful and it would
 * let a success quietly overwrite the detail a caller is about to read.
 */
pwc_status pwc_error_record(pwc_status status,
                            const char *module,
                            const char *message,
                            int32_t platform_code,
                            const char *file,
                            uint32_t line);

/* A failure the platform told us nothing more about. */
#define PWC_FAIL(status, module, message)                                                                    \
    pwc_error_record((status), (module), (message), 0, __FILE__, (uint32_t) __LINE__)

/* A failure carrying a platform code: GetLastError(), errno, a D-Bus error.
 * Prefer this wherever one is available -- it is the single most useful field
 * in the record and it is lost forever the moment the call returns. */
#define PWC_FAIL_PLATFORM(status, module, message, code)                                                     \
    pwc_error_record((status), (module), (message), (int32_t) (code), __FILE__, (uint32_t) __LINE__)

/* Emit a trace record for something that is not a failure -- module bring-up,
 * shutdown, a state change worth seeing in the log.
 *
 * Deliberately separate from pwc_error_record: the last-error detail record
 * must only ever describe failures, so a note must not touch it. Both feed the
 * same ktrace ring.
 *
 * Same storage rule as PWC_FAIL: `module` and `message` must be static. */
void pwc_trace_note(uint32_t level, const char *module, const char *message, const char *file, uint32_t line);

#define PWC_NOTE(level, module, message)                                                                     \
    pwc_trace_note((level), (module), (message), __FILE__, (uint32_t) __LINE__)

PWC_END_DECLS

#endif /* PACKWANDC_KERNEL_PWC_ERROR_H */
