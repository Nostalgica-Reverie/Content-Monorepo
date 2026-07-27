/* The ktrace ring -- lock-free MPSC, fixed size, never allocates, drops rather
 * than stalls, and counts its drops (packwandc.md 3.7).
 *
 * Many threads write; exactly one drains. That asymmetry is the design:
 * writers sit on hot paths and must never wait on the reader, so a full ring
 * discards the incoming record and bumps a counter instead of blocking.
 *
 * Each slot carries its own commit marker rather than the ring relying on the
 * write cursor alone. See kernel/ktrace.c for why -- the cursor-only version
 * had two races that this layout removes by construction.
 */
#ifndef PACKWANDC_KERNEL_PWC_KTRACE_H
#define PACKWANDC_KERNEL_PWC_KTRACE_H

#include "packwandc/uapi/pwc_status.h"
#include "packwandc/uapi/pwc_trace.h"
#include <stdatomic.h>

enum { PWC_KTRACE_CAPACITY = 256 };

/* Power of two so the slot index is a mask rather than a division, and so the
 * wraparound reasoning in ktrace.c holds. */
static_assert((PWC_KTRACE_CAPACITY & (PWC_KTRACE_CAPACITY - 1)) == 0,
              "PWC_KTRACE_CAPACITY must be a power of two");

typedef struct pwc_ktrace {
    pwc_trace_record records[PWC_KTRACE_CAPACITY];
    /* committed[i] reads `sequence + 1` once slot i is fully written, and
     * `sequence` while it is free for that sequence. The reader consumes a slot
     * only on seeing the exact value it expects, so it can never read a slot
     * that a preempted writer has reserved but not yet filled. */
    atomic_uint_fast64_t committed[PWC_KTRACE_CAPACITY];
    atomic_uint_fast64_t write_sequence;
    atomic_uint_fast64_t read_sequence;
    atomic_uint_fast64_t drops;
} pwc_ktrace;

void pwc_ktrace_init(pwc_ktrace *trace);

/* Publish one record. Never blocks. Returns PWC_EOVERFLOW when the ring is
 * full, having counted a drop and left the sequence space untouched. */
pwc_status pwc_ktrace_write(pwc_ktrace *trace, const pwc_trace_record *record);

/* Consume the oldest record. Single consumer only. PWC_EAGAIN when the ring is
 * empty or the next record is reserved but not yet committed. */
pwc_status pwc_ktrace_read(pwc_ktrace *trace, pwc_trace_record *out);

uint64_t pwc_ktrace_drops(const pwc_ktrace *trace);

#endif /* PACKWANDC_KERNEL_PWC_KTRACE_H */
