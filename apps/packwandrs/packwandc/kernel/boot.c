#include "packwandc/kernel/pwc_boot_internal.h"
#include "packwandc/kernel/pwc_handle_table.h"
#include "packwandc/kernel/pwc_kernel.h"

typedef struct pwc_kernel_instance {
    bool booted;
    pwc_handle_table handles;
} pwc_kernel_instance;

static pwc_kernel_instance pwc_kernel;

pwc_status pwc_boot(const pwc_boot_config *config) {
    uint32_t capacity = PWC_HANDLE_CAPACITY_MAX;
    if (config != nullptr) {
        capacity = config->handle_capacity;
    }
    if (capacity == 0u || capacity > PWC_HANDLE_CAPACITY_MAX) {
        return PWC_EINVAL;
    }
    if (pwc_kernel.booted) {
        return PWC_EAGAIN;
    }
    pwc_handle_table_init(&pwc_kernel.handles, capacity);
    pwc_kernel.booted = true;
    return PWC_OK;
}

void pwc_shutdown(void) {
    pwc_kernel.booted = false;
    pwc_handle_table_init(&pwc_kernel.handles, PWC_HANDLE_CAPACITY_MAX);
}

pwc_handle_table *pwc_kernel_handles(void) { return pwc_kernel.booted ? &pwc_kernel.handles : nullptr; }
