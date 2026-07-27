/* Status code names and descriptions -- see packwandc.md 3.1.
 *
 * Both tables are generated from PWC_STATUS_LIST in uapi/pwc_status.h, so a
 * new status code cannot be added without also getting a name and a
 * description. That is the entire reason the list is an X-macro.
 */

#include "packwandc/uapi/pwc_status.h"

static thread_local pwc_error_detail pwc_last_detail = {.module = "core",
                                                        .message = "success",
                                                        .platform_code = 0};

const pwc_error_detail *pwc_last_error(void) { return &pwc_last_detail; }

const char *pwc_status_name(pwc_status status) {
    switch (status) {
#define PWC_STATUS_NAME_CASE(name, value, desc)                                                              \
    case (value):                                                                                            \
        return #name;
        PWC_STATUS_LIST(PWC_STATUS_NAME_CASE)
#undef PWC_STATUS_NAME_CASE
        default:
            /* Never NULL: callers log this directly and a null check at every
             * call site would be pure noise. */
            return "PWC_EUNKNOWN";
    }
}

const char *pwc_status_describe(pwc_status status) {
    switch (status) {
#define PWC_STATUS_DESC_CASE(name, value, desc)                                                              \
    case (value):                                                                                            \
        return (desc);
        PWC_STATUS_LIST(PWC_STATUS_DESC_CASE)
#undef PWC_STATUS_DESC_CASE
        default:
            return "unknown status code";
    }
}
