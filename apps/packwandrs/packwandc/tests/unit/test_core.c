/* Phase 0 unit tests: the ABI surface, status mapping, handle encoding, and
 * the two core syscalls.
 */

#include "pwc_test.h"

#include <stdatomic.h>
#include <string.h>

#include "packwandc/kernel/pwc_kernel.h"
#include "packwandc/kernel/pwc_arch_thread.h"
#include "packwandc/kernel/pwc_arena.h"
#include "packwandc/kernel/pwc_arch_wait.h"
#include "packwandc/kernel/pwc_ktrace.h"
#include "packwandc/kernel/pwc_sched.h"
#include "packwandc/kernel/pwc_slab.h"
#include "packwandc/uapi/pwc_handle.h"
#include "packwandc/uapi/pwc_status.h"
#include "packwandc/uapi/pwc_syscall.h"

static void test_arena_aligns_the_absolute_pointer(void) {
    _Alignas(16) uint8_t memory[64] = {0u};
    pwc_arena arena = {0};
    pwc_arena_init(&arena, &memory[1], sizeof(memory) - 1u);
    void *allocated = nullptr;
    PWC_CHECK_EQ_I(pwc_arena_alloc(&arena, 1u, 16u, &allocated), PWC_OK);
    PWC_CHECK_EQ_U((uintptr_t) allocated % 16u, 0u);
}
static void test_slab_rejects_double_free_and_foreign_pointers(void) {
    uint32_t memory[2] = {0u};
    uint32_t next[2] = {0u};
    uint32_t foreign = 0u;
    pwc_slab slab = {0};
    pwc_slab_init(&slab, memory, next, 2u, sizeof(memory[0]));

    void *first = nullptr;
    void *second = nullptr;
    PWC_CHECK_EQ_I(pwc_slab_alloc(&slab, &first), PWC_OK);
    PWC_CHECK_EQ_I(pwc_slab_free(&slab, first), PWC_OK);
    PWC_CHECK_EQ_I(pwc_slab_free(&slab, first), PWC_EINVAL);
    PWC_CHECK_EQ_I(pwc_slab_free(&slab, &foreign), PWC_EINVAL);
    PWC_CHECK_EQ_I(pwc_slab_alloc(&slab, &first), PWC_OK);
    PWC_CHECK_EQ_I(pwc_slab_alloc(&slab, &second), PWC_OK);
    PWC_CHECK(first != second);
}
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
    /* PWC_OK is zero and every failure is
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
     * builds with -Werror. Revisit when the floor is 19. */
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
    PWC_CHECK_EQ_I(PWC_SYS_pwc_ktrace_drain, 7);
    PWC_CHECK_EQ_I(PWC_SYS_pwc_ktrace_dropped, 8);
    PWC_CHECK_EQ_I(PWC_SYSCALL_COUNT, 27);
    PWC_CHECK_EQ_I(PWC_SYS_pwc_sh_parse, 192);
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
    /* A freshly created port has had nothing sent to it, so it is NOT ready.
     * This asserted the opposite until kernel/wait.c stopped fabricating
     * readiness -- see the header comment there for what a real answer needs. */
    PWC_CHECK_EQ_I(pwc_wait(&waitent, 1u, 0, &ready), PWC_ETIMEDOUT);
    PWC_CHECK_EQ_U(ready, 0u);
    PWC_CHECK_EQ_U(waitent.revents, 0u);

    /* A stale handle must still be rejected outright rather than timing out,
     * so callers can tell "your handle is wrong" from "nothing happened". */
    pwc_waitent stale = {.h = PWC_HANDLE_INVALID, .events = 1u, .revents = 0u};
    PWC_CHECK_EQ_I(pwc_wait(&stale, 1u, 0, &ready), PWC_EBADF);
    PWC_CHECK_EQ_I(pwc_handle_dup(port, PWC_RIGHT_READ, &duplicate), PWC_OK);
    PWC_CHECK_EQ_I(pwc_handle_dup(port, PWC_RIGHT_ALL | (1u << 7u), &duplicate), PWC_EPERM);
    PWC_CHECK_EQ_I(pwc_handle_close(port), PWC_OK);
    PWC_CHECK_EQ_I(pwc_handle_close(port), PWC_ESTALE);
    PWC_CHECK_EQ_I(pwc_ipc_port_create(&duplicate), PWC_OK);
    PWC_CHECK_EQ_U(duplicate.index, port.index);
    PWC_CHECK_EQ_I(pwc_handle_close(duplicate), PWC_OK);
    pwc_shutdown();
}
/* The last-error record.
 *
 * Worth testing precisely because it is the kind of feature that looks present
 * when it is not: a getter returning a never-written static compiles, links,
 * and reads plausibly at every call site. These assertions fail if the
 * recording side is ever removed or stops being reached. */
