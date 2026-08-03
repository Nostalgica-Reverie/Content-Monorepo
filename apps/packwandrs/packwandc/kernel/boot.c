/* pwc_boot / pwc_shutdown. Initialization and teardown are reverse orders. */

#include "packwandc/kernel/pwc_boot_internal.h"
#include "packwandc/kernel/pwc_error.h"
#include "packwandc/kernel/pwc_handle_table.h"
#include "packwandc/kernel/pwc_kernel.h"
#include "packwandc/kernel/pwc_module_registry.h"
#include "packwandc/kernel/pwc_sched.h"

typedef struct pwc_kernel_instance {
    bool booted;
    pwc_handle_table handles;
    pwc_ktrace trace;
    pwc_ipc_table ipc;
    pwc_sched sched;
    pwc_module_set modules;
} pwc_kernel_instance;

/* The one non-const file-scope variable in the tree. scripts/gate-banned.sh
 * exempts this file specifically, because a kernel needs exactly one instance
 * and threading it through every call site would be ceremony, not safety. */
static pwc_kernel_instance pwc_kernel;

pwc_status pwc_boot(const pwc_boot_config *config) {
    uint32_t capacity = PWC_HANDLE_CAPACITY_MAX;
    if (config != nullptr) {
        capacity = config->handle_capacity;
    }
    if (capacity == 0u || capacity > PWC_HANDLE_CAPACITY_MAX) {
        return PWC_FAIL(PWC_EINVAL, "core", "pwc_boot: handle capacity is zero or above the maximum");
    }
    if (pwc_kernel.booted) {
        return PWC_FAIL(PWC_EAGAIN, "core", "pwc_boot: the kernel is already booted");
    }

    pwc_handle_table_init(&pwc_kernel.handles, capacity);
    pwc_ktrace_init(&pwc_kernel.trace);
    pwc_ipc_table_init(&pwc_kernel.ipc);
    pwc_kernel.booted = true;

    /* The pool comes up before modules, because a module's init may want to
     * spawn a poller. A platform with no threading backend fails here rather
     * than running with a pool that accepts work and never runs it. */
    const uint32_t workers =
        (config != nullptr && config->worker_count != 0u) ? config->worker_count : PWC_SCHED_DEFAULT_WORKERS;
    const pwc_status scheduled = pwc_sched_init(&pwc_kernel.sched, workers);
    if (scheduled != PWC_OK) {
        pwc_kernel.booted = false;
        return scheduled;
    }

    size_t count = 0u;
    const pwc_module *const *const modules = pwc_module_registry(&count);
    const pwc_status initialised = pwc_modules_init(&pwc_kernel.modules, modules, count);
    if (initialised != PWC_OK) {
        /* pwc_modules_init has already unwound whatever it brought up. Undo the
         * rest of boot too, so a failed boot leaves nothing half-live for a
         * caller to accidentally use. */
        pwc_sched_shutdown(&pwc_kernel.sched);
        pwc_kernel.booted = false;
        return initialised;
    }

    PWC_NOTE(PWC_TRACE_LEVEL_INFO, "core", "kernel booted");
    return PWC_OK;
}

void pwc_shutdown(void) {
    if (!pwc_kernel.booted) {
        return;
    }
    PWC_NOTE(PWC_TRACE_LEVEL_INFO, "core", "kernel shutting down");

    /* Modules go first, and while the kernel still reports as booted: their
     * exit hooks may close handles and emit traces, both of which need the
     * subsystems below them to still be answering. */
    pwc_modules_shutdown(&pwc_kernel.modules);

    /* After the modules: their exit hooks close the objects that pollers are
     * blocked on, and pwc_sched_shutdown joins those pollers. Reversing this
     * order hangs shutdown on a poller nothing has unblocked. */
    pwc_sched_shutdown(&pwc_kernel.sched);

    pwc_kernel.booted = false;
    pwc_handle_table_init(&pwc_kernel.handles, PWC_HANDLE_CAPACITY_MAX);
    /* Re-initialised rather than left as-is: a subsequent boot must not inherit
     * the previous run's records, its sequence numbering, or its open ports. */
    pwc_ktrace_init(&pwc_kernel.trace);
    pwc_ipc_table_init(&pwc_kernel.ipc);
}

pwc_handle_table *pwc_kernel_handles(void) { return pwc_kernel.booted ? &pwc_kernel.handles : nullptr; }

pwc_ktrace *pwc_kernel_ktrace(void) { return pwc_kernel.booted ? &pwc_kernel.trace : nullptr; }

pwc_ipc_table *pwc_kernel_ipc(void) { return pwc_kernel.booted ? &pwc_kernel.ipc : nullptr; }

pwc_sched *pwc_kernel_sched(void) { return pwc_kernel.booted ? &pwc_kernel.sched : nullptr; }
