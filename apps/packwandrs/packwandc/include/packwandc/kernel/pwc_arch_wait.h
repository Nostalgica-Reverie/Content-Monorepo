#ifndef PACKWANDC_KERNEL_PWC_ARCH_WAIT_H
#define PACKWANDC_KERNEL_PWC_ARCH_WAIT_H
#include "packwandc/uapi/pwc_status.h"
pwc_status pwc_arch_wait_timeout(int64_t timeout_ms);
#endif