static void test_last_error_records_detail(void) {
    const pwc_boot_config config = {.handle_capacity = 8u, .worker_count = 1u};
    PWC_CHECK_EQ_I(pwc_boot(&config), PWC_OK);

    pwc_handle_t port = PWC_HANDLE_INVALID;
    PWC_CHECK_EQ_I(pwc_ipc_port_create(&port), PWC_OK);
    PWC_CHECK_EQ_I(pwc_handle_close(port), PWC_OK);

    /* Closing a stale handle: the generation-mismatch path, which is the one
     * failure in this layer where a silent wrong answer would be memory
     * corruption rather than an error. */
    PWC_CHECK_EQ_I(pwc_handle_close(port), PWC_ESTALE);

    const pwc_error_detail *detail = pwc_last_error();
    PWC_CHECK(detail != nullptr);
    PWC_CHECK_EQ_U(detail->struct_size, (uint32_t) sizeof(pwc_error_detail));
    PWC_CHECK_EQ_I(detail->status, PWC_ESTALE);
    PWC_CHECK_EQ_STR(detail->module, "core");
    /* A zero line means the record was default-initialised rather than
     * written by a PWC_FAIL at an actual failure site. */
    PWC_CHECK(detail->line > 0u);
    PWC_CHECK(detail->file != nullptr);
    PWC_CHECK(detail->message != nullptr);

    /* A success must not clobber the record: callers read it *after* getting a
     * failing status back, so anything in between would erase the diagnosis. */
    pwc_handle_t second = PWC_HANDLE_INVALID;
    PWC_CHECK_EQ_I(pwc_ipc_port_create(&second), PWC_OK);
    PWC_CHECK_EQ_I(pwc_last_error()->status, PWC_ESTALE);
    PWC_CHECK_EQ_I(pwc_handle_close(second), PWC_OK);

    pwc_shutdown();
}

/* The rights mask is the actionable part of an EPERM: "you asked for WRITE and
 * this handle does not have it" is a fix, "permission denied" is not. */
static void test_last_error_reports_missing_rights(void) {
    const pwc_boot_config config = {.handle_capacity = 8u, .worker_count = 1u};
    PWC_CHECK_EQ_I(pwc_boot(&config), PWC_OK);

    pwc_handle_t port = PWC_HANDLE_INVALID;
    pwc_handle_t narrowed = PWC_HANDLE_INVALID;
    PWC_CHECK_EQ_I(pwc_ipc_port_create(&port), PWC_OK);

    /* DUP and CLOSE are retained deliberately. Dropping DUP would make the
     * next call fail on the source handle's own rights check instead of on the
     * widening attempt, and dropping CLOSE would make the slot unclosable and
     * leak it for the rest of the boot. */
    const uint32_t retained = PWC_RIGHT_READ | PWC_RIGHT_DUP | PWC_RIGHT_CLOSE;
    PWC_CHECK_EQ_I(pwc_handle_dup(port, retained, &narrowed), PWC_OK);

    /* Rights only ever narrow: the narrowed handle cannot regain WRITE. */
    pwc_handle_t widened = PWC_HANDLE_INVALID;
    PWC_CHECK_EQ_I(pwc_handle_dup(narrowed, retained | PWC_RIGHT_WRITE, &widened), PWC_EPERM);

    const pwc_error_detail *detail = pwc_last_error();
    PWC_CHECK_EQ_I(detail->status, PWC_EPERM);
    /* platform_code carries the missing bits for this path -- WRITE alone, not
     * the whole requested mask. */
    PWC_CHECK_EQ_I(detail->platform_code, (int32_t) PWC_RIGHT_WRITE);

    PWC_CHECK_EQ_I(pwc_handle_close(narrowed), PWC_OK);
    PWC_CHECK_EQ_I(pwc_handle_close(port), PWC_OK);
    pwc_shutdown();
}

