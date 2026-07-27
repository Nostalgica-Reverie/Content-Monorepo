#include "packwandc/kernel/pwc_lockdep.h"
#include <stdlib.h>
void pwc_lockdep_init(pwc_lockdep *state) { *state = (pwc_lockdep){0}; }
pwc_status pwc_lockdep_acquire(pwc_lockdep *state, uint32_t level) {
    if (state == nullptr || level == 0u) {
        return PWC_EINVAL;
    }
    if (state->depth == PWC_LOCKDEP_DEPTH) {
        return PWC_EOVERFLOW;
    }
    if (state->depth > 0u && level <= state->levels[state->depth - 1u]) {
        return PWC_EPERM;
    }
    state->levels[state->depth] = level;
    ++state->depth;
    return PWC_OK;
}
pwc_status pwc_lockdep_release(pwc_lockdep *state, uint32_t level) {
    if (state == nullptr || state->depth == 0u || state->levels[state->depth - 1u] != level) {
        return PWC_EPERM;
    }
    --state->depth;
    return PWC_OK;
}
