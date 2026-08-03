/* Threading backend: none (the arch/common fallback).
 *
 * Linked only on a platform with no backend of its own -- today that means
 * macOS is not supported by this backend yet; keep the fallback explicit.
 * run it. Every entry point reports PWC_ENOSYS.
 *
 * The consequence is deliberate and worth stating: pwc_sched_init fails here,
 * so pwc_boot fails, so the whole native core refuses to come up rather than
 * running without a scheduler. A kernel that silently has no threads would
 * hand every caller a pool that accepts work and never runs it, which is a far
 * worse failure than not starting.
 */

#include "packwandc/kernel/pwc_arch_thread.h"
#include "packwandc/kernel/pwc_error.h"

pwc_status pwc_arch_thread_start(pwc_arch_thread_fn entry, void *arg, uintptr_t *out_handle) {
    (void) entry;
    (void) arg;
    (void) out_handle;
    return PWC_FAIL(PWC_ENOSYS, "arch/common", "no threading backend on this platform");
}

pwc_status pwc_arch_thread_join(uintptr_t handle) {
    (void) handle;
    return PWC_FAIL(PWC_ENOSYS, "arch/common", "no threading backend on this platform");
}

pwc_status pwc_arch_mutex_init(pwc_arch_mutex *mutex) {
    (void) mutex;
    return PWC_FAIL(PWC_ENOSYS, "arch/common", "no threading backend on this platform");
}

void pwc_arch_mutex_destroy(pwc_arch_mutex *mutex) { (void) mutex; }
void pwc_arch_mutex_lock(pwc_arch_mutex *mutex) { (void) mutex; }
void pwc_arch_mutex_unlock(pwc_arch_mutex *mutex) { (void) mutex; }

pwc_status pwc_arch_cond_init(pwc_arch_cond *cond) {
    (void) cond;
    return PWC_FAIL(PWC_ENOSYS, "arch/common", "no threading backend on this platform");
}

void pwc_arch_cond_destroy(pwc_arch_cond *cond) { (void) cond; }
void pwc_arch_cond_wait(pwc_arch_cond *cond, pwc_arch_mutex *mutex) {
    (void) cond;
    (void) mutex;
}
void pwc_arch_cond_signal(pwc_arch_cond *cond) { (void) cond; }
void pwc_arch_cond_broadcast(pwc_arch_cond *cond) { (void) cond; }
