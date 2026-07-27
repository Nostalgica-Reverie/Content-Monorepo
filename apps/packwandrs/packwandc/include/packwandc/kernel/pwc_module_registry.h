/* The static module registry (packwandc.md 3.5).
 *
 * WHY AN EXPLICIT TABLE AND NOT A LINK-TIME SECTION
 *
 * packwandc.md 3.5 sketches a PWC_MODULE_REGISTER macro that drops each
 * descriptor into a link-time section, the way Linux collects initcalls. That
 * is not used here, and the reason is portability rather than taste: the ELF
 * spelling is __attribute__((section)) plus linker-provided __start_/__stop_
 * symbols, while the MSVC spelling needs #pragma section and
 * __declspec(allocate) with a named segment and lexicographic ordering
 * convention. packwandc builds for both, the two mechanisms share no syntax,
 * and scripts/gate-banned.sh bans #pragma other than `once`.
 *
 * An explicit array costs one line per module and is still *static*
 * registration in the sense that matters (packwandc.md 3.5): no dlopen, no
 * runtime loading, no plugin ABI to defend. The only thing lost is that adding
 * a module means editing this list, which a reviewer sees rather than misses.
 */
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
