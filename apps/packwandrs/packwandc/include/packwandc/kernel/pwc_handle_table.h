#ifndef PACKWANDC_KERNEL_PWC_HANDLE_TABLE_H
#define PACKWANDC_KERNEL_PWC_HANDLE_TABLE_H
#include "packwandc/uapi/pwc_syscall.h"
enum { PWC_HANDLE_CAPACITY_MAX = 256 };
typedef enum pwc_object_kind {
    PWC_OBJECT_NONE = 0,
    PWC_OBJECT_PORT = 1,
    PWC_OBJECT_PROCESS = 2,
    PWC_OBJECT_FS_WATCH = 3
} pwc_object_kind;
typedef struct pwc_handle_slot {
    uint32_t generation;
    uint32_t rights;
    pwc_object_kind kind;
    uintptr_t payload;
} pwc_handle_slot;
typedef struct pwc_handle_table {
    pwc_handle_slot slots[PWC_HANDLE_CAPACITY_MAX + 1];
    uint32_t capacity;
} pwc_handle_table;
void pwc_handle_table_init(pwc_handle_table *table, uint32_t capacity);
pwc_status pwc_handle_open(pwc_handle_table *table, pwc_object_kind kind, uint32_t rights, pwc_handle_t *out);
pwc_status pwc_handle_validate(const pwc_handle_table *table, pwc_handle_t h, uint32_t required);
pwc_status pwc_handle_close_table(pwc_handle_table *table, pwc_handle_t h);
pwc_status
pwc_handle_payload_set(pwc_handle_table *table, pwc_handle_t h, pwc_object_kind kind, uintptr_t payload);
pwc_status
pwc_handle_payload_get(const pwc_handle_table *table, pwc_handle_t h, pwc_object_kind kind, uintptr_t *out);
pwc_status pwc_handle_dup_table(pwc_handle_table *table, pwc_handle_t h, uint32_t rights, pwc_handle_t *out);
#endif
