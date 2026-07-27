#include "packwandc/kernel/pwc_arch_keys.h"
pwc_status pwc_arch_keys_save(const uint8_t *secret, size_t secret_len) {
    (void) secret;
    (void) secret_len;
    return PWC_ENOSYS;
}
pwc_status pwc_arch_keys_load(uint8_t *buffer, size_t capacity, size_t *out_len) {
    (void) buffer;
    (void) capacity;
    (void) out_len;
    return PWC_ENOSYS;
}
pwc_status pwc_arch_keys_clear(void) { return PWC_ENOSYS; }
