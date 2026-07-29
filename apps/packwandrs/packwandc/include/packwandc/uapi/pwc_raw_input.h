/* Focused Windows Raw Input capture. This API is intentionally app-scoped: it
 * observes the Packwand window only and never installs a global hook. */
#ifndef PACKWANDC_UAPI_PWC_RAW_INPUT_H
#define PACKWANDC_UAPI_PWC_RAW_INPUT_H

#include "packwandc/uapi/pwc_abi.h"
#include "packwandc/uapi/pwc_status.h"

PWC_BEGIN_DECLS

enum { PWC_RAW_INPUT_QUEUE_CAPACITY = 2048u };
enum pwc_raw_input_kind { PWC_RAW_INPUT_KEYBOARD = 1, PWC_RAW_INPUT_MOUSE = 2 };

PWC_ABI_PACKED_BEGIN
typedef struct pwc_raw_input_event {
    uint32_t struct_size;
    uint32_t kind;
    uint32_t timestamp_ms;
    uint16_t make_code;
    uint16_t flags;
    uint16_t virtual_key;
    uint16_t button_flags;
    int32_t delta_x;
    int32_t delta_y;
    int16_t wheel_delta;
    uint16_t reserved;
} pwc_raw_input_event;
PWC_ABI_PACKED_END
static_assert(sizeof(pwc_raw_input_event) == 32u, "raw input event ABI");

/* Starts capture for one focused top-level Packwand window. `native_window` is
 * a Win32 HWND represented as an integer. Calling this does not disable
 * legacy input or alter OS pointer settings. */
PWC_API pwc_status pwc_raw_input_start(uintptr_t native_window);
PWC_API void pwc_raw_input_stop(void);
PWC_API pwc_status pwc_raw_input_read(pwc_raw_input_event *out);
PWC_API pwc_status pwc_raw_input_dropped(uint64_t *out);

PWC_END_DECLS
#endif
