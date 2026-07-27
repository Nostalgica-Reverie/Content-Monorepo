#ifndef PACKWANDC_KERNEL_PWC_ARCH_FS_H
#define PACKWANDC_KERNEL_PWC_ARCH_FS_H
#include "packwandc/uapi/pwc_status.h"
pwc_status pwc_arch_fs_read(const uint8_t *root,
                            size_t root_len,
                            const uint8_t *path,
                            size_t path_len,
                            uint8_t *buffer,
                            size_t capacity,
                            size_t *out_len);
pwc_status pwc_arch_fs_atomic_write(const uint8_t *root,
                                    size_t root_len,
                                    const uint8_t *path,
                                    size_t path_len,
                                    const uint8_t *content,
                                    size_t content_len);
pwc_status pwc_arch_fs_watch_open(const uint8_t *root, size_t root_len, uintptr_t *out_native);
pwc_status pwc_arch_fs_watch_read(uintptr_t native, size_t *out_events);
pwc_status pwc_arch_fs_watch_close(uintptr_t native);
#endif
