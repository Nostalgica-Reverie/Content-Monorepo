#include "packwandc/uapi/pwc_syscall.h"
#include "packwandc/kernel/pwc_arch_fs.h"
#include "packwandc/kernel/pwc_boot_internal.h"

static bool pwc_fs_separator(uint8_t value) { return value == (uint8_t) '/' || value == (uint8_t) '\\'; }

pwc_status pwc_fs_validate_relative(const uint8_t *path, size_t path_len) {
    if (path == nullptr && path_len != 0u) {
        return PWC_EINVAL;
    }
    if (path_len == 0u) {
        return PWC_OK;
    }
    if (pwc_fs_separator(path[0]) || (path_len >= 2u && path[1] == (uint8_t) ':')) {
        return PWC_EPERM;
    }
    size_t component_start = 0u;
    for (size_t index = 0u; index <= path_len; ++index) {
        if (index == path_len || pwc_fs_separator(path[index])) {
            const size_t length = index - component_start;
            if (length == 0u || (length == 1u && path[component_start] == (uint8_t) '.') ||
                (length == 2u && path[component_start] == (uint8_t) '.' &&
                 path[component_start + 1u] == (uint8_t) '.')) {
                return PWC_EPERM;
            }
            component_start = index + 1u;
        }
    }
    return PWC_OK;
}

pwc_status pwc_fs_read(const uint8_t *root,
                       size_t root_len,
                       const uint8_t *path,
                       size_t path_len,
                       uint8_t *buffer,
                       size_t capacity,
                       size_t *out_len) {
    if (root == nullptr || root_len == 0u || buffer == nullptr || capacity == 0u || out_len == nullptr) {
        return PWC_EINVAL;
    }
    PWC_TRY(pwc_fs_validate_relative(path, path_len));
    return pwc_arch_fs_read(root, root_len, path, path_len, buffer, capacity, out_len);
}

pwc_status pwc_fs_atomic_write(const uint8_t *root,
                               size_t root_len,
                               const uint8_t *path,
                               size_t path_len,
                               const uint8_t *content,
                               size_t content_len) {
    if (root == nullptr || root_len == 0u || content == nullptr) {
        return PWC_EINVAL;
    }
    PWC_TRY(pwc_fs_validate_relative(path, path_len));
    return pwc_arch_fs_atomic_write(root, root_len, path, path_len, content, content_len);
}

pwc_status pwc_fs_watch_open(const uint8_t *root, size_t root_len, pwc_handle_t *out) {
    if (root == nullptr || root_len == 0u || out == nullptr) {
        return PWC_EINVAL;
    }
    pwc_handle_table *const handles = pwc_kernel_handles();
    if (handles == nullptr) {
        return PWC_ECANCELED;
    }
    uintptr_t native = 0u;
    PWC_TRY(pwc_arch_fs_watch_open(root, root_len, &native));
    const pwc_status opened =
        pwc_handle_open(handles, PWC_OBJECT_FS_WATCH, PWC_RIGHT_WAIT | PWC_RIGHT_CLOSE, out);
    if (opened != PWC_OK) {
        (void) pwc_arch_fs_watch_close(native);
        return opened;
    }
    return pwc_handle_payload_set(handles, *out, PWC_OBJECT_FS_WATCH, native);
}

pwc_status pwc_fs_watch_read(pwc_handle_t watch, size_t *out_events) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    if (handles == nullptr || out_events == nullptr) {
        return PWC_EINVAL;
    }
    uintptr_t native = 0u;
    PWC_TRY(pwc_handle_payload_get(handles, watch, PWC_OBJECT_FS_WATCH, &native));
    return pwc_arch_fs_watch_read(native, out_events);
}

pwc_status pwc_fs_watch_close(pwc_handle_t watch) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    if (handles == nullptr) {
        return PWC_ECANCELED;
    }
    uintptr_t native = 0u;
    PWC_TRY(pwc_handle_payload_get(handles, watch, PWC_OBJECT_FS_WATCH, &native));
    const pwc_status closed_native = pwc_arch_fs_watch_close(native);
    const pwc_status closed_handle = pwc_handle_close_table(handles, watch);
    return closed_native != PWC_OK ? closed_native : closed_handle;
}
