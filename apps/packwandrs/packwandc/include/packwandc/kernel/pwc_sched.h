/* The kernel scheduler: a fixed worker pool and dedicated pollers.
 *
 * packwandc.md 3.6: modules never create threads. Everything that wants to run
 * off the calling thread comes through here, which is what makes the process's
 * thread population knowable rather than emergent.
 *
 * TWO SHAPES OF WORK, AND WHY
 *
 *   - **Queued work** runs on the fixed pool. For short tasks. N threads can
 *     absorb any number of these, because each one finishes.
 *   - **Dedicated pollers** get a thread each, for the life of the kernel. For
 *     blocking platform waits -- the case packwandc.md 3.6 calls out.
 *
 * The split is not a convenience. A blocking wait submitted to the pool
 * occupies a worker until it returns, which for a filesystem watch is "never";
 * submit as many as there are workers and the pool is dead with no diagnostic
 * anywhere. Making the long-running case a separate call means that mistake is
 * not available to make.
 */
#ifndef PACKWANDC_KERNEL_PWC_SCHED_H
#define PACKWANDC_KERNEL_PWC_SCHED_H

#include "packwandc/kernel/pwc_arch_thread.h"
#include "packwandc/uapi/pwc_status.h"

enum {
    PWC_SCHED_MAX_WORKERS = 64,
    /* Long-running poller threads, over and above the pool. One per open
     * filesystem watch today. */
    PWC_SCHED_MAX_POLLERS = 32,
    /* Items awaiting a worker. Bounded because there is no allocator: a full
     * queue reports back-pressure rather than growing. */
    PWC_SCHED_QUEUE_CAPACITY = 256,
    /* Used when a boot config asks for zero. Small on purpose: the pool is for
     * short tasks, and blocking waits get dedicated threads instead. */
    PWC_SCHED_DEFAULT_WORKERS = 4,
};

typedef void (*pwc_work_fn)(void *arg);

typedef struct pwc_work {
    pwc_work_fn fn;
    void *arg;
} pwc_work;

typedef struct pwc_sched {
    pwc_arch_mutex lock;
    /* Workers sleep here while the queue is empty. */
    pwc_arch_cond wake;

    pwc_work queue[PWC_SCHED_QUEUE_CAPACITY];
    uint32_t head;
    uint32_t tail;
    uint32_t queued;

    uintptr_t workers[PWC_SCHED_MAX_WORKERS];
    uint32_t worker_count;

    uintptr_t pollers[PWC_SCHED_MAX_POLLERS];
    uint32_t poller_count;

    /* Cleared by shutdown. Workers observe it under the lock and drain out. */
    bool running;
    /* False until init succeeds, so shutdown on a scheduler that never started
     * does not touch an uninitialised mutex. */
    bool started;
} pwc_sched;

/* Start `worker_count` workers. */
pwc_status pwc_sched_init(pwc_sched *sched, uint32_t worker_count);

/* Stop accepting work, wake everything, and join every thread.
 *
 * Idempotent, and safe on a scheduler that never started. Pollers are joined
 * too, so a poller blocked in a platform wait must be made to return by closing
 * whatever it waits on *before* this is called -- see pwc_sched_spawn_poller. */
void pwc_sched_shutdown(pwc_sched *sched);

/* Queue work for the pool. PWC_EOVERFLOW when the queue is full. */
pwc_status pwc_sched_submit(pwc_sched *sched, pwc_work_fn fn, void *arg);

/* Start a dedicated thread for a blocking wait.
 *
 * The function is expected to run until the thing it waits on is closed.
 * Shutdown joins it, so it MUST be interruptible that way -- a poller that
 * ignores closure hangs shutdown forever rather than leaking. */
pwc_status pwc_sched_spawn_poller(pwc_sched *sched, pwc_work_fn fn, void *arg);

/* Items waiting for a worker. For tests and diagnostics. */
uint32_t pwc_sched_pending(pwc_sched *sched);

#endif /* PACKWANDC_KERNEL_PWC_SCHED_H */