/* ktrace.
 *
 * The property under test is the one the ring used to get wrong: a drop must
 * not consume a sequence number. The old implementation reserved the sequence
 * before checking for space, so a dropped record left a slot the reader would
 * later hand out anyway, filled with data from a full lap earlier. A reader
 * could not tell that from a real record. */
/* Drain whatever is currently in the ring, returning how much came out.
 *
 * Boot is no longer silent: module bring-up and "kernel booted" emit INFO
 * notes, so a test that wants to observe its own record has to clear those
 * first. */
static uint32_t pwc_test_drain_ring(void) {
    uint32_t count = 0u;
    pwc_trace_record scratch = {0};
    while (pwc_ktrace_drain(&scratch) == PWC_OK) {
        ++count;
    }
    return count;
}

static void test_ktrace_records_failures(void) {
    const pwc_boot_config config = {.handle_capacity = 8u, .worker_count = 1u};
    PWC_CHECK_EQ_I(pwc_boot(&config), PWC_OK);

    /* Boot traces itself: one note per module initialised, plus one for the
     * kernel. Asserting a non-zero count here is what keeps module bring-up
     * observable rather than silent ceremony. */
    const uint32_t boot_notes = pwc_test_drain_ring();
    PWC_CHECK(boot_notes > 0u);

    pwc_trace_record record = {0};
    PWC_CHECK_EQ_I(pwc_ktrace_drain(&record), PWC_EAGAIN);

    /* Any recorded failure must also be traced -- the detail record and the
     * ring deliberately share one choke point. */
    pwc_handle_t bogus = {.index = 4000u, .generation = 7u};
    PWC_CHECK_EQ_I(pwc_handle_close(bogus), PWC_EBADF);

    PWC_CHECK_EQ_I(pwc_ktrace_drain(&record), PWC_OK);
    PWC_CHECK_EQ_U(record.struct_size, (uint32_t) sizeof(pwc_trace_record));
    PWC_CHECK_EQ_I(record.status, PWC_EBADF);
    PWC_CHECK_EQ_U(record.level, PWC_TRACE_LEVEL_ERROR);
    PWC_CHECK_EQ_STR(record.module, "core");
    PWC_CHECK(record.line > 0u);
    PWC_CHECK(record.message != nullptr);

    /* Drained to empty again. */
    PWC_CHECK_EQ_I(pwc_ktrace_drain(&record), PWC_EAGAIN);

    uint64_t dropped = 0u;
    PWC_CHECK_EQ_I(pwc_ktrace_dropped(&dropped), PWC_OK);
    PWC_CHECK(dropped == 0u);

    pwc_shutdown();
}

static void test_ktrace_overflow_drops_without_holes(void) {
    const pwc_boot_config config = {.handle_capacity = 8u, .worker_count = 1u};
    PWC_CHECK_EQ_I(pwc_boot(&config), PWC_OK);

    /* Clear boot's own notes so the arithmetic below is about this test's
     * records only. `next_sequence` is where the ring's numbering has reached,
     * since sequences are monotonic across the whole boot, not per-drain. */
    const uint64_t next_sequence = (uint64_t) pwc_test_drain_ring();

    /* Overfill: more failures than the ring can hold. */
    const uint32_t excess = 8u;
    pwc_handle_t bogus = {.index = 4000u, .generation = 7u};
    for (uint32_t i = 0u; i < (uint32_t) PWC_KTRACE_CAPACITY + excess; ++i) {
        PWC_CHECK_EQ_I(pwc_handle_close(bogus), PWC_EBADF);
    }

    uint64_t dropped = 0u;
    PWC_CHECK_EQ_I(pwc_ktrace_dropped(&dropped), PWC_OK);
    PWC_CHECK(dropped == (uint64_t) excess);

    /* Exactly CAPACITY records must come out, every one of them real, and the
     * sequences must be gapless. A hole left by a drop would surface here as
     * either a jump in `sequence` or a record with a zeroed struct_size. */
    uint64_t expected_sequence = next_sequence;
    uint32_t drained = 0u;
    pwc_trace_record record = {0};
    while (pwc_ktrace_drain(&record) == PWC_OK) {
        PWC_CHECK_EQ_U(record.struct_size, (uint32_t) sizeof(pwc_trace_record));
        PWC_CHECK(record.sequence == expected_sequence);
        PWC_CHECK_EQ_I(record.status, PWC_EBADF);
        ++expected_sequence;
        ++drained;
    }
    PWC_CHECK_EQ_U(drained, (uint32_t) PWC_KTRACE_CAPACITY);

    /* The ring must be reusable after being drained, not wedged by the wrap. */
    PWC_CHECK_EQ_I(pwc_handle_close(bogus), PWC_EBADF);
    PWC_CHECK_EQ_I(pwc_ktrace_drain(&record), PWC_OK);

    pwc_shutdown();
}

