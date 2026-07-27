#include "packwandc/kernel/pwc_arch_keys.h"
#include "packwandc/uapi/pwc_syscall.h"
pwc_status pwc_keys_save(const uint8_t *secret, size_t secret_len) {
    if (secret == nullptr || secret_len == 0u) {
        return PWC_EINVAL;
    }
    return pwc_arch_keys_save(secret, secret_len);
}
pwc_status pwc_keys_load(uint8_t *buffer, size_t capacity, size_t *out_len) {
    if (buffer == nullptr || capacity == 0u || out_len == nullptr) {
        return PWC_EINVAL;
    }
    return pwc_arch_keys_load(buffer, capacity, out_len);
}
pwc_status pwc_keys_clear(void) { return pwc_arch_keys_clear(); }
