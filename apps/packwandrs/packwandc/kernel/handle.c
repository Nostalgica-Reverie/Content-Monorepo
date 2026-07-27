#include "packwandc/kernel/pwc_error.h"
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
        return PWC_FAIL(PWC_EINVAL, "core", "pwc_handle_open: null table/out, no kind, or unknown rights");
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
    return PWC_FAIL_PLATFORM(PWC_ENOMEM, "core", "handle table is full", (int32_t) table->capacity);
}

pwc_status pwc_handle_validate(const pwc_handle_table *table, pwc_handle_t h, uint32_t required) {
    if (table == nullptr || h.index == 0u || h.index > table->capacity) {
        return PWC_FAIL(PWC_EBADF, "core", "handle index is zero or past the table capacity");
    }
    const pwc_handle_slot *slot = &table->slots[h.index];
    /* The generation check comes before the liveness check on purpose. A stale
     * handle whose slot has since been reused is the dangerous case -- it names
     * a live object that is not the caller's -- and it must report ESTALE, not
     * succeed and not merely say "bad handle". This ordering is what turns a
     * use-after-free into a returned error (packwandc.md 3.2). */
    if (slot->generation != h.generation) {
        return PWC_FAIL_PLATFORM(
            PWC_ESTALE, "core", "handle generation mismatch: the slot was reused", (int32_t) h.index);
    }
    if (slot->kind == PWC_OBJECT_NONE) {
        return PWC_FAIL_PLATFORM(PWC_EBADF, "core", "handle refers to a closed slot", (int32_t) h.index);
    }
    if ((slot->rights & required) != required) {
        /* The missing bits, not the requested set: that is the actionable part. */
        return PWC_FAIL_PLATFORM(
            PWC_EPERM, "core", "handle lacks a required right", (int32_t) (required & ~slot->rights));
    }
    return PWC_OK;
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
    /* Rights only ever narrow (packwandc.md 3.2). A dup asking for a bit the
     * source does not hold is refused rather than silently clamped, so a
     * capability handed to a less-trusted consumer cannot be re-widened and a
     * caller cannot believe it got more than it did. */
    if ((rights & ~table->slots[h.index].rights) != 0u) {
        return PWC_FAIL_PLATFORM(PWC_EPERM,
                                 "core",
                                 "pwc_handle_dup cannot add rights the source lacks",
                                 (int32_t) (rights & ~table->slots[h.index].rights));
    }
    return pwc_handle_open(table, table->slots[h.index].kind, rights, out);
}