/* Module registration.
 *
 * Boot must actually drive kernel/module.c rather than the modules being
 * linked in and hoped for. Each module's init emits an INFO note, so the trace
 * is the evidence that the registry ran -- if boot stopped calling
 * pwc_modules_init, these notes would simply not be there. */
static void test_boot_initialises_every_module(void) {
    const pwc_boot_config config = {.handle_capacity = 8u, .worker_count = 1u};
    PWC_CHECK_EQ_I(pwc_boot(&config), PWC_OK);

    bool saw_pwfs = false;
    bool saw_pwproc = false;
    bool saw_pwkeys = false;
    bool saw_core = false;

    pwc_trace_record record = {0};
    while (pwc_ktrace_drain(&record) == PWC_OK) {
        PWC_CHECK_EQ_U(record.level, PWC_TRACE_LEVEL_INFO);
        /* Notes are not failures and must never masquerade as one. */
        PWC_CHECK_EQ_I(record.status, PWC_OK);
        if (strcmp(record.module, "pwfs") == 0) {
            saw_pwfs = true;
        } else if (strcmp(record.module, "pwproc") == 0) {
            saw_pwproc = true;
        } else if (strcmp(record.module, "pwkeys") == 0) {
            saw_pwkeys = true;
        } else if (strcmp(record.module, "core") == 0) {
            saw_core = true;
        }
    }

    PWC_CHECK(saw_pwfs);
    PWC_CHECK(saw_pwproc);
    PWC_CHECK(saw_pwkeys);
    PWC_CHECK(saw_core);

    /* Booting twice is refused rather than silently re-running module init. */
    PWC_CHECK_EQ_I(pwc_boot(&config), PWC_EAGAIN);

    pwc_shutdown();

    /* Shutdown is idempotent: a second call must not run exit hooks again. */
    pwc_shutdown();
}

/* pwipc framed messages.
 *
 * The guarantee is that a reader gets back exactly the messages that were
 * sent, with their boundaries intact -- not a byte stream it has to
 * re-delimit. Message sizes below are deliberately not multiples of anything,
 * so a framing bug shows up as a shifted boundary rather than lining up by
 * luck. */
static void test_ipc_frames_round_trip(void) {
    const pwc_boot_config config = {.handle_capacity = 8u, .worker_count = 1u};
    PWC_CHECK_EQ_I(pwc_boot(&config), PWC_OK);

    pwc_handle_t port = PWC_HANDLE_INVALID;
    PWC_CHECK_EQ_I(pwc_ipc_port_create(&port), PWC_OK);

    uint8_t buffer[256] = {0};
    size_t length = 0u;

    /* Empty port: EAGAIN, not a zero-length message. */
    PWC_CHECK_EQ_I(pwc_ipc_recv(port, buffer, sizeof(buffer), &length), PWC_EAGAIN);

    static const uint8_t first[] = "pack list --side client";
    static const uint8_t second[] = "mod add sodium";
    PWC_CHECK_EQ_I(pwc_ipc_send(port, first, sizeof(first) - 1u), PWC_OK);
    PWC_CHECK_EQ_I(pwc_ipc_send(port, second, sizeof(second) - 1u), PWC_OK);

    /* FIFO, and each message comes back whole. */
    PWC_CHECK_EQ_I(pwc_ipc_recv(port, buffer, sizeof(buffer), &length), PWC_OK);
    PWC_CHECK_EQ_U(length, sizeof(first) - 1u);
    PWC_CHECK(memcmp(buffer, first, length) == 0);

    PWC_CHECK_EQ_I(pwc_ipc_recv(port, buffer, sizeof(buffer), &length), PWC_OK);
    PWC_CHECK_EQ_U(length, sizeof(second) - 1u);
    PWC_CHECK(memcmp(buffer, second, length) == 0);

    PWC_CHECK_EQ_I(pwc_ipc_recv(port, buffer, sizeof(buffer), &length), PWC_EAGAIN);

    /* A zero-length message is a real message, distinct from "nothing queued". */
    PWC_CHECK_EQ_I(pwc_ipc_send(port, first, 0u), PWC_OK);
    PWC_CHECK_EQ_I(pwc_ipc_recv(port, buffer, sizeof(buffer), &length), PWC_OK);
    PWC_CHECK_EQ_U(length, 0u);

    /* Too small a buffer must not consume the frame -- the caller can retry. */
    PWC_CHECK_EQ_I(pwc_ipc_send(port, first, sizeof(first) - 1u), PWC_OK);
    uint8_t tiny[4] = {0};
    PWC_CHECK_EQ_I(pwc_ipc_recv(port, tiny, sizeof(tiny), &length), PWC_EOVERFLOW);
    PWC_CHECK_EQ_U(length, sizeof(first) - 1u);
    PWC_CHECK_EQ_I(pwc_ipc_recv(port, buffer, sizeof(buffer), &length), PWC_OK);
    PWC_CHECK_EQ_U(length, sizeof(first) - 1u);

    PWC_CHECK_EQ_I(pwc_ipc_port_close(port), PWC_OK);
    /* The handle is dead with the slot, so a second close is a stale handle. */
    PWC_CHECK_EQ_I(pwc_ipc_port_close(port), PWC_ESTALE);

    pwc_shutdown();
}

