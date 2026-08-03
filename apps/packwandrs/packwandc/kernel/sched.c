/* Worker pool and platform wait pollers. Shutdown broadcasts before joining. */

#include "packwandc/kernel/pwc_error.h"
#include "packwandc/kernel/pwc_sched.h"

#include <string.h>

enum { PWC_SCHED_QUEUE_MASK = PWC_SCHED_QUEUE_CAPACITY - 1 };

static_assert((PWC_SCHED_QUEUE_CAPACITY & (PWC_SCHED_QUEUE_CAPACITY - 1)) == 0,
              "PWC_SCHED_QUEUE_CAPACITY must be a power of two");

/* Pop one item, or report that there is nothing to take. Caller holds the lock. */
static bool pwc_sched_take(pwc_sched *sched, pwc_work *out) {
    if (sched->queued == 0u) {
        return false;
    }
    *out = sched->queue[sched->head & (uint32_t) PWC_SCHED_QUEUE_MASK];
    ++sched->head;
    --sched->queued;
    return true;
}

static void pwc_sched_worker(void *arg) {
    pwc_sched *const sched = (pwc_sched *) arg;

    for (;;) {
        pwc_arch_mutex_lock(&sched->lock);
        /* Predicate in a loop, because a condition variable may wake
         * spuriously and because a broadcast wakes every worker for what may
         * be a single item. */
        while (sched->running && sched->queued == 0u) {
            pwc_arch_cond_wait(&sched->wake, &sched->lock);
        }

        pwc_work work = {0};
        const bool have_work = pwc_sched_take(sched, &work);
        const bool stopping = !sched->running;
        pwc_arch_mutex_unlock(&sched->lock);

        if (have_work) {
            /* Run outside the lock. Running under it would serialise the pool
             * down to one worker and let any callback that submits more work
             * deadlock on a mutex it already holds. */
            work.fn(work.arg);
            continue;
        }
        if (stopping) {
            /* Queue drained and shutting down. Work already queued is run
             * first, on purpose: dropping it silently would make shutdown lose
             * whatever was in flight. */
            return;
        }
    }
}

pwc_status pwc_sched_init(pwc_sched *sched, uint32_t worker_count) {
    if (sched == nullptr || worker_count == 0u || worker_count > (uint32_t) PWC_SCHED_MAX_WORKERS) {
        return PWC_FAIL(PWC_EINVAL, "core", "pwc_sched_init: zero workers or above the maximum");
    }

    memset(sched, 0, sizeof(*sched));
    PWC_TRY(pwc_arch_mutex_init(&sched->lock));
    const pwc_status cond = pwc_arch_cond_init(&sched->wake);
    if (cond != PWC_OK) {
        pwc_arch_mutex_destroy(&sched->lock);
        return cond;
    }

    sched->running = true;
    sched->started = true;

    for (uint32_t i = 0u; i < worker_count; ++i) {
        uintptr_t handle = 0u;
        const pwc_status started = pwc_arch_thread_start(pwc_sched_worker, sched, &handle);
        if (started != PWC_OK) {
            /* Unwind rather than run degraded. A pool that silently has fewer
             * workers than asked for is a latency bug nobody can find later. */
            pwc_sched_shutdown(sched);
            return started;
        }
        sched->workers[i] = handle;
        sched->worker_count = i + 1u;
    }

    return PWC_OK;
}

