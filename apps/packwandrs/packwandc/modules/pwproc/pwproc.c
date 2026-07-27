#include "packwandc/kernel/pwc_arch_proc.h"
#include "packwandc/kernel/pwc_boot_internal.h"

pwc_status pwc_proc_adopt(uint32_t pid, pwc_handle_t *out) {
    if (out == nullptr) {
        return PWC_EINVAL;
    }
    pwc_handle_table *const handles = pwc_kernel_handles();
    if (handles == nullptr) {
        return PWC_ECANCELED;
    }
    uintptr_t native = 0u;
    PWC_TRY(pwc_arch_proc_adopt(pid, &native));
    const pwc_status opened =
        pwc_handle_open(handles, PWC_OBJECT_PROCESS, PWC_RIGHT_WAIT | PWC_RIGHT_CLOSE, out);
    if (opened != PWC_OK) {
        (void) pwc_arch_proc_kill(native);
        return opened;
    }
    return pwc_handle_payload_set(handles, *out, PWC_OBJECT_PROCESS, native);
}

pwc_status pwc_proc_kill(pwc_handle_t process) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    if (handles == nullptr) {
        return PWC_ECANCELED;
    }
    uintptr_t native = 0u;
    PWC_TRY(pwc_handle_payload_get(handles, process, PWC_OBJECT_PROCESS, &native));
    const pwc_status killed = pwc_arch_proc_kill(native);
    const pwc_status closed = pwc_handle_close_table(handles, process);
    return killed != PWC_OK ? killed : closed;
}
pwc_status pwc_proc_exists(uint32_t pid, uint32_t *out_alive) { return pwc_arch_proc_exists(pid, out_alive); }
