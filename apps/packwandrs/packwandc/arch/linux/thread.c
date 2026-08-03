/* Threading backend: pthreads.
 *
 * The mirror of arch/win32/thread.c, including the fixed start-record table:
 * pthread_create takes a single void* and the kernel's entry points take a
 * different shape, so the pair has to be carried through something. A malloc
 * would be the obvious carrier and is forbidden outside the allocators
 * so the records live in a fixed table and a start fails
 * when it is full rather than allocating.
 */

#define _GNU_SOURCE 1

#include "packwandc/kernel/pwc_arch_thread.h"
#include "packwandc/kernel/pwc_error.h"

#include <errno.h>
#include <pthread.h>
#include <stdlib.h>

static_assert(sizeof(pthread_mutex_t) <= PWC_ARCH_MUTEX_STORAGE,
              "pthread_mutex_t does not fit the opaque storage");
static_assert(sizeof(pthread_cond_t) <= PWC_ARCH_COND_STORAGE,
              "pthread_cond_t does not fit the opaque storage");

/* Cast through void* so -Wcast-align sees the struct's alignment guarantee
 * rather than the byte array's alignment of 1. */
static pthread_mutex_t *pwc_pmutex(pwc_arch_mutex *mutex) {
    return (pthread_mutex_t *) (void *) mutex->opaque;
}

static pthread_cond_t *pwc_pcond(pwc_arch_cond *cond) { return (pthread_cond_t *) (void *) cond->opaque; }

typedef struct pwc_thread_start {
    pwc_arch_thread_fn entry;
    void *arg;
} pwc_thread_start;

enum { PWC_THREAD_MAX = 128 };

static struct {
    pwc_thread_start starts[PWC_THREAD_MAX];
    bool used[PWC_THREAD_MAX];
    pthread_mutex_t lock;
} pwc_threads = {.lock = PTHREAD_MUTEX_INITIALIZER};

static void *pwc_thread_trampoline(void *parameter) {
    pwc_thread_start *const slot = (pwc_thread_start *) parameter;
    const pwc_arch_thread_fn entry = slot->entry;
    void *const arg = slot->arg;

    /* Released before the body runs: a long-lived poller holding its table
     * entry would cap concurrent threads at the table size rather than at the
     * OS limit. */
    (void) pthread_mutex_lock(&pwc_threads.lock);
    const size_t index = (size_t) (slot - &pwc_threads.starts[0]);
    pwc_threads.used[index] = false;
    (void) pthread_mutex_unlock(&pwc_threads.lock);

    entry(arg);
    return nullptr;
}

pwc_status pwc_arch_thread_start(pwc_arch_thread_fn entry, void *arg, uintptr_t *out_handle) {
    if (entry == nullptr || out_handle == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_thread_start: null entry or out");
    }

    (void) pthread_mutex_lock(&pwc_threads.lock);
    size_t index = (size_t) PWC_THREAD_MAX;
    for (size_t i = 0u; i < (size_t) PWC_THREAD_MAX; ++i) {
        if (!pwc_threads.used[i]) {
            pwc_threads.used[i] = true;
            pwc_threads.starts[i].entry = entry;
            pwc_threads.starts[i].arg = arg;
            index = i;
            break;
        }
    }
    (void) pthread_mutex_unlock(&pwc_threads.lock);

    if (index == (size_t) PWC_THREAD_MAX) {
        return PWC_FAIL_PLATFORM(PWC_ENOMEM, "arch/linux", "no free thread start slots", PWC_THREAD_MAX);
    }

    pthread_t thread;
    const int created = pthread_create(&thread, nullptr, pwc_thread_trampoline, &pwc_threads.starts[index]);
    if (created != 0) {
        (void) pthread_mutex_lock(&pwc_threads.lock);
        pwc_threads.used[index] = false;
        (void) pthread_mutex_unlock(&pwc_threads.lock);
        /* pthread_create reports through its return value, not errno. */
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "pthread_create failed", created);
    }

    /* pthread_t is opaque and not required to be an integer, but it is an
     * unsigned long on glibc and the handle only ever travels back to
     * pwc_arch_thread_join on the same platform. Asserted rather than assumed. */
    static_assert(sizeof(pthread_t) <= sizeof(uintptr_t), "pthread_t must fit in the opaque join handle");
    uintptr_t handle = 0u;
    __builtin_memcpy(&handle, &thread, sizeof(thread));
    if (handle == 0u) {
        /* Zero is the "no thread" sentinel in the shared interface. A real
         * pthread_t of zero would be indistinguishable from it, so refuse
         * rather than hand back an unjoinable token. */
        return PWC_FAIL(PWC_EIO, "arch/linux", "pthread_t collided with the null handle sentinel");
    }
    *out_handle = handle;
    return PWC_OK;
}

pwc_status pwc_arch_thread_join(uintptr_t handle) {
    if (handle == 0u) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_thread_join: null handle");
    }
    pthread_t thread;
    __builtin_memcpy(&thread, &handle, sizeof(thread));
    const int joined = pthread_join(thread, nullptr);
    if (joined != 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "pthread_join failed", joined);
    }
    return PWC_OK;
}

pwc_status pwc_arch_mutex_init(pwc_arch_mutex *mutex) {
    if (mutex == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_mutex_init: null mutex");
    }
    const int result = pthread_mutex_init(pwc_pmutex(mutex), nullptr);
    if (result != 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "pthread_mutex_init failed", result);
    }
    return PWC_OK;
}

void pwc_arch_mutex_destroy(pwc_arch_mutex *mutex) {
    if (mutex != nullptr) {
        (void) pthread_mutex_destroy(pwc_pmutex(mutex));
    }
}

/* Lock and unlock cannot report failure through the shared interface, and that
 * is deliberate: the only documented failures are caller bugs (an uninitialised
 * or already-held non-recursive mutex), and a lock that "might not have locked"
 * is not something a caller can meaningfully handle. */
void pwc_arch_mutex_lock(pwc_arch_mutex *mutex) { (void) pthread_mutex_lock(pwc_pmutex(mutex)); }

void pwc_arch_mutex_unlock(pwc_arch_mutex *mutex) { (void) pthread_mutex_unlock(pwc_pmutex(mutex)); }

pwc_status pwc_arch_cond_init(pwc_arch_cond *cond) {
    if (cond == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/linux", "pwc_arch_cond_init: null cond");
    }
    const int result = pthread_cond_init(pwc_pcond(cond), nullptr);
    if (result != 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/linux", "pthread_cond_init failed", result);
    }
    return PWC_OK;
}

void pwc_arch_cond_destroy(pwc_arch_cond *cond) {
    if (cond != nullptr) {
        (void) pthread_cond_destroy(pwc_pcond(cond));
    }
}

void pwc_arch_cond_wait(pwc_arch_cond *cond, pwc_arch_mutex *mutex) {
    (void) pthread_cond_wait(pwc_pcond(cond), pwc_pmutex(mutex));
}

void pwc_arch_cond_signal(pwc_arch_cond *cond) { (void) pthread_cond_signal(pwc_pcond(cond)); }

void pwc_arch_cond_broadcast(pwc_arch_cond *cond) { (void) pthread_cond_broadcast(pwc_pcond(cond)); }
