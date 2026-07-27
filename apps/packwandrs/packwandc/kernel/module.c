#include "packwandc/kernel/pwc_module.h"
#include "packwandc/kernel/pwc_module_registry.h"
#include <string.h>

/* The registry. Deliberately not in dependency order -- pwc_modules_init
 * resolves that from each descriptor's `depends`, and listing them
 * pre-sorted here would mask a missing dependency edge by accident. */
static const pwc_module *const pwc_modules[] = {
    &pwc_module_pwproc,
    &pwc_module_pwkeys,
    &pwc_module_pwfs,
    &pwc_module_pwsh,
};

const pwc_module *const *pwc_module_registry(size_t *out_count) {
    if (out_count != nullptr) {
        *out_count = sizeof(pwc_modules) / sizeof(pwc_modules[0]);
    }
    return pwc_modules;
}

static bool pwc_module_named(const pwc_module *module, const char *name) {
    return module != nullptr && module->name != nullptr && strcmp(module->name, name) == 0;
}
static bool pwc_dependency_ready(const pwc_module_set *set, const char *name) {
    for (size_t index = 0u; index < set->count; ++index) {
        if (pwc_module_named(set->ordered[index], name)) {
            return true;
        }
    }
    return false;
}
static bool pwc_module_already_added(const pwc_module_set *set, const pwc_module *module) {
    for (size_t index = 0u; index < set->count; ++index) {
        if (set->ordered[index] == module) {
            return true;
        }
    }
    return false;
}
static bool pwc_dependencies_ready(const pwc_module_set *set, const pwc_module *module) {
    if (module->depends == nullptr) {
        return true;
    }
    for (size_t index = 0u; module->depends[index] != nullptr; ++index) {
        if (!pwc_dependency_ready(set, module->depends[index])) {
            return false;
        }
    }
    return true;
}
void pwc_modules_shutdown(pwc_module_set *set) {
    if (set == nullptr) {
        return;
    }
    while (set->count > 0u) {
        --set->count;
        const pwc_module *module = set->ordered[set->count];
        if (module->exit != nullptr) {
            module->exit(&set->contexts[set->count]);
        }
    }
}
pwc_status pwc_modules_init(pwc_module_set *set, const pwc_module *const *modules, size_t count) {
    if (set == nullptr || modules == nullptr || count > PWC_MODULE_MAX) {
        return PWC_EINVAL;
    }
    *set = (pwc_module_set){0};
    while (set->count < count) {
        bool progress = false;
        for (size_t index = 0u; index < count; ++index) {
            const pwc_module *module = modules[index];
            if (module == nullptr || module->name == nullptr || module->init == nullptr) {
                pwc_modules_shutdown(set);
                return PWC_EINVAL;
            }
            if (!pwc_module_already_added(set, module) && pwc_dependencies_ready(set, module)) {
                const pwc_status status = module->init(&set->contexts[set->count]);
                if (status != PWC_OK) {
                    pwc_modules_shutdown(set);
                    return status;
                }
                set->ordered[set->count] = module;
                ++set->count;
                progress = true;
            }
        }
        if (!progress) {
            pwc_modules_shutdown(set);
            return PWC_EINVAL;
        }
    }
    return PWC_OK;
}
