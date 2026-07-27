#ifndef PACKWANDC_KERNEL_PWC_SCHED_H
#define PACKWANDC_KERNEL_PWC_SCHED_H
#include "packwandc/uapi/pwc_status.h"
enum { PWC_SCHED_MAX_WORKERS = 64 };
typedef struct pwc_sched {
    uint32_t worker_count;
    bool running;
} pwc_sched;
pwc_status pwc_sched_init(pwc_sched *sched, uint32_t worker_count);
void pwc_sched_shutdown(pwc_sched *sched);
#endif
