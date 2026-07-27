#ifndef PACKWANDC_KERNEL_PWC_MODULE_H
#define PACKWANDC_KERNEL_PWC_MODULE_H
#include "packwandc/uapi/pwc_status.h"
enum { PWC_MODULE_MAX = 32 };
typedef struct pwc_module_ctx {
    void *state;
} pwc_module_ctx;
typedef struct pwc_module {
    const char *name;
    uint32_t abi_version;
    const char *const *depends;
    pwc_status (*init)(pwc_module_ctx *ctx);
    void (*exit)(pwc_module_ctx *ctx);
} pwc_module;
typedef struct pwc_module_set {
    const pwc_module *ordered[PWC_MODULE_MAX];
    pwc_module_ctx contexts[PWC_MODULE_MAX];
    size_t count;
} pwc_module_set;
pwc_status pwc_modules_init(pwc_module_set *set, const pwc_module *const *modules, size_t count);
void pwc_modules_shutdown(pwc_module_set *set);
#endif
