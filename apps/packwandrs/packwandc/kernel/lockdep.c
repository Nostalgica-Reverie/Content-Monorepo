/* Lock-order verification -- the lockdep analogue.
 *
 * Locks carry a declared level and must be acquired in strictly increasing
 * order. A cycle in lock ordering is the one concurrency bug that cannot be
 * found by reading a single function, and it is usually reproduced only under
 * timing you do not control -- so the check is done eagerly, on every
 * acquisition, rather than left to a deadlock in production.
 *
 * WHY THIS ABORTS RATHER THAN RETURNING
 *
 * A returned PWC_EPERM is a value a caller can ignore, and a lock-order
 * violation is not a runtime condition to be handled: it is a programming
 * error that has already made the process's locking unsound. Aborting with the
 * full acquisition chain turns it into a stack trace at the exact offending
 * acquisition, which is the whole value of the check.
 *
 * In release builds (NDEBUG) the status is returned instead, per the spec's
 * "debug and sanitizer builds" wording -- an abort in a shipped desktop app
 * would trade a possible deadlock for a certain crash.
 */

#include "packwandc/kernel/pwc_lockdep.h"

#include <stdio.h>
#include <stdlib.h>

void pwc_lockdep_init(pwc_lockdep *state) { *state = (pwc_lockdep){0}; }

#ifndef NDEBUG
/* Print the chain that led here, innermost last, then abort.
 *
 * stderr and not ktrace: the ring is drained asynchronously by the host, and
 * this call does not return, so anything written there would be lost. */
static void pwc_lockdep_violation(const pwc_lockdep *state, uint32_t level) {
    (void) fprintf(stderr, "lockdep: lock order violation acquiring level %u\n", level);
    (void) fprintf(stderr, "lockdep: held chain, outermost first (%u deep):\n", state->depth);
    for (uint32_t i = 0u; i < state->depth; ++i) {
        (void) fprintf(stderr, "lockdep:   [%u] level %u\n", i, state->levels[i]);
    }
    (void) fprintf(stderr,
                   "lockdep: level %u must be greater than the innermost held level %u\n",
                   level,
                   state->levels[state->depth - 1u]);
    (void) fflush(stderr);
    abort();
}
#endif

pwc_status pwc_lockdep_acquire(pwc_lockdep *state, uint32_t level) {
    if (state == nullptr || level == 0u) {
        return PWC_EINVAL;
    }
    if (state->depth == PWC_LOCKDEP_DEPTH) {
        return PWC_EOVERFLOW;
    }
    if (state->depth > 0u && level <= state->levels[state->depth - 1u]) {
#ifndef NDEBUG
        pwc_lockdep_violation(state, level); /* does not return */
#endif
        return PWC_EPERM;
    }
    state->levels[state->depth] = level;
    ++state->depth;
    return PWC_OK;
}

pwc_status pwc_lockdep_release(pwc_lockdep *state, uint32_t level) {
    /* Releases must mirror acquisitions exactly. Releasing out of order is the
     * same class of bug, but it is reported rather than fatal: unwinding paths
     * are where this is most likely to be a recoverable mistake. */
    if (state == nullptr || state->depth == 0u || state->levels[state->depth - 1u] != level) {
        return PWC_EPERM;
    }
    --state->depth;
    return PWC_OK;
}
