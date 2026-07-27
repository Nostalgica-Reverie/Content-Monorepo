#ifndef PACKWANDC_KERNEL_PWC_LOCKDEP_H
#define PACKWANDC_KERNEL_PWC_LOCKDEP_H
#include "packwandc/uapi/pwc_status.h"
enum { PWC_LOCKDEP_DEPTH = 32 };
typedef struct pwc_lockdep {
    uint32_t levels[PWC_LOCKDEP_DEPTH];
    uint32_t depth;
} pwc_lockdep;
void pwc_lockdep_init(pwc_lockdep *state);
pwc_status pwc_lockdep_acquire(pwc_lockdep *state, uint32_t level);
pwc_status pwc_lockdep_release(pwc_lockdep *state, uint32_t level);
#endif
