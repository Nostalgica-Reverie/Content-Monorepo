#include "packwandc/kernel/pwc_slab.h"

void pwc_slab_init(pwc_slab *slab, void *memory, uint32_t *next, uint32_t capacity, size_t object_size) {
    slab->memory = memory;
    slab->next = next;
    slab->capacity = capacity;
    slab->object_size = object_size;
    slab->free_head = capacity == 0u ? UINT32_MAX : 0u;
    for (uint32_t index = 0u; index < capacity; ++index) {
        next[index] = index + 1u < capacity ? index + 1u : UINT32_MAX;
    }
}

pwc_status pwc_slab_alloc(pwc_slab *slab, void **out) {
    if (slab == nullptr || out == nullptr || slab->memory == nullptr || slab->next == nullptr ||
        slab->object_size == 0u) {
        return PWC_EINVAL;
    }
    if (slab->free_head == UINT32_MAX) {
        return PWC_ENOMEM;
    }
    const uint32_t index = slab->free_head;
    slab->free_head = slab->next[index];
    *out = &slab->memory[(size_t) index * slab->object_size];
    return PWC_OK;
}

pwc_status pwc_slab_free(pwc_slab *slab, void *object) {
    if (slab == nullptr || object == nullptr || slab->memory == nullptr || slab->next == nullptr ||
        slab->object_size == 0u) {
        return PWC_EINVAL;
    }
    const uintptr_t offset = (uintptr_t) object - (uintptr_t) slab->memory;
    if (offset % slab->object_size != 0u || offset / slab->object_size >= slab->capacity) {
        return PWC_EINVAL;
    }
    const uint32_t index = (uint32_t) (offset / slab->object_size);
    slab->next[index] = slab->free_head;
    slab->free_head = index;
    return PWC_OK;
}