/* Drives the ring several times past its capacity, so every message after the
 * first lap is written across the wrap point. A wrap bug here corrupts the
 * payload or the length prefix; both surface as a mismatch below. */
static void test_ipc_ring_wraps_cleanly(void) {
    const pwc_boot_config config = {.handle_capacity = 8u, .worker_count = 1u};
    PWC_CHECK_EQ_I(pwc_boot(&config), PWC_OK);

    pwc_handle_t port = PWC_HANDLE_INVALID;
    PWC_CHECK_EQ_I(pwc_ipc_port_create(&port), PWC_OK);

    /* 300 bytes plus a 4-byte header does not divide the ring, so successive
     * laps land at different offsets and exercise every split position. */
    enum { PWC_TEST_MESSAGE = 300, PWC_TEST_ROUNDS = 200 };
    uint8_t sent[PWC_TEST_MESSAGE] = {0};
    uint8_t received[PWC_TEST_MESSAGE] = {0};
    size_t length = 0u;

    for (uint32_t round = 0u; round < (uint32_t) PWC_TEST_ROUNDS; ++round) {
        for (size_t i = 0u; i < (size_t) PWC_TEST_MESSAGE; ++i) {
            sent[i] = (uint8_t) ((i + round) & 0xffu);
        }
        PWC_CHECK_EQ_I(pwc_ipc_send(port, sent, sizeof(sent)), PWC_OK);
        PWC_CHECK_EQ_I(pwc_ipc_recv(port, received, sizeof(received), &length), PWC_OK);
        PWC_CHECK_EQ_U(length, (size_t) PWC_TEST_MESSAGE);
        PWC_CHECK(memcmp(sent, received, length) == 0);
    }

    /* Filling the ring reports back-pressure rather than silently overwriting
     * messages the reader has not taken yet. */
    pwc_status fill = PWC_OK;
    uint32_t queued = 0u;
    while (fill == PWC_OK) {
        fill = pwc_ipc_send(port, sent, sizeof(sent));
        if (fill == PWC_OK) {
            ++queued;
        }
    }
    PWC_CHECK_EQ_I(fill, PWC_EOVERFLOW);
    PWC_CHECK(queued > 0u);

    /* Everything queued before the refusal is still intact and readable. */
    for (uint32_t i = 0u; i < queued; ++i) {
        PWC_CHECK_EQ_I(pwc_ipc_recv(port, received, sizeof(received), &length), PWC_OK);
        PWC_CHECK_EQ_U(length, (size_t) PWC_TEST_MESSAGE);
    }
    PWC_CHECK_EQ_I(pwc_ipc_recv(port, received, sizeof(received), &length), PWC_EAGAIN);

    PWC_CHECK_EQ_I(pwc_ipc_port_close(port), PWC_OK);
    pwc_shutdown();
}

/* --- pw4shell ----------------------------------------------------------- */

