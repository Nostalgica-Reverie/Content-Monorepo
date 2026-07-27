#ifndef PACKWANDC_KERNEL_PWC_KTRACE_H
#define PACKWANDC_KERNEL_PWC_KTRACE_H
#include "packwandc/uapi/pwc_status.h"
#include <stdatomic.h>
enum { PWC_KTRACE_CAPACITY = 256, PWC_KTRACE_PAYLOAD = 32 };
typedef struct pwc_ktrace_record {
    uint64_t sequence;
    uint32_t level;
    uint32_t module;
    int32_t syscall_nr;
    pwc_status status;
    uint8_t payload[PWC_KTRACE_PAYLOAD];
} pwc_ktrace_record;
typedef struct pwc_ktrace {
    pwc_ktrace_record records[PWC_KTRACE_CAPACITY];
    atomic_uint_fast64_t write_sequence;
    atomic_uint_fast64_t read_sequence;
    atomic_uint_fast64_t drops;
} pwc_ktrace;
void pwc_ktrace_init(pwc_ktrace *trace);
pwc_status pwc_ktrace_write(pwc_ktrace *trace, const pwc_ktrace_record *record);
pwc_status pwc_ktrace_read(pwc_ktrace *trace, pwc_ktrace_record *out);
uint64_t pwc_ktrace_drops(const pwc_ktrace *trace);
#endif
