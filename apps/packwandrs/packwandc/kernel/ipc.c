/* pwipc: framed messages over a shared ring. */

#include "packwandc/kernel/pwc_boot_internal.h"
#include "packwandc/kernel/pwc_error.h"
#include "packwandc/kernel/pwc_ipc.h"
#include "packwandc/uapi/pwc_syscall.h"

#include <string.h>

enum { PWC_IPC_MASK = PWC_IPC_PORT_CAPACITY - 1 };

void pwc_ipc_table_init(pwc_ipc_table *table) {
    if (table == nullptr) {
        return;
    }
    for (size_t i = 0u; i < (size_t) PWC_IPC_MAX_PORTS; ++i) {
        table->ports[i].in_use = false;
        atomic_init(&table->ports[i].head, 0u);
        atomic_init(&table->ports[i].tail, 0u);
    }
}

pwc_status pwc_ipc_port_alloc(pwc_ipc_table *table, uint32_t *out_index) {
    if (table == nullptr || out_index == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "pwipc", "pwc_ipc_port_alloc: null table or out");
    }
    for (uint32_t i = 0u; i < (uint32_t) PWC_IPC_MAX_PORTS; ++i) {
        if (!table->ports[i].in_use) {
            table->ports[i].in_use = true;
            /* Reset rather than trusting the previous tenant's cursors: a
             * reused slot must start empty, not inherit a backlog. */
            atomic_store_explicit(&table->ports[i].head, 0u, memory_order_relaxed);
            atomic_store_explicit(&table->ports[i].tail, 0u, memory_order_relaxed);
            *out_index = i;
            return PWC_OK;
        }
    }
    return PWC_FAIL_PLATFORM(PWC_ENOMEM, "pwipc", "no free port slots", PWC_IPC_MAX_PORTS);
}

pwc_status pwc_ipc_port_free(pwc_ipc_table *table, uint32_t index) {
    if (table == nullptr || index >= (uint32_t) PWC_IPC_MAX_PORTS) {
        return PWC_FAIL(PWC_EINVAL, "pwipc", "pwc_ipc_port_free: port index out of range");
    }
    if (!table->ports[index].in_use) {
        return PWC_FAIL(PWC_EBADF, "pwipc", "pwc_ipc_port_free: port is not open");
    }
    table->ports[index].in_use = false;
    return PWC_OK;
}

size_t pwc_ipc_port_pending(const pwc_ipc_port *port) {
    if (port == nullptr) {
        return 0u;
    }
    const uint64_t tail = (uint64_t) atomic_load_explicit(&port->tail, memory_order_acquire);
    const uint64_t head = (uint64_t) atomic_load_explicit(&port->head, memory_order_relaxed);
    return (size_t) (tail - head);
}

/* Copy into the ring, splitting at the wrap point. */
static void pwc_ipc_put(pwc_ipc_port *port, uint64_t at, const uint8_t *data, size_t length) {
    const size_t offset = (size_t) (at & (uint64_t) PWC_IPC_MASK);
    const size_t room = (size_t) PWC_IPC_PORT_CAPACITY - offset;
    const size_t first = length < room ? length : room;
    memcpy(&port->buffer[offset], data, first);
    if (first < length) {
        memcpy(&port->buffer[0], &data[first], length - first);
    }
}

/* Copy out of the ring, splitting at the wrap point. */
static void pwc_ipc_get(const pwc_ipc_port *port, uint64_t at, uint8_t *out, size_t length) {
    const size_t offset = (size_t) (at & (uint64_t) PWC_IPC_MASK);
    const size_t room = (size_t) PWC_IPC_PORT_CAPACITY - offset;
    const size_t first = length < room ? length : room;
    memcpy(out, &port->buffer[offset], first);
    if (first < length) {
        memcpy(&out[first], &port->buffer[0], length - first);
    }
}

