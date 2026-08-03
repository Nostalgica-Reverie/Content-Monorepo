/* Static module registry shared by the native core. */
#ifndef PACKWANDC_KERNEL_PWC_MODULE_REGISTRY_H
#define PACKWANDC_KERNEL_PWC_MODULE_REGISTRY_H

#include "packwandc/kernel/pwc_module.h"

/* One descriptor per subsystem, defined beside the module it describes. */
extern const pwc_module pwc_module_pwfs;
extern const pwc_module pwc_module_pwproc;
extern const pwc_module pwc_module_pwkeys;
extern const pwc_module pwc_module_pwsh;

/* Every module the kernel brings up, in declaration order. Dependency order is
 * resolved by pwc_modules_init from each descriptor's `depends`, so this array
 * does not have to be topologically sorted -- and deliberately is not, so that
 * a missing dependency edge fails loudly instead of being masked by the order
 * someone happened to type. */
const pwc_module *const *pwc_module_registry(size_t *out_count);

#endif /* PACKWANDC_KERNEL_PWC_MODULE_REGISTRY_H */
