#ifndef PACKWANDC_KERNEL_PWC_ARCH_KEYS_H
#define PACKWANDC_KERNEL_PWC_ARCH_KEYS_H
#include "packwandc/uapi/pwc_status.h"
pwc_status pwc_arch_keys_save(const uint8_t *secret, size_t secret_len);
pwc_status pwc_arch_keys_load(uint8_t *buffer, size_t capacity, size_t *out_len);
pwc_status pwc_arch_keys_clear(void);
#endif
