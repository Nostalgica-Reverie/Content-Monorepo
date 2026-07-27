#include "packwandc/kernel/pwc_arch_proc.h"
pwc_status pwc_arch_proc_adopt(uint32_t pid, uintptr_t *out_native) {
    (void) pid;
    (void) out_native;
    return PWC_ENOSYS;
}
pwc_status pwc_arch_proc_kill(uintptr_t native) {
    (void) native;
    return PWC_ENOSYS;
}
pwc_status pwc_arch_proc_exists(uint32_t pid, uint32_t *out_alive) {
    (void) pid;
    (void) out_alive;
    return PWC_ENOSYS;
}