pwc_status pwc_sched_submit(pwc_sched *sched, pwc_work_fn fn, void *arg) {
    if (sched == nullptr || fn == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "core", "pwc_sched_submit: null scheduler or function");
    }
    if (!sched->started) {
        return PWC_FAIL(PWC_ECANCELED, "core", "pwc_sched_submit: the scheduler is not running");
    }

    pwc_arch_mutex_lock(&sched->lock);
    if (!sched->running) {
        pwc_arch_mutex_unlock(&sched->lock);
        return PWC_FAIL(PWC_ECANCELED, "core", "pwc_sched_submit: the scheduler is shutting down");
    }
    if (sched->queued >= (uint32_t) PWC_SCHED_QUEUE_CAPACITY) {
        pwc_arch_mutex_unlock(&sched->lock);
        return PWC_FAIL_PLATFORM(PWC_EOVERFLOW, "core", "the work queue is full", PWC_SCHED_QUEUE_CAPACITY);
    }

    sched->queue[sched->tail & (uint32_t) PWC_SCHED_QUEUE_MASK] = (pwc_work){.fn = fn, .arg = arg};
    ++sched->tail;
    ++sched->queued;
    pwc_arch_mutex_unlock(&sched->lock);

    /* Signalled after the unlock. Waking a worker while still holding the lock
     * makes it wake straight into contention for a mutex we have not released. */
    pwc_arch_cond_signal(&sched->wake);
    return PWC_OK;
}

pwc_status pwc_sched_spawn_poller(pwc_sched *sched, pwc_work_fn fn, void *arg) {
    if (sched == nullptr || fn == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "core", "pwc_sched_spawn_poller: null scheduler or function");
    }
    if (!sched->started) {
        return PWC_FAIL(PWC_ECANCELED, "core", "pwc_sched_spawn_poller: the scheduler is not running");
    }

    pwc_arch_mutex_lock(&sched->lock);
    if (!sched->running || sched->poller_count >= (uint32_t) PWC_SCHED_MAX_POLLERS) {
        const bool stopping = !sched->running;
        pwc_arch_mutex_unlock(&sched->lock);
        return stopping
                   ? PWC_FAIL(PWC_ECANCELED, "core", "the scheduler is shutting down")
                   : PWC_FAIL_PLATFORM(PWC_ENOMEM, "core", "no free poller slots", PWC_SCHED_MAX_POLLERS);
    }
    const uint32_t index = sched->poller_count;
    /* Keep the lock through start and handle publication. Otherwise shutdown
     * can observe a reserved slot containing zero, skip its join, destroy the
     * scheduler, and let the just-created poller run against dead state. */
    ++sched->poller_count;
    uintptr_t handle = 0u;
    const pwc_status started = pwc_arch_thread_start(fn, arg, &handle);
    if (started != PWC_OK) {
        sched->poller_count = index;
        pwc_arch_mutex_unlock(&sched->lock);
        return started;
    }
    sched->pollers[index] = handle;
    pwc_arch_mutex_unlock(&sched->lock);
    return PWC_OK;
}

uint32_t pwc_sched_pending(pwc_sched *sched) {
    if (sched == nullptr || !sched->started) {
        return 0u;
    }
    pwc_arch_mutex_lock(&sched->lock);
    const uint32_t queued = sched->queued;
    pwc_arch_mutex_unlock(&sched->lock);
    return queued;
}

void pwc_sched_shutdown(pwc_sched *sched) {
    if (sched == nullptr || !sched->started) {
        return;
    }

    pwc_arch_mutex_lock(&sched->lock);
    sched->running = false;
    pwc_arch_mutex_unlock(&sched->lock);

    /* Broadcast, not signal: signal wakes one worker, which exits, leaving the
     * rest asleep forever and the join below hanging. */
    pwc_arch_cond_broadcast(&sched->wake);

    /* Joins happen outside the lock. Holding it here deadlocks immediately --
     * the worker being joined needs the same lock to see `running` go false. */
    for (uint32_t i = 0u; i < sched->worker_count; ++i) {
        if (sched->workers[i] != 0u) {
            (void) pwc_arch_thread_join(sched->workers[i]);
            sched->workers[i] = 0u;
        }
    }
    for (uint32_t i = 0u; i < sched->poller_count; ++i) {
        if (sched->pollers[i] != 0u) {
            (void) pwc_arch_thread_join(sched->pollers[i]);
            sched->pollers[i] = 0u;
        }
    }

    sched->worker_count = 0u;
    sched->poller_count = 0u;
    sched->started = false;
    pwc_arch_cond_destroy(&sched->wake);
    pwc_arch_mutex_destroy(&sched->lock);
}
