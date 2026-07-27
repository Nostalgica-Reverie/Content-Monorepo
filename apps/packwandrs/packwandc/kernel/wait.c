#include "packwandc/kernel/pwc_boot_internal.h"

pwc_status pwc_wait(pwc_waitent *ents, size_t n, int64_t timeout_ms, size_t *out_ready) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    if (handles == nullptr || ents == nullptr || out_ready == nullptr || n == 0u || timeout_ms < -1) {
        return PWC_EINVAL;
    }
    *out_ready = 0u;
    for (size_t i = 0u; i < n; ++i) {
        ents[i].revents = 0u;
        const pwc_status status = pwc_handle_validate(handles, ents[i].h, PWC_RIGHT_WAIT);
        if (status != PWC_OK) {
            return status;
        }
        ents[i].revents = ents[i].events;
        ++*out_ready;
    }
    return PWC_OK;
}
