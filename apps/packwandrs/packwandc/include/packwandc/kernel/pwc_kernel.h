#ifndef PACKWANDC_KERNEL_PWC_KERNEL_H
#define PACKWANDC_KERNEL_PWC_KERNEL_H
#include "packwandc/uapi/pwc_syscall.h"
typedef struct pwc_boot_config {
    uint32_t handle_capacity;
    uint32_t worker_count;
} pwc_boot_config;
PWC_API PWC_NODISCARD pwc_status pwc_boot(const pwc_boot_config *config);
PWC_API void pwc_shutdown(void);
#endif
