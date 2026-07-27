#ifndef PACKWANDC_KERNEL_PWC_ARENA_H
#define PACKWANDC_KERNEL_PWC_ARENA_H

#include "packwandc/uapi/pwc_status.h"

typedef struct pwc_arena {
    uint8_t *memory;
    size_t capacity;
    size_t used;
} pwc_arena;

void pwc_arena_init(pwc_arena *arena, void *memory, size_t capacity);
pwc_status pwc_arena_alloc(pwc_arena *arena, size_t size, size_t alignment, void **out);
void pwc_arena_reset(pwc_arena *arena);

#endif
