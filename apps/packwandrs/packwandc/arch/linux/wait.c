/* Blocking primitive backing pwc_wait's timeout (packwandc.md 3.3).
 *
 * The full epoll-backed multiplexer is kernel/wait.c's job; this is the arch
 * side's single responsibility: sleep for a bounded time without burning CPU
 * and without being cut short by a signal.
 */

#define _POSIX_C_SOURCE 200809L

#include "packwandc/kernel/pwc_arch_wait.h"
#include "packwandc/kernel/pwc_error.h"

#include <errno.h>
#include <time.h>

enum {
    PWC_WAIT_MS_PER_SEC = 1000,
    PWC_WAIT_NS_PER_MS = 1000000,
};

pwc_status pwc_arch_wait_timeout(int64_t timeout_ms) {
    if (timeout_ms < -1) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_wait_timeout: timeout below -1");
    }
    if (timeout_ms <= 0) {
        /* 0 is a poll and -1 is "block until an event", which at this layer --
         * with nothing to be woken by -- must not become an infinite sleep. */
        return PWC_OK;
    }

    struct timespec remaining = {
        .tv_sec = (time_t) (timeout_ms / PWC_WAIT_MS_PER_SEC),
        .tv_nsec = (long) ((timeout_ms % PWC_WAIT_MS_PER_SEC) * PWC_WAIT_NS_PER_MS),
    };

    /* nanosleep returns EINTR with the unslept remainder, so a signal shortens
     * the wait unless the call is resumed. Looping on the remainder is what
     * makes the timeout mean what the caller asked for; a bare nanosleep here
     * would return early every time the host takes a signal. */
    while (nanosleep(&remaining, &remaining) != 0) {
        if (errno != EINTR) {
            return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "nanosleep failed", errno);
        }
    }
    return PWC_OK;
}
