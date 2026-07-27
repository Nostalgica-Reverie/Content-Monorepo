#include "packwandc/kernel/pwc_ktrace.h"
#include <string.h>
void pwc_ktrace_init(pwc_ktrace *trace) {
    memset(trace, 0, sizeof(*trace));
    atomic_init(&trace->write_sequence, 0u);
    atomic_init(&trace->read_sequence, 0u);
    atomic_init(&trace->drops, 0u);
}
pwc_status pwc_ktrace_write(pwc_ktrace *trace, const pwc_ktrace_record *record) {
    if (trace == nullptr || record == nullptr) {
        return PWC_EINVAL;
    }
    const uint64_t sequence = atomic_fetch_add_explicit(&trace->write_sequence, 1u, memory_order_relaxed);
    const uint64_t read = atomic_load_explicit(&trace->read_sequence, memory_order_acquire);
    if (sequence - read >= PWC_KTRACE_CAPACITY) {
        (void) atomic_fetch_add_explicit(&trace->drops, 1u, memory_order_relaxed);
        return PWC_EOVERFLOW;
    }
    trace->records[sequence % PWC_KTRACE_CAPACITY] = *record;
    trace->records[sequence % PWC_KTRACE_CAPACITY].sequence = sequence;
    atomic_thread_fence(memory_order_release);
    return PWC_OK;
}
pwc_status pwc_ktrace_read(pwc_ktrace *trace, pwc_ktrace_record *out) {
    if (trace == nullptr || out == nullptr) {
        return PWC_EINVAL;
    }
    const uint64_t read = atomic_load_explicit(&trace->read_sequence, memory_order_relaxed);
    const uint64_t write = atomic_load_explicit(&trace->write_sequence, memory_order_acquire);
    if (read >= write) {
        return PWC_EAGAIN;
    }
    *out = trace->records[read % PWC_KTRACE_CAPACITY];
    atomic_store_explicit(&trace->read_sequence, read + 1u, memory_order_release);
    return PWC_OK;
}
uint64_t pwc_ktrace_drops(const pwc_ktrace *trace) {
    return atomic_load_explicit(&trace->drops, memory_order_relaxed);
}
