/* Phase 0 unit tests: the ABI surface, status mapping, handle encoding, and
 * the two core syscalls. See packwandc.md 11.
 */

#include "pwc_test.h"

#include "packwandc/kernel/pwc_kernel.h"
#include "packwandc/uapi/pwc_handle.h"
#include "packwandc/uapi/pwc_status.h"
#include "packwandc/uapi/pwc_syscall.h"

static void test_version_syscall(void) {
    uint32_t major = 0xffffffffu;
    uint32_t minor = 0xffffffffu;

    PWC_CHECK_EQ_I(pwc_sys_version(&major, &minor), PWC_OK);
    PWC_CHECK_EQ_U(major, PWC_ABI_VERSION_MAJOR);
    PWC_CHECK_EQ_U(minor, PWC_ABI_VERSION_MINOR);
}

static void test_version_rejects_null(void) {
    uint32_t scratch = 0u;

    /* Every syscall validates its own arguments; there is no trusted caller. */
    PWC_CHECK_EQ_I(pwc_sys_version(nullptr, &scratch), PWC_EINVAL);
    PWC_CHECK_EQ_I(pwc_sys_version(&scratch, nullptr), PWC_EINVAL);
    PWC_CHECK_EQ_I(pwc_sys_version(nullptr, nullptr), PWC_EINVAL);
}

static void test_status_names(void) {
    PWC_CHECK_EQ_STR(pwc_status_name(PWC_OK), "PWC_OK");
    PWC_CHECK_EQ_STR(pwc_status_name(PWC_EINVAL), "PWC_EINVAL");
    PWC_CHECK_EQ_STR(pwc_status_name(PWC_ESTALE), "PWC_ESTALE");

    /* The never-NULL promise, for a code that is not in the table. */
    PWC_CHECK_EQ_STR(pwc_status_name(-9999), "PWC_EUNKNOWN");
    PWC_CHECK_EQ_STR(pwc_status_describe(-9999), "unknown status code");
}

static void test_status_syscall_matches_direct_call(void) {
    /* Syscall 2 is a thin wrapper; if these ever diverge the wrapper has
     * grown logic it should not have. */
    PWC_CHECK_EQ_STR(pwc_sys_status_name(PWC_EBADF), pwc_status_name(PWC_EBADF));
}

static void test_status_codes_are_negative(void) {
    /* The contract in packwandc.md 3.1: PWC_OK is zero and every failure is
     * negative, so `st < 0` is a complete failure test. */
#define PWC_CHECK_STATUS_SIGN(name, value, desc) PWC_CHECK((value) <= 0);
    PWC_STATUS_LIST(PWC_CHECK_STATUS_SIGN)
#undef PWC_CHECK_STATUS_SIGN
    PWC_CHECK_EQ_I(PWC_OK, 0);
}

static void test_handle_encoding_roundtrips(void) {
    const pwc_handle_t original = {.index = 0x1234abcdu, .generation = 0x0fedcba9u};
    const pwc_handle_t decoded = pwc_handle_unpack(pwc_handle_pack(original));

    PWC_CHECK(pwc_handle_eq(original, decoded));
    PWC_CHECK_EQ_U(decoded.index, 0x1234abcdu);
    PWC_CHECK_EQ_U(decoded.generation, 0x0fedcba9u);
}

static void test_handle_encoding_edges(void) {
    /* All-ones in both fields must survive: the packing must not sign-extend
     * or truncate at the 32-bit boundary. */
    const pwc_handle_t saturated = {.index = 0xffffffffu, .generation = 0xffffffffu};
    const pwc_handle_t decoded = pwc_handle_unpack(pwc_handle_pack(saturated));

    PWC_CHECK(pwc_handle_eq(saturated, decoded));
    PWC_CHECK_EQ_U(pwc_handle_pack(saturated), 0xffffffffffffffffull);
}

static void test_invalid_handle(void) {
    const pwc_handle_t invalid = PWC_HANDLE_INVALID;

    /* A zero-initialised handle must be invalid by construction, so a caller
     * who forgets to initialise gets PWC_EBADF and not slot 0.
     *
     * Written {0} rather than {}: C23 allows the empty initialiser, but clang
     * 16 still reports it as a GNU extension under -Wpedantic, and the gate
     * builds with -Werror (packwandc.md 7.1). Revisit when the floor is 19. */
    const pwc_handle_t zeroed = {0};

    PWC_CHECK(!pwc_handle_is_valid(invalid));
    PWC_CHECK(!pwc_handle_is_valid(zeroed));
    PWC_CHECK(pwc_handle_eq(invalid, zeroed));

    const pwc_handle_t live = {.index = 1u, .generation = 0u};
    PWC_CHECK(pwc_handle_is_valid(live));
}

