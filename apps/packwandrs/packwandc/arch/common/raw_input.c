#include "packwandc/uapi/pwc_raw_input.h"

pwc_status pwc_raw_input_start(uintptr_t native_window) {
    (void) native_window;
    return PWC_ENOSYS;
}

void pwc_raw_input_stop(void) {
}

pwc_status pwc_raw_input_read(pwc_raw_input_event *out) {
    return out == nullptr ? PWC_EINVAL : PWC_ENOSYS;
}

pwc_status pwc_raw_input_dropped(uint64_t *out) {
    if (out == nullptr) {
        return PWC_EINVAL;
    }
    *out = 0u;
    return PWC_ENOSYS;
}