/* Parse `text` and assert it produced exactly the expected words. */
static void pwsh_expect(const char *text, uint32_t argc, const char *const *words) {
    pwc_sh_command cmd = {0};
    PWC_CHECK_EQ_I(pwc_sh_parse((const uint8_t *) text, strlen(text), &cmd), PWC_OK);
    PWC_CHECK_EQ_U(cmd.argc, argc);
    for (uint32_t i = 0u; i < argc && i < cmd.argc; ++i) {
        PWC_CHECK_EQ_U(cmd.arglen[i], (uint32_t) strlen(words[i]));
        PWC_CHECK_EQ_STR((const char *) cmd.argv[i], words[i]);
    }
}

static void pwsh_expect_rejected(const char *text, pwc_status expected) {
    pwc_sh_command cmd = {0};
    PWC_CHECK_EQ_I(pwc_sh_parse((const uint8_t *) text, strlen(text), &cmd), expected);
}

static void test_sh_parses_the_grammar(void) {
    static const char *const simple[] = {"pack", "list", "--side", "client"};
    pwsh_expect("pack list --side client", 4u, simple);

    /* Leading, trailing and repeated whitespace are all insignificant. */
    pwsh_expect("   pack   list   --side   client   ", 4u, simple);

    static const char *const quoted[] = {"echo", "hello world", "second"};
    pwsh_expect("echo \"hello world\" second", 3u, quoted);

    /* Single quotes are fully literal, so Windows paths need no doubling. */
    static const char *const literal[] = {"echo", "..\\dir\\file"};
    pwsh_expect("echo '..\\dir\\file'", 2u, literal);

    /* Escapes inside double quotes only. */
    static const char *const escaped[] = {"echo", "a\tb\"c"};
    pwsh_expect("echo \"a\\tb\\\"c\"", 2u, escaped);

    /* Comments run to end of line, and attach without needing a space. */
    static const char *const commented[] = {"version"};
    pwsh_expect("version # explain yourself", 1u, commented);
    pwsh_expect("version# no space", 1u, commented);

    /* A blank or comment-only line is valid and yields nothing to run. */
    pwsh_expect("", 0u, nullptr);
    pwsh_expect("   ", 0u, nullptr);
    pwsh_expect("# just a comment", 0u, nullptr);

    /* An empty quoted string is a real, empty argument -- not nothing. */
    static const char *const empty_arg[] = {"echo", ""};
    pwsh_expect("echo \"\"", 2u, empty_arg);
}

static void test_sh_rejects_malformed_input(void) {
    /* Every one of these is refused whole rather than truncated: a tokeniser
     * that silently drops a tail hands the dispatcher a command that looks
     * well-formed and means something else. */
    pwsh_expect_rejected("echo \"unterminated", PWC_EINVAL);
    pwsh_expect_rejected("echo 'unterminated", PWC_EINVAL);
    pwsh_expect_rejected("echo \"trailing backslash \\", PWC_EINVAL);
    pwsh_expect_rejected("echo \"bad \\q escape\"", PWC_EINVAL);
    pwsh_expect_rejected("one\ntwo", PWC_EINVAL);

    /* Too many words. */
    pwsh_expect_rejected("a b c d e f g h i j k l m n o p q r", PWC_EOVERFLOW);

    /* A word longer than PWC_SH_MAX_ARG. */
    char oversized[PWC_SH_MAX_ARG + 32] = {0};
    memset(oversized, 'x', sizeof(oversized) - 1u);
    pwsh_expect_rejected(oversized, PWC_EOVERFLOW);

    /* A line longer than PWC_SH_MAX_LINE. */
    pwc_sh_command cmd = {0};
    static uint8_t long_line[PWC_SH_MAX_LINE + 8] = {0};
    memset(long_line, 'a', sizeof(long_line));
    PWC_CHECK_EQ_I(pwc_sh_parse(long_line, sizeof(long_line), &cmd), PWC_EOVERFLOW);

    /* An embedded NUL would truncate the word for any C consumer while the
     * length field still claimed the original size. */
    static const uint8_t embedded[] = {'e', 'c', 'h', 'o', ' ', 'a', 0u, 'b'};
    PWC_CHECK_EQ_I(pwc_sh_parse(embedded, sizeof(embedded), &cmd), PWC_EINVAL);
}