pwc_status pwc_ipc_port_send(pwc_ipc_port *port, const uint8_t *data, size_t length) {
    if (port == nullptr || (data == nullptr && length != 0u)) {
        return PWC_FAIL(PWC_EINVAL, "pwipc", "pwc_ipc_port_send: null port or data");
    }
    if (length > (size_t) PWC_IPC_MAX_MESSAGE) {
        return PWC_FAIL(PWC_EOVERFLOW, "pwipc", "message exceeds the per-frame maximum");
    }

    const size_t frame = (size_t) PWC_IPC_FRAME_HEADER + length;
    const uint64_t tail = (uint64_t) atomic_load_explicit(&port->tail, memory_order_relaxed);
    const uint64_t head = (uint64_t) atomic_load_explicit(&port->head, memory_order_acquire);
    if ((size_t) (tail - head) + frame > (size_t) PWC_IPC_PORT_CAPACITY) {
        /* Refused whole. A partial frame would desynchronise the reader
         * permanently, so back-pressure is reported instead. */
        return PWC_FAIL(PWC_EOVERFLOW, "pwipc", "port ring is full");
    }

    /* Little-endian, byte at a time: frames are unaligned and may wrap. */
    const uint32_t header = (uint32_t) length;
    const uint8_t encoded[PWC_IPC_FRAME_HEADER] = {
        (uint8_t) (header & 0xffu),
        (uint8_t) ((header >> 8u) & 0xffu),
        (uint8_t) ((header >> 16u) & 0xffu),
        (uint8_t) ((header >> 24u) & 0xffu),
    };
    pwc_ipc_put(port, tail, encoded, (size_t) PWC_IPC_FRAME_HEADER);
    if (length != 0u) {
        pwc_ipc_put(port, tail + (uint64_t) PWC_IPC_FRAME_HEADER, data, length);
    }

    /* Publication. Release ordering makes every byte written above visible to
     * a reader that observes this store -- this is what makes a frame atomic. */
    atomic_store_explicit(&port->tail, (uint_fast64_t) (tail + frame), memory_order_release);
    return PWC_OK;
}

pwc_status pwc_ipc_port_recv(pwc_ipc_port *port, uint8_t *buffer, size_t capacity, size_t *out_len) {
    if (port == nullptr || buffer == nullptr || out_len == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "pwipc", "pwc_ipc_port_recv: null port, buffer or out_len");
    }

    const uint64_t head = (uint64_t) atomic_load_explicit(&port->head, memory_order_relaxed);
    const uint64_t tail = (uint64_t) atomic_load_explicit(&port->tail, memory_order_acquire);
    const size_t available = (size_t) (tail - head);
    if (available < (size_t) PWC_IPC_FRAME_HEADER) {
        return PWC_EAGAIN;
    }

    uint8_t encoded[PWC_IPC_FRAME_HEADER] = {0};
    pwc_ipc_get(port, head, encoded, (size_t) PWC_IPC_FRAME_HEADER);
    const size_t length = (size_t) encoded[0] | ((size_t) encoded[1] << 8u) | ((size_t) encoded[2] << 16u) |
                          ((size_t) encoded[3] << 24u);

    if (available < (size_t) PWC_IPC_FRAME_HEADER + length) {
        /* Unreachable while the single-writer contract holds: tail is published
         * only after a whole frame lands. Reported rather than asserted,
         * because the alternative is reading past the writer's cursor. */
        return PWC_FAIL(PWC_EIO, "pwipc", "truncated frame: more than one writer on this port?");
    }
    if (length > capacity) {
        /* The frame stays queued, so a caller can retry with a large enough
         * buffer instead of losing the message. */
        *out_len = length;
        return PWC_FAIL(PWC_EOVERFLOW, "pwipc", "message is larger than the caller's buffer");
    }

    if (length != 0u) {
        pwc_ipc_get(port, head + (uint64_t) PWC_IPC_FRAME_HEADER, buffer, length);
    }
    *out_len = length;
    atomic_store_explicit(
        &port->head, (uint_fast64_t) (head + (uint64_t) PWC_IPC_FRAME_HEADER + length), memory_order_release);
    return PWC_OK;
}

