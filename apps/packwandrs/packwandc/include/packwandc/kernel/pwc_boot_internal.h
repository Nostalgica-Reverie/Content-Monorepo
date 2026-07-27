#ifndef PACKWANDC_KERNEL_PWC_BOOT_INTERNAL_H
#define PACKWANDC_KERNEL_PWC_BOOT_INTERNAL_H
#include "packwandc/kernel/pwc_handle_table.h"
#include "packwandc/kernel/pwc_ipc.h"
#include "packwandc/kernel/pwc_ktrace.h"
#include "packwandc/kernel/pwc_sched.h"

pwc_handle_table *pwc_kernel_handles(void);

/* The boot-owned trace ring, or NULL before pwc_boot. Callers on failure paths
 * must tolerate NULL rather than requiring a booted kernel: argument validation
 * can and does fail before boot, and losing that trace is preferable to a
 * null dereference inside the error path itself. */
pwc_ktrace *pwc_kernel_ktrace(void);

/* The boot-owned port table, or NULL before pwc_boot. */
pwc_ipc_table *pwc_kernel_ipc(void);

/* The boot-owned worker pool, or NULL before pwc_boot. */
pwc_sched *pwc_kernel_sched(void);
#endif
