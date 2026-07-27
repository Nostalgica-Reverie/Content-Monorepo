#ifndef PACKWANDC_KERNEL_PWC_ARCH_PROC_H
#define PACKWANDC_KERNEL_PWC_ARCH_PROC_H
#include "packwandc/uapi/pwc_status.h"
pwc_status pwc_arch_proc_adopt(uint32_t pid, uintptr_t *out_native);
pwc_status pwc_arch_proc_kill(uintptr_t native);
pwc_status pwc_arch_proc_exists(uint32_t pid, uint32_t *out_alive);
#endif