/* --- syscall surface ---------------------------------------------------- */

/* Resolve a port handle to its slot. Every syscall below goes through this, so
 * the rights and generation checks happen in exactly one place. */
static pwc_status pwc_ipc_resolve(pwc_handle_t handle, uint32_t rights, pwc_ipc_port **out_port) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    pwc_ipc_table *const table = pwc_kernel_ipc();
    if (handles == nullptr || table == nullptr) {
        return PWC_FAIL(PWC_ECANCELED, "pwipc", "the kernel is not booted");
    }
    PWC_TRY(pwc_handle_validate(handles, handle, rights));

    uintptr_t payload = 0u;
    PWC_TRY(pwc_handle_payload_get(handles, handle, PWC_OBJECT_PORT, &payload));
    /* Stored biased by one, so a valid slot 0 is not mistaken for "unset". */
    const uint32_t index = (uint32_t) (payload - 1u);
    if (index >= (uint32_t) PWC_IPC_MAX_PORTS || !table->ports[index].in_use) {
        return PWC_FAIL(PWC_EBADF, "pwipc", "port handle refers to a closed slot");
    }
    *out_port = &table->ports[index];
    return PWC_OK;
}

pwc_status pwc_ipc_port_create(pwc_handle_t *out) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    pwc_ipc_table *const table = pwc_kernel_ipc();
    if (handles == nullptr || table == nullptr) {
        return PWC_FAIL(PWC_ECANCELED, "pwipc", "the kernel is not booted");
    }
    if (out == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "pwipc", "pwc_ipc_port_create: null out");
    }

    uint32_t index = 0u;
    PWC_TRY(pwc_ipc_port_alloc(table, &index));

    const pwc_status opened = pwc_handle_open(handles, PWC_OBJECT_PORT, PWC_RIGHT_ALL, out);
    if (opened != PWC_OK) {
        (void) pwc_ipc_port_free(table, index);
        return opened;
    }
    const pwc_status attached =
        pwc_handle_payload_set(handles, *out, PWC_OBJECT_PORT, (uintptr_t) index + 1u);
    if (attached != PWC_OK) {
        (void) pwc_handle_close_table(handles, *out);
        (void) pwc_ipc_port_free(table, index);
        return attached;
    }
    return PWC_OK;
}

pwc_status pwc_ipc_send(pwc_handle_t port, const uint8_t *data, size_t length) {
    pwc_ipc_port *resolved = nullptr;
    PWC_TRY(pwc_ipc_resolve(port, PWC_RIGHT_WRITE, &resolved));
    return pwc_ipc_port_send(resolved, data, length);
}

pwc_status pwc_ipc_recv(pwc_handle_t port, uint8_t *buffer, size_t capacity, size_t *out_len) {
    pwc_ipc_port *resolved = nullptr;
    PWC_TRY(pwc_ipc_resolve(port, PWC_RIGHT_READ, &resolved));
    return pwc_ipc_port_recv(resolved, buffer, capacity, out_len);
}

pwc_status pwc_ipc_port_close(pwc_handle_t port) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    pwc_ipc_table *const table = pwc_kernel_ipc();
    if (handles == nullptr || table == nullptr) {
        return PWC_FAIL(PWC_ECANCELED, "pwipc", "the kernel is not booted");
    }

    uintptr_t payload = 0u;
    PWC_TRY(pwc_handle_payload_get(handles, port, PWC_OBJECT_PORT, &payload));
    const uint32_t index = (uint32_t) (payload - 1u);

    /* The slot is released even if closing the handle fails, because the slot
     * is the scarcer resource: leaking one costs a future session its port. */
    const pwc_status freed = pwc_ipc_port_free(table, index);
    const pwc_status closed = pwc_handle_close_table(handles, port);
    return freed != PWC_OK ? freed : closed;
}
