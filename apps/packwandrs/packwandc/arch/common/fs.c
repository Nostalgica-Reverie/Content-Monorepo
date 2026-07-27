#include "packwandc/kernel/pwc_arch_fs.h"
pwc_status pwc_arch_fs_read(const uint8_t *root,
                            size_t root_len,
                            const uint8_t *path,
                            size_t path_len,
                            uint8_t *buffer,
                            size_t capacity,
                            size_t *out_len) {
    (void) root;
    (void) root_len;
    (void) path;
    (void) path_len;
    (void) buffer;
    (void) capacity;
    (void) out_len;
    return PWC_ENOSYS;
}
pwc_status pwc_arch_fs_atomic_write(const uint8_t *root,
                                    size_t root_len,
                                    const uint8_t *path,
                                    size_t path_len,
                                    const uint8_t *content,
                                    size_t content_len) {
    (void) root;
    (void) root_len;
    (void) path;
    (void) path_len;
    (void) content;
    (void) content_len;
    return PWC_ENOSYS;
}
pwc_status pwc_arch_fs_watch_open(const uint8_t *root, size_t root_len, uintptr_t *out_native) {
    (void) root;
    (void) root_len;
    (void) out_native;
    return PWC_ENOSYS;
}
pwc_status pwc_arch_fs_watch_read(uintptr_t native, size_t *out_events) {
    (void) native;
    (void) out_events;
    return PWC_ENOSYS;
}
pwc_status pwc_arch_fs_watch_close(uintptr_t native) {
    (void) native;
    return PWC_ENOSYS;
}
