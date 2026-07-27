/* pwipc -- ports carrying framed messages over a shared ring (packwandc.md 5).
 *
 * A port is a single-producer, single-consumer byte ring with a length prefix
 * in front of every message, so a reader gets back exactly the messages a
 * writer sent rather than a stream it has to re-delimit. pw4shell writes its
 * output here and the host drains it; that is the first consumer, and the
 * shape is driven by it.
 *
 * SPSC, and that is load-bearing. The ring publishes `tail` only after a whole
 * frame is in the buffer, so a reader either sees a complete message or sees
 * nothing -- there is no torn frame and no lock. Two concurrent writers on one
 * port would break that, which is why the contract is one producer per port
 * rather than "it happens to work most of the time".
 *
 * No allocation: ports come from a fixed table sized at boot (packwandc.md
 * 3.4), so the whole subsystem is a compile-time amount of memory.
 */
#ifndef PACKWANDC_KERNEL_PWC_IPC_H
#define PACKWANDC_KERNEL_PWC_IPC_H

#include "packwandc/uapi/pwc_status.h"
#include <stdatomic.h>

enum {
    /* Bytes per port. Sized for a burst of shell output between drains rather
     * than for a single message. */
    PWC_IPC_PORT_CAPACITY = 8192,
    /* Concurrent ports. One per shell session plus headroom. */
    PWC_IPC_MAX_PORTS = 8,
    /* Length prefix width. Frames are `uint32_t length` then `length` bytes. */
    PWC_IPC_FRAME_HEADER = 4,
    /* Largest single message. Bounded well below the ring so one oversized
     * send cannot wedge a port that would otherwise drain fine. */
    PWC_IPC_MAX_MESSAGE = 4096,
};

/* Power of two so the offset is a mask rather than a modulo, and so the
 * monotonic-counter wraparound below is exact. */
static_assert((PWC_IPC_PORT_CAPACITY & (PWC_IPC_PORT_CAPACITY - 1)) == 0,
              "PWC_IPC_PORT_CAPACITY must be a power of two");
static_assert(PWC_IPC_MAX_MESSAGE + PWC_IPC_FRAME_HEADER < PWC_IPC_PORT_CAPACITY,
              "a single maximum-size frame must fit with room to spare");

typedef struct pwc_ipc_port {
    uint8_t buffer[PWC_IPC_PORT_CAPACITY];
    /* Monotonic byte counters, never wrapped by hand -- the mask does that.
     * `used = tail - head` stays correct across a uint64 wrap. */
    atomic_uint_fast64_t head; /* reader owns */
    atomic_uint_fast64_t tail; /* writer owns */
    bool in_use;
} pwc_ipc_port;

typedef struct pwc_ipc_table {
    pwc_ipc_port ports[PWC_IPC_MAX_PORTS];
} pwc_ipc_table;

void pwc_ipc_table_init(pwc_ipc_table *table);

/* Claim a free port slot. `out_index` is the slot, not a handle. */
pwc_status pwc_ipc_port_alloc(pwc_ipc_table *table, uint32_t *out_index);

/* Release a slot and discard anything still buffered in it. */
pwc_status pwc_ipc_port_free(pwc_ipc_table *table, uint32_t index);

/* Append one framed message. PWC_EOVERFLOW when the ring lacks room for the
 * whole frame -- a partial write is never published. */
pwc_status pwc_ipc_port_send(pwc_ipc_port *port, const uint8_t *data, size_t length);

/* Pop the oldest framed message. PWC_EAGAIN when empty; PWC_EOVERFLOW when the
 * caller's buffer is smaller than the frame, in which case the frame stays
 * queued and `out_len` reports the size needed. */
pwc_status pwc_ipc_port_recv(pwc_ipc_port *port, uint8_t *buffer, size_t capacity, size_t *out_len);

/* Bytes currently buffered, headers included. For readiness checks. */
size_t pwc_ipc_port_pending(const pwc_ipc_port *port);

#endif /* PACKWANDC_KERNEL_PWC_IPC_H */
