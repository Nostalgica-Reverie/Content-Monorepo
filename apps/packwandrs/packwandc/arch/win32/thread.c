/* Threading backend: Win32 (packwandc.md 3.6).
 *
 * SRWLOCK and CONDITION_VARIABLE rather than CRITICAL_SECTION: they need no
 * destroy call, they are pointer-sized, and they pair directly with
 * SleepConditionVariableSRW. The lock is used exclusively (never shared), so
 * the acquire/release pair below is always the exclusive one.
 */

#include "packwandc/kernel/pwc_arch_thread.h"
#include "packwandc/kernel/pwc_error.h"

#include <windows.h>

static_assert(sizeof(SRWLOCK) <= PWC_ARCH_MUTEX_STORAGE, "SRWLOCK does not fit the opaque storage");
static_assert(sizeof(CONDITION_VARIABLE) <= PWC_ARCH_COND_STORAGE,
              "CONDITION_VARIABLE does not fit the opaque storage");

/* Cast through void* so -Wcast-align sees the alignment guarantee on the
 * struct rather than the byte array's alignment of 1. */
static SRWLOCK *pwc_srw(pwc_arch_mutex *mutex) { return (SRWLOCK *) (void *) mutex->opaque; }

static CONDITION_VARIABLE *pwc_cv(pwc_arch_cond *cond) {
    return (CONDITION_VARIABLE *) (void *) cond->opaque;
}

/* The trampoline exists because Win32 wants an unsigned-returning stdcall
 * entry and the kernel's thread functions return nothing. */
typedef struct pwc_thread_start {
    pwc_arch_thread_fn entry;
    void *arg;
} pwc_thread_start;

/* One record per live thread. A heap allocation would be the obvious way to
 * pass the entry point through CreateThread, but packwandc.md 3.4 forbids one
 * outside the allocators -- so the records live in a fixed table sized to the
 * same ceiling the scheduler uses, and a start fails when the table is full
 * rather than allocating. */
enum { PWC_THREAD_MAX = 128 };

static struct {
    pwc_thread_start starts[PWC_THREAD_MAX];
    /* Guards `starts` and `used`. Initialised on first use via the one-time
     * init below, because there is no module init hook this early. */
    SRWLOCK lock;
    bool used[PWC_THREAD_MAX];
} pwc_threads;

static DWORD WINAPI pwc_thread_trampoline(LPVOID parameter) {
    pwc_thread_start *const slot = (pwc_thread_start *) parameter;
    const pwc_arch_thread_fn entry = slot->entry;
    void *const arg = slot->arg;

    /* The slot is released before the body runs: the body may outlive
     * everything else, and holding a table entry for its whole life would cap
     * concurrent threads at the table size rather than at the OS limit. */
    AcquireSRWLockExclusive(&pwc_threads.lock);
    const size_t index = (size_t) (slot - &pwc_threads.starts[0]);
    pwc_threads.used[index] = false;
    ReleaseSRWLockExclusive(&pwc_threads.lock);

    entry(arg);
    return 0u;
}

pwc_status pwc_arch_thread_start(pwc_arch_thread_fn entry, void *arg, uintptr_t *out_handle) {
    if (entry == nullptr || out_handle == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/win32", "pwc_arch_thread_start: null entry or out");
    }

    AcquireSRWLockExclusive(&pwc_threads.lock);
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
    ReleaseSRWLockExclusive(&pwc_threads.lock);

    if (index == (size_t) PWC_THREAD_MAX) {
        return PWC_FAIL_PLATFORM(PWC_ENOMEM, "arch/win32", "no free thread start slots", PWC_THREAD_MAX);
    }

    HANDLE thread = CreateThread(nullptr, 0u, pwc_thread_trampoline, &pwc_threads.starts[index], 0u, nullptr);
    if (thread == nullptr) {
        const DWORD code = GetLastError();
        AcquireSRWLockExclusive(&pwc_threads.lock);
        pwc_threads.used[index] = false;
        ReleaseSRWLockExclusive(&pwc_threads.lock);
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "CreateThread failed", code);
    }

    *out_handle = (uintptr_t) thread;
    return PWC_OK;
}

pwc_status pwc_arch_thread_join(uintptr_t handle) {
    if (handle == 0u) {
        return PWC_FAIL(PWC_EINVAL, "arch/win32", "pwc_arch_thread_join: null handle");
    }
    HANDLE thread = (HANDLE) handle;
    if (WaitForSingleObject(thread, INFINITE) == WAIT_FAILED) {
        const DWORD code = GetLastError();
        (void) CloseHandle(thread);
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "WaitForSingleObject on a thread failed", code);
    }
    if (CloseHandle(thread) == 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "CloseHandle on a thread failed", GetLastError());
    }
    return PWC_OK;
}

pwc_status pwc_arch_mutex_init(pwc_arch_mutex *mutex) {
    if (mutex == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/win32", "pwc_arch_mutex_init: null mutex");
    }
    InitializeSRWLock(pwc_srw(mutex));
    return PWC_OK;
}

/* SRWLOCK has no destructor; the entry point exists so callers are symmetric
 * across platforms, where pthreads does need one. */
void pwc_arch_mutex_destroy(pwc_arch_mutex *mutex) { (void) mutex; }

void pwc_arch_mutex_lock(pwc_arch_mutex *mutex) { AcquireSRWLockExclusive(pwc_srw(mutex)); }

void pwc_arch_mutex_unlock(pwc_arch_mutex *mutex) { ReleaseSRWLockExclusive(pwc_srw(mutex)); }

pwc_status pwc_arch_cond_init(pwc_arch_cond *cond) {
    if (cond == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/win32", "pwc_arch_cond_init: null cond");
    }
    InitializeConditionVariable(pwc_cv(cond));
    return PWC_OK;
}

void pwc_arch_cond_destroy(pwc_arch_cond *cond) { (void) cond; }

void pwc_arch_cond_wait(pwc_arch_cond *cond, pwc_arch_mutex *mutex) {
    /* A false return is a timeout or a spurious wake, both of which the
     * caller's predicate loop already handles -- there is nothing to report. */
    (void) SleepConditionVariableSRW(pwc_cv(cond), pwc_srw(mutex), INFINITE, 0u);
}

void pwc_arch_cond_signal(pwc_arch_cond *cond) { WakeConditionVariable(pwc_cv(cond)); }

void pwc_arch_cond_broadcast(pwc_arch_cond *cond) { WakeAllConditionVariable(pwc_cv(cond)); }
