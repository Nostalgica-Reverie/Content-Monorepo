#include "packwandc/kernel/pwc_arch_wait.h"
pwc_status pwc_arch_wait_timeout(int64_t timeout_ms) { return timeout_ms < -1 ? PWC_EINVAL : PWC_OK; }
