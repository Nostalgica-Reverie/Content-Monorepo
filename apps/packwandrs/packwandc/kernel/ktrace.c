/* Lock-free MPSC trace ring (packwandc.md 3.7).
 *
 * THE TWO RACES THIS LAYOUT REMOVES
 *
 * The previous version reserved a sequence with fetch_add and *then* checked
 * whether the ring was full. Both halves of that were wrong:
 *
 *   1. A drop still consumed its sequence number. The slot for it was never
 *      written, but the reader had no way to know: on reaching that sequence it
 *      returned whatever the slot held from 256 records ago, as though it were
 *      fresh. A dropped record silently became a duplicated stale one.
 *
 *   2. Reservation is not publication. With writer A holding sequence 5 and
 *      preempted before storing, writer B could complete sequence 6 and leave
 *      the write cursor at 7. The reader compared cursors, concluded 5 was
 *      available, and read a slot nobody had written yet.
 *
 * The fix is that the write cursor no longer means "published". Reservation is
 * a compare-exchange, so a full ring costs nothing and skips no sequence, and
 * each slot carries its own commit marker that the reader must match exactly.
 * `records[i]` is therefore only ever read after the release store to
 * `committed[i]` that published it.
 */

#include "packwandc/kernel/pwc_ktrace.h"

#include <string.h>

enum { PWC_KTRACE_MASK = PWC_KTRACE_CAPACITY - 1 };

void pwc_ktrace_init(pwc_ktrace *trace) {
    if (trace == nullptr) {
        return;
    }
    memset(trace, 0, sizeof(*trace));
    for (size_t i = 0u; i < (size_t) PWC_KTRACE_CAPACITY; ++i) {
        /* Slot i is initially free for sequence i, which is what the writer's
         * "is this slot free" test compares against. */
        atomic_init(&trace->committed[i], (uint_fast64_t) i);
    }
    atomic_init(&trace->write_sequence, 0u);
    atomic_init(&trace->read_sequence, 0u);
    atomic_init(&trace->drops, 0u);
}

pwc_status pwc_ktrace_write(pwc_ktrace *trace, const pwc_trace_record *record) {
    if (trace == nullptr || record == nullptr) {
        return PWC_EINVAL;
    }

    uint64_t sequence = (uint64_t) atomic_load_explicit(&trace->write_sequence, memory_order_relaxed);
    for (;;) {
        const size_t slot = (size_t) (sequence & (uint64_t) PWC_KTRACE_MASK);
        const uint64_t state = (uint64_t) atomic_load_explicit(&trace->committed[slot], memory_order_acquire);
        if (state != sequence) {
            /* The slot still holds an unread record, so the ring is full at
             * this sequence. Drop rather than stall -- a writer on a hot path
             * must never wait on the drain (packwandc.md 3.7). No sequence is
             * consumed, so this leaves no hole for the reader to trip over. */
            (void) atomic_fetch_add_explicit(&trace->drops, 1u, memory_order_relaxed);
            return PWC_EOVERFLOW;
        }
        /* Claim the sequence. On failure another writer took it and the
         * updated value is reloaded into `sequence` for the retry. */
        if (atomic_compare_exchange_weak_explicit(&trace->write_sequence,
                                                  &sequence,
                                                  sequence + 1u,
                                                  memory_order_relaxed,
                                                  memory_order_relaxed)) {
            trace->records[slot] = *record;
            trace->records[slot].sequence = sequence;
            trace->records[slot].struct_size = (uint32_t) sizeof(pwc_trace_record);
            /* Publication. Everything above must be visible to a reader that
             * observes this store, which is what the release ordering buys. */
            atomic_store_explicit(
                &trace->committed[slot], (uint_fast64_t) (sequence + 1u), memory_order_release);
            return PWC_OK;
        }
    }
}

pwc_status pwc_ktrace_read(pwc_ktrace *trace, pwc_trace_record *out) {
    if (trace == nullptr || out == nullptr) {
        return PWC_EINVAL;
    }

    /* Single consumer, so the read cursor needs no synchronisation with itself. */
    const uint64_t sequence = (uint64_t) atomic_load_explicit(&trace->read_sequence, memory_order_relaxed);
    const size_t slot = (size_t) (sequence & (uint64_t) PWC_KTRACE_MASK);

    if ((uint64_t) atomic_load_explicit(&trace->committed[slot], memory_order_acquire) != sequence + 1u) {
        /* Empty, or the writer holding this sequence has not published yet.
         * Both mean "nothing to hand out", and neither may read the slot. */
        return PWC_EAGAIN;
    }

    *out = trace->records[slot];
    atomic_store_explicit(&trace->read_sequence, (uint_fast64_t) (sequence + 1u), memory_order_relaxed);
    /* Release the slot for the sequence that will next land on it, one full
     * lap later. This is the store a blocked writer is waiting to observe. */
    atomic_store_explicit(&trace->committed[slot],
                          (uint_fast64_t) (sequence + (uint64_t) PWC_KTRACE_CAPACITY),
                          memory_order_release);
    return PWC_OK;
}

uint64_t pwc_ktrace_drops(const pwc_ktrace *trace) {
    return trace == nullptr ? 0u : (uint64_t) atomic_load_explicit(&trace->drops, memory_order_relaxed);
}
