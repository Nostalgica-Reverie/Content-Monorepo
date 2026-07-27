#include "packwandc/kernel/pwc_handle_table.h"

void pwc_handle_table_init(pwc_handle_table *table, uint32_t capacity) {
    *table = (pwc_handle_table){0};
    table->capacity = capacity;
    for (uint32_t i = 1u; i <= capacity; ++i) {
        table->slots[i].generation = 1u;
    }
}

pwc_status
pwc_handle_open(pwc_handle_table *table, pwc_object_kind kind, uint32_t rights, pwc_handle_t *out) {
    if (table == nullptr || out == nullptr || kind == PWC_OBJECT_NONE ||
        (rights & ~(uint32_t) PWC_RIGHT_ALL) != 0u) {
        return PWC_EINVAL;
    }
    for (uint32_t i = 1u; i <= table->capacity; ++i) {
        pwc_handle_slot *slot = &table->slots[i];
        if (slot->kind == PWC_OBJECT_NONE) {
            slot->kind = kind;
            slot->rights = rights;
            out->index = i;
            out->generation = slot->generation;
            return PWC_OK;
        }
    }
    return PWC_ENOMEM;
}

pwc_status pwc_handle_validate(const pwc_handle_table *table, pwc_handle_t h, uint32_t required) {
    if (table == nullptr || h.index == 0u || h.index > table->capacity) {
        return PWC_EBADF;
    }
    const pwc_handle_slot *slot = &table->slots[h.index];
    if (slot->generation != h.generation) {
        return PWC_ESTALE;
    }
    if (slot->kind == PWC_OBJECT_NONE) {
        return PWC_EBADF;
    }
    return (slot->rights & required) == required ? PWC_OK : PWC_EPERM;
}

pwc_status pwc_handle_close_table(pwc_handle_table *table, pwc_handle_t h) {
    const pwc_status status = pwc_handle_validate(table, h, PWC_RIGHT_CLOSE);
    if (status != PWC_OK) {
        return status;
    }
    pwc_handle_slot *slot = &table->slots[h.index];
    slot->kind = PWC_OBJECT_NONE;
    slot->rights = PWC_RIGHT_NONE;
    slot->payload = 0u;
    ++slot->generation;
    if (slot->generation == 0u) {
        slot->generation = 1u;
    }
    return PWC_OK;
}

pwc_status
pwc_handle_payload_set(pwc_handle_table *table, pwc_handle_t h, pwc_object_kind kind, uintptr_t payload) {
    const pwc_status status = pwc_handle_validate(table, h, PWC_RIGHT_CLOSE);
    if (status != PWC_OK) {
        return status;
    }
    if (table->slots[h.index].kind != kind || payload == 0u) {
        return PWC_EINVAL;
    }
    table->slots[h.index].payload = payload;
    return PWC_OK;
}

pwc_status
pwc_handle_payload_get(const pwc_handle_table *table, pwc_handle_t h, pwc_object_kind kind, uintptr_t *out) {
    const pwc_status status = pwc_handle_validate(table, h, PWC_RIGHT_CLOSE);
    if (status != PWC_OK) {
        return status;
    }
    if (out == nullptr || table->slots[h.index].kind != kind || table->slots[h.index].payload == 0u) {
        return PWC_EINVAL;
    }
    *out = table->slots[h.index].payload;
    return PWC_OK;
}
pwc_status pwc_handle_dup_table(pwc_handle_table *table, pwc_handle_t h, uint32_t rights, pwc_handle_t *out) {
    const pwc_status status = pwc_handle_validate(table, h, PWC_RIGHT_DUP);
    if (status != PWC_OK) {
        return status;
    }
    if ((rights & ~table->slots[h.index].rights) != 0u) {
        return PWC_EPERM;
    }
    return pwc_handle_open(table, table->slots[h.index].kind, rights, out);
}
