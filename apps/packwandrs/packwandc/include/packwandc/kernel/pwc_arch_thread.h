/* Threading primitives, per platform.
 *
 * WHY THIS EXISTS AT ALL
 *
 * C11 gives us <threads.h>, which would make this file unnecessary. It is not
 * available: neither the Universal CRT nor clang's own headers ship one for
 * x86_64-pc-windows-msvc, verified by compile probe rather than assumed. So
 * threading joins the rest of the OS surface in arch/.
 *
 * OPAQUE STORAGE, NOT POINTERS
 *
 * A mutex is a fixed-size byte buffer that each backend reinterprets as its
 * native type, rather than a pointer to something allocated. That is forced by
 * There is no allocator outside kernel/arena.c and
 * kernel/slab.c, so a lock has to be able to live inside whatever struct owns
 * it. Each backend static_asserts that its native type actually fits.
 *
 * The sizes are generous on purpose. pthread_mutex_t is 40 bytes on x86_64
 * glibc and SRWLOCK is 8; 64 covers both with room for a platform that is
 * larger, and the cost is bytes in a handful of long-lived structs.
 */
#ifndef PACKWANDC_KERNEL_PWC_ARCH_THREAD_H
#define PACKWANDC_KERNEL_PWC_ARCH_THREAD_H

#include "packwandc/uapi/pwc_status.h"

enum {
    PWC_ARCH_MUTEX_STORAGE = 64,
    PWC_ARCH_COND_STORAGE = 64,
};

/* Aligned to 16 so a backend can cast the buffer to any native type without
 * tripping -Wcast-align or landing on an unaligned atomic. */
typedef struct pwc_arch_mutex {
    _Alignas(16) unsigned char opaque[PWC_ARCH_MUTEX_STORAGE];
} pwc_arch_mutex;

typedef struct pwc_arch_cond {
    _Alignas(16) unsigned char opaque[PWC_ARCH_COND_STORAGE];
} pwc_arch_cond;

/* A thread entry point. Returns nothing: a worker reports through the state it
 * was handed, and a return value nobody can read is a place for errors to
 * vanish. */
typedef void (*pwc_arch_thread_fn)(void *arg);

/* Start a thread. `out_handle` receives an opaque, non-zero join token. */
pwc_status pwc_arch_thread_start(pwc_arch_thread_fn entry, void *arg, uintptr_t *out_handle);

/* Wait for a thread to finish and release its handle. Must be called exactly
 * once per successful start, or the OS thread object leaks. */
pwc_status pwc_arch_thread_join(uintptr_t handle);

pwc_status pwc_arch_mutex_init(pwc_arch_mutex *mutex);
void pwc_arch_mutex_destroy(pwc_arch_mutex *mutex);
void pwc_arch_mutex_lock(pwc_arch_mutex *mutex);
void pwc_arch_mutex_unlock(pwc_arch_mutex *mutex);

pwc_status pwc_arch_cond_init(pwc_arch_cond *cond);
void pwc_arch_cond_destroy(pwc_arch_cond *cond);

/* Atomically release `mutex` and sleep until signalled; reacquires on return.
 *
 * Spurious wakeups are permitted and do happen, so every caller must re-test
 * its predicate in a loop rather than assuming a wake means the condition
 * holds. */
void pwc_arch_cond_wait(pwc_arch_cond *cond, pwc_arch_mutex *mutex);

/* Wake one waiter / every waiter. Broadcast is what shutdown uses: signalling
 * once would leave every worker but one asleep forever. */
void pwc_arch_cond_signal(pwc_arch_cond *cond);
void pwc_arch_cond_broadcast(pwc_arch_cond *cond);

#endif /* PACKWANDC_KERNEL_PWC_ARCH_THREAD_H */
