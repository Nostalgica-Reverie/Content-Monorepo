#include "packwandc/kernel/pwc_arch_wait.h"
#include <windows.h>
pwc_status pwc_arch_wait_timeout(int64_t timeout_ms) {
    if (timeout_ms < -1) {
        return PWC_EINVAL;
    }
    if (timeout_ms > 0) {
        const uint64_t bounded = (uint64_t) timeout_ms > UINT32_MAX ? UINT32_MAX : (uint64_t) timeout_ms;
        Sleep((DWORD) bounded);
    }
    return PWC_OK;
}
