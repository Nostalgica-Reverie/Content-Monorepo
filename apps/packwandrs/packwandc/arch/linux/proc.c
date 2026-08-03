/* pwproc backend: pidfd + process groups.
 *
 * WHY NOT A CGROUP v2 SCOPE
 *
 * The spec names "pidfd plus a cgroup v2 scope" as the Linux answer, and a
 * scope really is the only construct that matches Windows' job-object
 * guarantee: membership is inherited, cannot be escaped by double-forking, and
 * the whole set dies together. It is not used here because creating one needs
 * either write access to a delegated cgroup subtree or a systemd
 * transient-unit call over D-Bus. Neither is available to an unprivileged
 * desktop process on a stock system, and a backend that works only under
 * systemd with delegation configured is worse than one that states its limit.
 *
 * What is used instead:
 *
 *   - a pidfd for identity. The classic teardown race is that the pid exits,
 *     the number is recycled, and the killer signals an innocent process. A
 *     pidfd names the process itself rather than the number, so the leader is
 *     always signalled correctly no matter how long the handle is held.
 *   - a process-group signal for reach. The supervisor already starts each
 *     child in its own process group (crates/packwand-launch/src/supervisor.rs),
 *     so signalling the negated pgid reaches the child and every descendant
 *     that has not deliberately left the group.
 *
 * That is the guarantee the previous `nix::sys::signal::killpg` call had, minus
 * the dependency, plus a race-free leader kill it did not have. A descendant
 * that calls setsid escapes here and would not escape a cgroup scope; that gap
 * is real, and it is why the cgroup route stays on the table rather than being
 * struck from the spec.
 */

/* Feature-test macros must precede every include or they are silently ignored
 * -- _GNU_SOURCE is what exposes syscall(). */
#define _GNU_SOURCE 1

#include "packwandc/kernel/pwc_arch_proc.h"
#include "packwandc/kernel/pwc_error.h"

#include <errno.h>
#include <signal.h>
#include <sys/syscall.h>
#include <unistd.h>

/* Both syscalls predate their glibc wrappers -- pidfd_send_signal is in Linux
 * 5.1 and pidfd_open in 5.3, but the wrappers only arrived in glibc 2.36.
 * Going through syscall() with a local fallback number keeps this building
 * against older libc without a configure probe. The numbers below are the
 * architecture-independent ones. */
#ifndef __NR_pidfd_send_signal
#define __NR_pidfd_send_signal 424
#endif
#ifndef __NR_pidfd_open
#define __NR_pidfd_open 434
#endif

/* The pid occupies the high half of the payload, the biased fd the low half.
 * Both are needed at kill time: the fd to signal the leader without a pid race,
 * the pid to signal the group.
 *
 * Kept in separate enums, and the low-half mask expressed as UINT32_MAX rather
 * than a member here, because an enum holding both -1 and 0xffffffff forces an
 * underlying type wider than int. C23 permits that, but it is a surprising
 * thing to make a reader work out from the member list. */
enum { PWC_PROC_FD_INVALID = -1 };
enum { PWC_PROC_PID_SHIFT = 32 };

static int pwc_pidfd_open(uint32_t pid) {
    const long raw = syscall(__NR_pidfd_open, (pid_t) pid, 0u);
    return raw < 0 ? PWC_PROC_FD_INVALID : (int) raw;
}

pwc_status pwc_arch_proc_adopt(uint32_t pid, uintptr_t *out_native) {
    if (pid == 0u || out_native == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_proc_adopt: zero pid or null out");
    }

    const int pidfd = pwc_pidfd_open(pid);
    if (pidfd == PWC_PROC_FD_INVALID) {
        const int code = errno;
        return code == ESRCH ? PWC_FAIL_PLATFORM(PWC_ENOENT, "arch/linux", "no such process to adopt", code)
                             : PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "pidfd_open failed", code);
    }

    /* The fd is stored biased by one so that the packed payload is never zero:
     * the handle layer treats a zero payload as "unset". */
    const uint64_t packed =
        ((uint64_t) pid << (uint64_t) PWC_PROC_PID_SHIFT) | (uint64_t) ((uint32_t) pidfd + 1u);
    *out_native = (uintptr_t) packed;
    return PWC_OK;
}

pwc_status pwc_arch_proc_kill(uintptr_t native) {
    if (native == 0u) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_proc_kill: empty process payload");
    }
    const uint64_t packed = (uint64_t) native;
    const pid_t pid = (pid_t) (packed >> (uint64_t) PWC_PROC_PID_SHIFT);
    const int pidfd = (int) ((uint32_t) (packed & (uint64_t) UINT32_MAX) - 1u);

    /* Group first, then the leader.
     *
     * The order matters. Killing the leader first lets it be reaped and its
     * pgid become reusable before the group signal lands; doing the group
     * first means every descendant is already dying when the leader goes. Both
     * signals are best-effort against an already-dead target -- ESRCH is the
     * desired end state here, not a failure. */
    int failure = 0;
    if (pid > 0 && kill(-pid, SIGKILL) != 0 && errno != ESRCH) {
        failure = errno;
    }

    const long sent = syscall(__NR_pidfd_send_signal, pidfd, SIGKILL, nullptr, 0u);
    if (sent < 0 && errno != ESRCH && failure == 0) {
        failure = errno;
    }

    /* The fd is closed whatever happened above: leaking it would leak the
     * process's exit-notification slot for the life of the host. */
    const int closed = close(pidfd);
    if (failure != 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "SIGKILL to the process tree failed", failure);
    }
    if (closed != 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "close on the pidfd failed", errno);
    }
    return PWC_OK;
}

pwc_status pwc_arch_proc_exists(uint32_t pid, uint32_t *out_alive) {
    if (pid == 0u || out_alive == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_proc_exists: zero pid or null out");
    }

    /* Signal 0 runs the existence and permission checks without delivering
     * anything -- the standard liveness probe. */
    if (kill((pid_t) pid, 0) == 0) {
        *out_alive = 1u;
        return PWC_OK;
    }

    const int code = errno;
    if (code == ESRCH) {
        *out_alive = 0u;
        return PWC_OK;
    }
    if (code == EPERM) {
        /* Alive, merely not signallable by this uid. Reporting it dead would
         * let the tree-kill test pass against a surviving tree, which is the
         * exact failure pwproc exists to make impossible. */
        *out_alive = 1u;
        return PWC_OK;
    }
    return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "kill(pid, 0) failed for an indeterminate reason", code);
}
