#include "packwandc/kernel/pwc_arena.h"

void pwc_arena_init(pwc_arena *arena, void *memory, size_t capacity) {
    arena->memory = memory;
    arena->capacity = capacity;
    arena->used = 0u;
}

pwc_status pwc_arena_alloc(pwc_arena *arena, size_t size, size_t alignment, void **out) {
    if (arena == nullptr || out == nullptr || arena->memory == nullptr || size == 0u || alignment == 0u ||
        (alignment & (alignment - 1u)) != 0u) {
        return PWC_EINVAL;
    }
    const uintptr_t base = (uintptr_t) arena->memory;
    const uintptr_t mask = (uintptr_t) alignment - 1u;
    if (arena->used > UINTPTR_MAX - base) {
        return PWC_EOVERFLOW;
    }
    const uintptr_t current = base + arena->used;
    if (current > UINTPTR_MAX - mask) {
        return PWC_EOVERFLOW;
    }
    const uintptr_t aligned = (current + mask) & ~mask;
    const uintptr_t displacement = aligned - base;
    if (displacement > SIZE_MAX) {
        return PWC_EOVERFLOW;
    }
    const size_t offset = (size_t) displacement;
    if (offset > arena->capacity || size > arena->capacity - offset) {
        return PWC_ENOMEM;
    }
    *out = (void *) aligned;
    arena->used = offset + size;
    return PWC_OK;
}

void pwc_arena_reset(pwc_arena *arena) {
    if (arena != nullptr) {
        arena->used = 0u;
    }
}
