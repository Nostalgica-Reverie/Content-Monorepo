/* packwandc trace records -- the dmesg analogue (packwandc.md 3.7).
 *
 * The record type is uapi rather than kernel-internal because the host drains
 * the ring across the FFI boundary: packwandc-host pulls records out and feeds
 * them to the existing frontend log paths. The ring itself
 * (include/packwandc/kernel/pwc_ktrace.h) stays internal -- callers get records
 * out of it through a syscall, never a pointer into it.
 *
 * WHAT IS DELIBERATELY NOT HERE
 *
 * No syscall number field, though packwandc.md 3.7 lists one. Nothing
 * currently knows it: records are emitted at the failure choke point in
 * kernel/status.c, which sees a status and a source location but not the
 * dispatch that led there. A field that is always zero is worse than an absent
 * one -- it reads as information. It arrives with the dispatcher that can fill
 * it in.
 */
#ifndef PACKWANDC_UAPI_PWC_TRACE_H
#define PACKWANDC_UAPI_PWC_TRACE_H

#include "packwandc/uapi/pwc_abi.h"
#include "packwandc/uapi/pwc_status.h"

PWC_BEGIN_DECLS

/* Severity. Ordered so that a consumer can filter with a single >= test. */
enum {
    PWC_TRACE_LEVEL_DEBUG = 0u,
    PWC_TRACE_LEVEL_INFO = 1u,
    PWC_TRACE_LEVEL_WARN = 2u,
    PWC_TRACE_LEVEL_ERROR = 3u,
};

PWC_ABI_PACKED_BEGIN
typedef struct pwc_trace_record {
    uint32_t struct_size;  /* sizeof(pwc_trace_record); forward compatibility */
    uint32_t level;        /* one of PWC_TRACE_LEVEL_* */
    uint64_t sequence;     /* monotonic across the ring's lifetime */
    int32_t status;        /* the pwc_status being reported, or PWC_OK */
    int32_t platform_code; /* GetLastError()/errno/D-Bus code, 0 if none */
    uint32_t line;         /* __LINE__ of the emitting site */
    uint32_t reserved;     /* explicit, so the pointer alignment below is not
                            * silent padding a future field would collide with */
    const char *module;    /* static, never NULL */
    const char *message;   /* static, never NULL */
    const char *file;      /* static, never NULL */
} pwc_trace_record;
PWC_ABI_PACKED_END

static_assert(sizeof(pwc_trace_record) == 56, "pwc_trace_record is part of the wire ABI");

PWC_END_DECLS

#endif /* PACKWANDC_UAPI_PWC_TRACE_H */