static void test_sh_executes_builtins(void) {
    const pwc_boot_config config = {.handle_capacity = 8u, .worker_count = 1u};
    PWC_CHECK_EQ_I(pwc_boot(&config), PWC_OK);

    pwc_handle_t port = PWC_HANDLE_INVALID;
    PWC_CHECK_EQ_I(pwc_ipc_port_create(&port), PWC_OK);

    pwc_sh_command cmd = {0};
    uint8_t buffer[512] = {0};
    size_t length = 0u;

    static const char echo_line[] = "echo hello world";
    PWC_CHECK_EQ_I(pwc_sh_exec(port, (const uint8_t *) echo_line, sizeof(echo_line) - 1u, &cmd), PWC_OK);
    PWC_CHECK_EQ_I(pwc_ipc_recv(port, buffer, sizeof(buffer), &length), PWC_OK);
    PWC_CHECK_EQ_U(length, strlen("hello world"));
    PWC_CHECK(memcmp(buffer, "hello world", length) == 0);

    /* status decodes a code through the kernel's own tables. */
    static const char status_line[] = "status -7";
    PWC_CHECK_EQ_I(pwc_sh_exec(port, (const uint8_t *) status_line, sizeof(status_line) - 1u, &cmd), PWC_OK);
    PWC_CHECK_EQ_I(pwc_ipc_recv(port, buffer, sizeof(buffer), &length), PWC_OK);
    PWC_CHECK(memcmp(buffer, "PWC_ESTALE", strlen("PWC_ESTALE")) == 0);

    /* An unknown verb is PWC_ENOSYS, NOT a parse error -- the caller still
     * gets the parsed argv so the host can dispatch pack/mod/diag itself. */
    static const char host_line[] = "pack list --side client";
    PWC_CHECK_EQ_I(pwc_sh_exec(port, (const uint8_t *) host_line, sizeof(host_line) - 1u, &cmd), PWC_ENOSYS);
    PWC_CHECK_EQ_U(cmd.argc, 4u);
    PWC_CHECK_EQ_STR((const char *) cmd.argv[0], "pack");
    PWC_CHECK_EQ_STR((const char *) cmd.argv[3], "client");

    /* A malformed line fails at the parse stage and never reaches dispatch. */
    static const char bad_line[] = "echo \"unterminated";
    PWC_CHECK_EQ_I(pwc_sh_exec(port, (const uint8_t *) bad_line, sizeof(bad_line) - 1u, &cmd), PWC_EINVAL);

    /* Pressing enter on an empty prompt is a no-op, not an error. */
    PWC_CHECK_EQ_I(pwc_sh_exec(port, (const uint8_t *) "", 0u, &cmd), PWC_OK);
    PWC_CHECK_EQ_U(cmd.argc, 0u);

    /* An invalid port discards output rather than failing the command. */
    PWC_CHECK_EQ_I(pwc_sh_exec(PWC_HANDLE_INVALID, (const uint8_t *) echo_line, sizeof(echo_line) - 1u, &cmd),
                   PWC_OK);

    PWC_CHECK_EQ_I(pwc_ipc_port_close(port), PWC_OK);
    pwc_shutdown();
}

/* --- scheduler ---------------------------------------------------------- */

/* Shared counter for the pool test. Atomic because the whole point is that
 * several workers touch it at once -- a plain int here would be the exact data
 * race the pool is supposed to make safe to have. */
static atomic_uint_fast32_t pwc_test_ran;

static void pwc_test_increment(void *arg) {
    (void) arg;
    (void) atomic_fetch_add_explicit(&pwc_test_ran, 1u, memory_order_relaxed);
}

static void test_sched_runs_queued_work(void) {
    pwc_sched sched = {0};
    atomic_init(&pwc_test_ran, 0u);

    PWC_CHECK_EQ_I(pwc_sched_init(&sched, 4u), PWC_OK);

    enum { PWC_TEST_JOBS = 200 };
    for (uint32_t i = 0u; i < (uint32_t) PWC_TEST_JOBS; ++i) {
        PWC_CHECK_EQ_I(pwc_sched_submit(&sched, pwc_test_increment, nullptr), PWC_OK);
    }

    /* Shutdown drains the queue before joining, so every submitted job must
     * have run by the time it returns. Anything less would mean shutdown
     * silently discards work that was already accepted. */
    pwc_sched_shutdown(&sched);
    PWC_CHECK_EQ_U((uint32_t) atomic_load(&pwc_test_ran), (uint32_t) PWC_TEST_JOBS);

    /* Idempotent: a second shutdown must not double-join or fault. */
    pwc_sched_shutdown(&sched);
}

