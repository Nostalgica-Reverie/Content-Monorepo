#include "packwandc/kernel/pwc_arch_keys.h"
#include "packwandc/uapi/pwc_syscall.h"
#include "packwandc/kernel/pwc_error.h"
#include "packwandc/kernel/pwc_module_registry.h"
pwc_status pwc_keys_save(const uint8_t *secret, size_t secret_len) {
    if (secret == nullptr || secret_len == 0u) {
        return PWC_EINVAL;
    }
    return pwc_arch_keys_save(secret, secret_len);
}
pwc_status pwc_keys_load(uint8_t *buffer, size_t capacity, size_t *out_len) {
    if (buffer == nullptr || capacity == 0u || out_len == nullptr) {
        return PWC_EINVAL;
    }
    return pwc_arch_keys_load(buffer, capacity, out_len);
}
pwc_status pwc_keys_clear(void) { return pwc_arch_keys_clear(); }

/* --- module descriptor (packwandc.md 3.5) ------------------------------- */

static pwc_status pwc_pwkeys_init(pwc_module_ctx *ctx) {
    /* No state to build: pwkeys is stateless, and every object it hands out
     * lives in the kernel handle table rather than in the module. The
     * descriptor still earns its place -- it puts pwkeys in the boot order and
     * gives it a teardown hook for when that stops being true. */
    ctx->state = nullptr;
    PWC_NOTE(PWC_TRACE_LEVEL_INFO, "pwkeys", "module initialised: OS credential storage");
    return PWC_OK;
}

static void pwc_pwkeys_exit(pwc_module_ctx *ctx) {
    ctx->state = nullptr;
    PWC_NOTE(PWC_TRACE_LEVEL_INFO, "pwkeys", "module shut down");
}

const pwc_module pwc_module_pwkeys = {
    .name = "pwkeys",
    .abi_version = PWC_ABI_VERSION_MAJOR,
    .depends = nullptr,
    .init = pwc_pwkeys_init,
    .exit = pwc_pwkeys_exit,
};
