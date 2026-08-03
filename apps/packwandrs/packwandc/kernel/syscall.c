/* Core syscall implementations. */

#include "packwandc/uapi/pwc_syscall.h"
#include "packwandc/kernel/pwc_boot_internal.h"

pwc_status pwc_sys_version(uint32_t *out_major, uint32_t *out_minor) {
    /* Null checks first and unconditionally. Every syscall validates its own
     * arguments -- there is no trusted caller, because the Rust safe layer is
     * not the only possible one (tests call directly). */
    if (out_major == nullptr || out_minor == nullptr) {
        return PWC_EINVAL;
    }

    *out_major = PWC_ABI_VERSION_MAJOR;
    *out_minor = PWC_ABI_VERSION_MINOR;
    return PWC_OK;
}

const char *pwc_sys_status_name(pwc_status status) { return pwc_status_name(status); }
pwc_status pwc_handle_close(pwc_handle_t h) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    return handles == nullptr ? PWC_ECANCELED : pwc_handle_close_table(handles, h);
}
pwc_status pwc_handle_dup(pwc_handle_t h, uint32_t rights, pwc_handle_t *out) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    return handles == nullptr ? PWC_ECANCELED : pwc_handle_dup_table(handles, h, rights, out);
}
const pwc_error_detail *pwc_last_error_get(void) { return pwc_last_error(); }

pwc_status pwc_ktrace_drain(pwc_trace_record *out) {
    if (out == nullptr) {
        return PWC_EINVAL;
    }
    pwc_ktrace *const trace = pwc_kernel_ktrace();
    if (trace == nullptr) {
        return PWC_ECANCELED;
    }
    return pwc_ktrace_read(trace, out);
}

pwc_status pwc_ktrace_dropped(uint64_t *out) {
    if (out == nullptr) {
        return PWC_EINVAL;
    }
    pwc_ktrace *const trace = pwc_kernel_ktrace();
    if (trace == nullptr) {
        return PWC_ECANCELED;
    }
    *out = pwc_ktrace_drops(trace);
    return PWC_OK;
}

/* --- compile-time table self-check --------------------------------------
 *
 * Verifies that every syscall number is inside the range reserved for its
 * module (syscalls.def). This catches the copy-paste error of adding a pwfs
 * syscall in the pwproc range, which would otherwise only show up much later
 * as confusing trace output.
 *
 * Only the ranges for modules that actually exist are defined. Pre-declaring
 * the rest would be dead code, and -Wunused-macros correctly rejects it -- so
 * each module adds its own pair here as it lands.
 */

#define PWC_RANGE_core_LO   1
#define PWC_RANGE_core_HI   15
#define PWC_RANGE_pwfs_LO   16
#define PWC_RANGE_pwfs_HI   31
#define PWC_RANGE_pwproc_LO 32
#define PWC_RANGE_pwproc_HI 47
#define PWC_RANGE_pwkeys_LO 48
#define PWC_RANGE_pwkeys_HI 63
#define PWC_RANGE_pwipc_LO  64
#define PWC_RANGE_pwipc_HI  79
#define PWC_RANGE_pwsh_LO   192
#define PWC_RANGE_pwsh_HI   207

#define PWC_SYSCALL(nr, name, module, ret, ...)                                                              \
    static_assert((nr) >= PWC_RANGE_##module##_LO && (nr) <= PWC_RANGE_##module##_HI,                        \
                  #name " has a syscall number outside the range reserved for " #module);
#include "packwandc/uapi/syscalls.def"
#undef PWC_SYSCALL