static void test_sched_rejects_after_shutdown(void) {
    pwc_sched sched = {0};
    atomic_init(&pwc_test_ran, 0u);

    PWC_CHECK_EQ_I(pwc_sched_init(&sched, 2u), PWC_OK);
    pwc_sched_shutdown(&sched);

    /* Accepting work nothing will ever run is worse than refusing it. */
    PWC_CHECK_EQ_I(pwc_sched_submit(&sched, pwc_test_increment, nullptr), PWC_ECANCELED);
    PWC_CHECK_EQ_U((uint32_t) atomic_load(&pwc_test_ran), 0u);

    /* And a scheduler that never started must not fault on either call. */
    pwc_sched fresh = {0};
    PWC_CHECK_EQ_I(pwc_sched_submit(&fresh, pwc_test_increment, nullptr), PWC_ECANCELED);
    pwc_sched_shutdown(&fresh);
}

static void test_sched_validates_arguments(void) {
    pwc_sched sched = {0};
    PWC_CHECK_EQ_I(pwc_sched_init(nullptr, 4u), PWC_EINVAL);
    PWC_CHECK_EQ_I(pwc_sched_init(&sched, 0u), PWC_EINVAL);
    PWC_CHECK_EQ_I(pwc_sched_init(&sched, (uint32_t) PWC_SCHED_MAX_WORKERS + 1u), PWC_EINVAL);

    PWC_CHECK_EQ_I(pwc_sched_init(&sched, 1u), PWC_OK);
    PWC_CHECK_EQ_I(pwc_sched_submit(&sched, nullptr, nullptr), PWC_EINVAL);
    pwc_sched_shutdown(&sched);
}

/* A poller occupies its thread until whatever it waits on is released. This
 * stands in for a filesystem watch: it blocks on a flag the test clears, which
 * is the same shape as blocking on a handle the owner closes. */
static atomic_bool pwc_test_poller_stop;
static atomic_uint_fast32_t pwc_test_poller_ticks;

static void pwc_test_poller(void *arg) {
    (void) arg;
    while (!atomic_load_explicit(&pwc_test_poller_stop, memory_order_acquire)) {
        (void) atomic_fetch_add_explicit(&pwc_test_poller_ticks, 1u, memory_order_relaxed);
        (void) pwc_arch_wait_timeout(1);
    }
}

static void test_sched_joins_dedicated_pollers(void) {
    pwc_sched sched = {0};
    atomic_init(&pwc_test_poller_stop, false);
    atomic_init(&pwc_test_poller_ticks, 0u);

    PWC_CHECK_EQ_I(pwc_sched_init(&sched, 2u), PWC_OK);
    PWC_CHECK_EQ_I(pwc_sched_spawn_poller(&sched, pwc_test_poller, nullptr), PWC_OK);

    /* Let it actually get going, so the join below is a real join rather than
     * one against a thread that already returned. */
    while (atomic_load(&pwc_test_poller_ticks) == 0u) {
        (void) pwc_arch_wait_timeout(1);
    }

    /* Release the poller BEFORE shutdown. This is the contract
     * pwc_sched_spawn_poller documents: shutdown joins pollers, so one that
     * cannot be unblocked hangs it forever. Getting this wrong is how a
     * process ends up unkillable on exit. */
    atomic_store_explicit(&pwc_test_poller_stop, true, memory_order_release);
    pwc_sched_shutdown(&sched);
}

int main(void) {
    PWC_RUN(test_arena_aligns_the_absolute_pointer);
    PWC_RUN(test_slab_rejects_double_free_and_foreign_pointers);
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
    PWC_RUN(test_last_error_records_detail);
    PWC_RUN(test_last_error_reports_missing_rights);
    PWC_RUN(test_ktrace_records_failures);
    PWC_RUN(test_ktrace_overflow_drops_without_holes);
    PWC_RUN(test_boot_initialises_every_module);
    PWC_RUN(test_ipc_frames_round_trip);
    PWC_RUN(test_ipc_ring_wraps_cleanly);
    PWC_RUN(test_sh_parses_the_grammar);
    PWC_RUN(test_sh_rejects_malformed_input);
    PWC_RUN(test_sh_executes_builtins);
    PWC_RUN(test_sched_validates_arguments);
    PWC_RUN(test_sched_runs_queued_work);
    PWC_RUN(test_sched_rejects_after_shutdown);
    PWC_RUN(test_sched_joins_dedicated_pollers);
    return pwc_test_report("packwandc-core");
}
