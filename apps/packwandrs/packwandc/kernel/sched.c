#include "packwandc/kernel/pwc_sched.h"
pwc_status pwc_sched_init(pwc_sched *sched, uint32_t worker_count) {
    if (sched == nullptr || worker_count == 0u || worker_count > PWC_SCHED_MAX_WORKERS) {
        return PWC_EINVAL;
    }
    sched->worker_count = worker_count;
    sched->running = true;
    return PWC_OK;
}
void pwc_sched_shutdown(pwc_sched *sched) {
    if (sched != nullptr) {
        sched->running = false;
        sched->worker_count = 0u;
    }
}
