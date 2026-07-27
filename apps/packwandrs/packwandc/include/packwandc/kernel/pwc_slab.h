#ifndef PACKWANDC_KERNEL_PWC_SLAB_H
#define PACKWANDC_KERNEL_PWC_SLAB_H

#include "packwandc/uapi/pwc_status.h"

typedef struct pwc_slab {
    uint8_t *memory;
    uint32_t *next;
    uint32_t free_head;
    uint32_t capacity;
    size_t object_size;
} pwc_slab;

void pwc_slab_init(pwc_slab *slab, void *memory, uint32_t *next, uint32_t capacity, size_t object_size);
pwc_status pwc_slab_alloc(pwc_slab *slab, void **out);
pwc_status pwc_slab_free(pwc_slab *slab, void *object);

#endif
