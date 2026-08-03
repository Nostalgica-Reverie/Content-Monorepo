/* pwc_wait -- the epoll analogue.
 *
 * STATE OF THIS FILE
 *
 * The multiplexer the spec describes -- one call waiting across a filesystem
 * watch, a child process, an IPC port and an input device, backed by epoll on
 * Linux, IOCP on Windows and kqueue on macOS -- is NOT built. What is here is
 * handle validation, a real timeout, and an honest "nothing became ready".
 *
 * It previously reported every entry ready the instant it was asked, without
 * consulting any object and without ever sleeping. That is worse than
 * unimplemented: a caller polling in a loop is told to go and handle work that
 * has not happened, on every iteration, forever. PWC_ETIMEDOUT costs the same
 * effort and is true.
 *
 * WHAT REAL READINESS NEEDS, so the next person need not re-derive it:
 *
 *   - Linux can already answer for a watch. arch/linux/fs.c opens the inotify
 *     fd O_NONBLOCK, so reading it is a truthful readiness probe; putting epoll
 *     around that fd is the small half of the job.
 *   - Windows cannot. arch/win32/fs.c calls ReadDirectoryChangesW
 *     synchronously, so it blocks rather than reporting emptiness -- probing it
 *     would hang a worker instead of answering. It has to move to overlapped
 *     I/O against an IOCP first, which the native design requires and
 *     the reason this is not a small change.
 *   - Process exit needs the object to carry something waitable. On Windows the
 *     payload is the job handle, which is not signalled by the last process
 *     exiting, so the process handle would have to be retained beside it. On
 *     Linux the pidfd already in the payload is directly pollable.
 *
 * Until those land this returns PWC_ETIMEDOUT, and the timeout is real.
 */

#include "packwandc/kernel/pwc_arch_wait.h"
#include "packwandc/kernel/pwc_boot_internal.h"
#include "packwandc/kernel/pwc_error.h"

pwc_status pwc_wait(pwc_waitent *ents, size_t n, int64_t timeout_ms, size_t *out_ready) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    if (handles == nullptr) {
        return PWC_FAIL(PWC_ECANCELED, "core", "pwc_wait called before the kernel booted");
    }
    if (ents == nullptr || out_ready == nullptr || n == 0u || timeout_ms < -1) {
        return PWC_FAIL(PWC_EINVAL, "core", "pwc_wait: null entries/out, zero count, or timeout below -1");
    }

    *out_ready = 0u;

    /* Every handle is validated before anything blocks. A caller that passed a
     * stale or rights-deficient handle deserves to hear about it immediately
     * rather than after sitting through the whole timeout. */
    for (size_t i = 0u; i < n; ++i) {
        ents[i].revents = 0u;
        PWC_TRY(pwc_handle_validate(handles, ents[i].h, PWC_RIGHT_WAIT));
    }

    /* -1 means "block until something happens". Honouring that literally with
     * no readiness source wired up would hang the caller forever, which is a
     * worse failure than returning promptly and saying nothing was ready. */
    const int64_t bounded = timeout_ms < 0 ? 0 : timeout_ms;
    PWC_TRY(pwc_arch_wait_timeout(bounded));

    /* revents stays zero on every entry: no object was consulted, so reporting
     * anything else would be a fabrication. */
    return PWC_FAIL(PWC_ETIMEDOUT, "core", "pwc_wait has no readiness source wired up yet");
}
