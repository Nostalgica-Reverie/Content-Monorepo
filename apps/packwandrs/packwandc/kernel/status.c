/* Status names and descriptions are generated from PWC_STATUS_LIST. */

#include "packwandc/kernel/pwc_boot_internal.h"
#include "packwandc/kernel/pwc_error.h"
#include "packwandc/uapi/pwc_status.h"
#include "packwandc/uapi/pwc_trace.h"

/* Thread-local by design. A shared record would be a data race on every
 * concurrent failure and would hand callers another thread's diagnosis; the
 * kernel runs a worker pool, so that is the normal case
 * rather than an edge one. */
static thread_local pwc_error_detail pwc_last_detail = {
    .struct_size = (uint32_t) sizeof(pwc_error_detail),
    .status = PWC_OK,
    .platform_code = 0,
    .line = 0u,
    .module = "core",
    .message = "no error recorded on this thread",
    .file = "kernel/status.c",
};

const pwc_error_detail *pwc_last_error(void) { return &pwc_last_detail; }

/* The single path onto the trace ring.
 *
 * Only static string pointers are ever copied, never caller data, so nothing
 * that reaches here can carry a secret into the trace.
 *
 * The write result is deliberately discarded at every call site: a full ring
 * drops this record and counts it, and failing to trace something must never
 * change what the caller is told. */
static void pwc_trace_emit(uint32_t level,
                           pwc_status status,
                           int32_t platform_code,
                           const char *module,
                           const char *message,
                           const char *file,
                           uint32_t line) {
    pwc_ktrace *const trace = pwc_kernel_ktrace();
    if (trace == nullptr) {
        /* Before boot, or after shutdown. Argument validation can and does fail
         * outside a booted kernel, and losing that trace beats a null
         * dereference inside the error path itself. */
        return;
    }
    const pwc_trace_record record = {
        .struct_size = (uint32_t) sizeof(pwc_trace_record),
        .level = level,
        .sequence = 0u, /* assigned by the ring on publication */
        .status = status,
        .platform_code = platform_code,
        .line = line,
        .reserved = 0u,
        .module = module,
        .message = message,
        .file = file,
    };
    (void) pwc_ktrace_write(trace, &record);
}

void pwc_trace_note(uint32_t level,
                    const char *module,
                    const char *message,
                    const char *file,
                    uint32_t line) {
    /* Notes never touch pwc_last_detail. That record's contract is "the last
     * thing that went wrong", and letting a successful bring-up overwrite it
     * would erase a diagnosis a caller is about to read. */
    pwc_trace_emit(level,
                   PWC_OK,
                   0,
                   module != nullptr ? module : "?",
                   message != nullptr ? message : "(no message)",
                   file != nullptr ? file : "?",
                   line);
}

pwc_status pwc_error_record(pwc_status status,
                            const char *module,
                            const char *message,
                            int32_t platform_code,
                            const char *file,
                            uint32_t line) {
    /* Successes never overwrite the record: the getter's contract is "the last
     * thing that went wrong", and a success clobbering it would make the detail
     * useless exactly when a caller is between a failing call and reading it. */
    if (status >= PWC_OK) {
        return status;
    }

    /* Never NULL: consumers log these directly, and a null check at every read
     * would be noise that only ever fires on a caller bug here. */
    const char *const safe_module = module != nullptr ? module : "?";
    const char *const safe_message = message != nullptr ? message : "(no message)";
    const char *const safe_file = file != nullptr ? file : "?";

    pwc_last_detail = (pwc_error_detail){
        .struct_size = (uint32_t) sizeof(pwc_error_detail),
        .status = status,
        .platform_code = platform_code,
        .line = line,
        .module = safe_module,
        .message = safe_message,
        .file = safe_file,
    };

    /* Every recorded failure is also traced.
     *
     * This is the whole reason the detail record and the trace ring share a
     * choke point: the thread-local holds exactly one failure and the next one
     * overwrites it, so anything that fails while nobody is looking is lost.
     * The ring keeps the last PWC_KTRACE_CAPACITY of them for the host to
     * drain, which is what makes a failure from three calls ago still
     * diagnosable. */
    pwc_trace_emit(PWC_TRACE_LEVEL_ERROR, status, platform_code, safe_module, safe_message, safe_file, line);

    return status;
}

const char *pwc_status_name(pwc_status status) {
    switch (status) {
#define PWC_STATUS_NAME_CASE(name, value, desc)                                                              \
    case (value):                                                                                            \
        return #name;
        PWC_STATUS_LIST(PWC_STATUS_NAME_CASE)
#undef PWC_STATUS_NAME_CASE
        default:
            /* Never NULL: callers log this directly and a null check at every
             * call site would be pure noise. */
            return "PWC_EUNKNOWN";
    }
}

const char *pwc_status_describe(pwc_status status) {
    switch (status) {
#define PWC_STATUS_DESC_CASE(name, value, desc)                                                              \
    case (value):                                                                                            \
        return (desc);
        PWC_STATUS_LIST(PWC_STATUS_DESC_CASE)
#undef PWC_STATUS_DESC_CASE
        default:
            return "unknown status code";
    }
}
