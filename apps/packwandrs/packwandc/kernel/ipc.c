#include "packwandc/kernel/pwc_boot_internal.h"
pwc_status pwc_ipc_port_create(pwc_handle_t *out) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    return handles == nullptr ? PWC_ECANCELED : pwc_handle_open(handles, PWC_OBJECT_PORT, PWC_RIGHT_ALL, out);
}
