/* Lockdep must abort on an out-of-order acquisition (packwandc.md 3.6).
 *
 * THIS TEST USED TO BE UNABLE TO FAIL. It ended in `return 3` and CTest was
 * told WILL_FAIL, so any non-zero exit satisfied it -- including that `return 3`
 * itself. It passed identically whether lockdep aborted, quietly returned
 * PWC_EPERM, or was deleted outright. A rule that cannot fail is worse than no
 * rule, because it reads as coverage (handoff trap 15).
 *
 * It is now checked on output: CMake matches the violation banner with
 * PASS_REGULAR_EXPRESSION and rejects the NOT-REACHED marker below via
 * FAIL_REGULAR_EXPRESSION. Silence fails. Returning instead of aborting fails.
 */

#include "packwandc/kernel/pwc_lockdep.h"

#include <stdio.h>
#ifdef _WIN32
#include <stdlib.h>
#include <windows.h>
#endif

/* Windows Error Reporting catches abort() and spends ~22 seconds deciding what
 * to do with it, and on an interactive desktop it puts up a modal dialog. Both
 * are intolerable in a test suite. Suppressing that is test-harness business,
 * not the kernel's -- kernel/lockdep.c stays free of platform code, per the
 * arch/ split in packwandc.md 2. */
static void pwc_test_silence_crash_reporting(void) {
#ifdef _WIN32
    (void) _set_abort_behavior(0u, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
    (void) SetErrorMode(SEM_NOGPFAULTERRORBOX | SEM_FAILCRITICALERRORS);
#endif
}

int main(void) {
    pwc_lockdep state;
    pwc_test_silence_crash_reporting();
    pwc_lockdep_init(&state);

    /* Ascending order is legal and must succeed. Asserted rather than assumed:
     * if these were rejected, the inversion below would be reached for the
     * wrong reason and the test would pass without testing anything. */
    if (pwc_lockdep_acquire(&state, 20u) != PWC_OK) {
        (void) fprintf(stderr, "SETUP FAILED: ascending acquire of level 20 was rejected\n");
        return 2;
    }
    if (pwc_lockdep_acquire(&state, 30u) != PWC_OK) {
        (void) fprintf(stderr, "SETUP FAILED: ascending acquire of level 30 was rejected\n");
        return 2;
    }

    /* Level 10 taken while 30 is held: a strict inversion, and the shape a real
     * deadlock is built from. This call must not return. */
    (void) pwc_lockdep_acquire(&state, 10u);

    (void) fprintf(stderr, "NOT REACHED: lockdep returned instead of aborting\n");
    return 0;
}
