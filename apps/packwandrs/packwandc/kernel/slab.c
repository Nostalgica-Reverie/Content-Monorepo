#include "packwandc/kernel/pwc_slab.h"

#define PWC_SLAB_END       UINT32_MAX
#define PWC_SLAB_ALLOCATED (UINT32_MAX - 1u)

void pwc_slab_init(pwc_slab *slab, void *memory, uint32_t *next, uint32_t capacity, size_t object_size) {
    slab->memory = memory;
    slab->next = next;
    slab->capacity = capacity;
    slab->object_size = object_size;
    slab->free_head = capacity == 0u ? PWC_SLAB_END : 0u;
    for (uint32_t index = 0u; index < capacity; ++index) {
        next[index] = index + 1u < capacity ? index + 1u : PWC_SLAB_END;
    }
}

pwc_status pwc_slab_alloc(pwc_slab *slab, void **out) {
    if (slab == nullptr || out == nullptr || slab->memory == nullptr || slab->next == nullptr ||
        slab->object_size == 0u) {
        return PWC_EINVAL;
    }
    if (slab->free_head == PWC_SLAB_END) {
        return PWC_ENOMEM;
    }
    if (slab->free_head >= slab->capacity) {
        return PWC_EIO;
    }
    const uint32_t index = slab->free_head;
    slab->free_head = slab->next[index];
    slab->next[index] = PWC_SLAB_ALLOCATED;
    *out = &slab->memory[(size_t) index * slab->object_size];
    return PWC_OK;
}

pwc_status pwc_slab_free(pwc_slab *slab, void *object) {
    if (slab == nullptr || object == nullptr || slab->memory == nullptr || slab->next == nullptr ||
        slab->object_size == 0u) {
        return PWC_EINVAL;
    }
    const uintptr_t base = (uintptr_t) slab->memory;
    const uintptr_t address = (uintptr_t) object;
    if (address < base) {
        return PWC_EINVAL;
    }
    const uintptr_t offset = address - base;
    if (offset % slab->object_size != 0u || offset / slab->object_size >= slab->capacity) {
        return PWC_EINVAL;
    }
    const uint32_t index = (uint32_t) (offset / slab->object_size);
    if (slab->next[index] != PWC_SLAB_ALLOCATED) {
        return PWC_EINVAL;
    }
    slab->next[index] = slab->free_head;
    slab->free_head = index;
    return PWC_OK;
}