static void test_rights_narrow_only(void) {
    /* PWC_RIGHT_ALL is asserted to be the union of every right at compile
     * time; this checks the runtime consequence callers depend on. */
    PWC_CHECK_EQ_U(PWC_RIGHT_ALL & PWC_RIGHT_READ, PWC_RIGHT_READ);
    PWC_CHECK_EQ_U(PWC_RIGHT_NONE, 0u);
    PWC_CHECK_EQ_U(PWC_RIGHT_ALL & ~(unsigned) PWC_RIGHT_WRITE,
                   PWC_RIGHT_READ | PWC_RIGHT_WAIT | PWC_RIGHT_TRANSFER | PWC_RIGHT_DUP | PWC_RIGHT_CLOSE);
}

static void test_syscall_numbers_are_stable(void) {
    /* These literals are the ABI. Changing one is a breaking change and must
     * fail here as well as in tests/golden/syscalls.txt. */
    PWC_CHECK_EQ_I(PWC_SYS_pwc_sys_version, 1);
    PWC_CHECK_EQ_I(PWC_SYS_pwc_sys_status_name, 2);
    PWC_CHECK_EQ_I(PWC_SYS_pwc_handle_close, 3);
    PWC_CHECK_EQ_I(PWC_SYS_pwc_ipc_port_create, 64);
    PWC_CHECK_EQ_I(PWC_SYS_pwc_fs_validate_relative, 16);
    PWC_CHECK_EQ_I(PWC_SYS_pwc_proc_adopt, 32);
    PWC_CHECK_EQ_I(PWC_SYS_pwc_keys_save, 48);
    PWC_CHECK_EQ_I(PWC_SYSCALL_COUNT, 19);
}

static void test_fs_relative_validation(void) {
    static const uint8_t normal[] = "assets/icon.png";
    static const uint8_t parent[] = "../outside";
    static const uint8_t drive[] = "C:\\outside";
    static const uint8_t absolute[] = "/outside";
    PWC_CHECK_EQ_I(pwc_fs_validate_relative(nullptr, 0u), PWC_OK);
    PWC_CHECK_EQ_I(pwc_fs_validate_relative(normal, sizeof(normal) - 1u), PWC_OK);
    PWC_CHECK_EQ_I(pwc_fs_validate_relative(parent, sizeof(parent) - 1u), PWC_EPERM);
    PWC_CHECK_EQ_I(pwc_fs_validate_relative(drive, sizeof(drive) - 1u), PWC_EPERM);
    PWC_CHECK_EQ_I(pwc_fs_validate_relative(absolute, sizeof(absolute) - 1u), PWC_EPERM);
}
static void test_port_handle_lifecycle(void) {
    const pwc_boot_config config = {.handle_capacity = 2u, .worker_count = 1u};
    pwc_handle_t port = PWC_HANDLE_INVALID;
    pwc_handle_t duplicate = PWC_HANDLE_INVALID;
    pwc_waitent waitent = {.h = PWC_HANDLE_INVALID, .events = 1u, .revents = 0u};
    size_t ready = 0u;

    PWC_CHECK_EQ_I(pwc_boot(&config), PWC_OK);
    PWC_CHECK_EQ_I(pwc_ipc_port_create(&port), PWC_OK);
    waitent.h = port;
    PWC_CHECK_EQ_I(pwc_wait(&waitent, 1u, 0, &ready), PWC_OK);
    PWC_CHECK_EQ_U(ready, 1u);
    PWC_CHECK_EQ_U(waitent.revents, waitent.events);
    PWC_CHECK_EQ_I(pwc_handle_dup(port, PWC_RIGHT_READ, &duplicate), PWC_OK);
    PWC_CHECK_EQ_I(pwc_handle_dup(port, PWC_RIGHT_ALL | (1u << 7u), &duplicate), PWC_EPERM);
    PWC_CHECK_EQ_I(pwc_handle_close(port), PWC_OK);
    PWC_CHECK_EQ_I(pwc_handle_close(port), PWC_ESTALE);
    PWC_CHECK_EQ_I(pwc_ipc_port_create(&duplicate), PWC_OK);
    PWC_CHECK_EQ_U(duplicate.index, port.index);
    PWC_CHECK_EQ_I(pwc_handle_close(duplicate), PWC_OK);
    pwc_shutdown();
}
int main(void) {
    PWC_RUN(test_version_syscall);
    PWC_RUN(test_version_rejects_null);
    PWC_RUN(test_status_names);
    PWC_RUN(test_status_syscall_matches_direct_call);
    PWC_RUN(test_status_codes_are_negative);
    PWC_RUN(test_handle_encoding_roundtrips);
    PWC_RUN(test_handle_encoding_edges);
    PWC_RUN(test_invalid_handle);
    PWC_RUN(test_rights_narrow_only);
    PWC_RUN(test_syscall_numbers_are_stable);

    PWC_RUN(test_port_handle_lifecycle);
    PWC_RUN(test_fs_relative_validation);
    return pwc_test_report("packwandc-core");
}
