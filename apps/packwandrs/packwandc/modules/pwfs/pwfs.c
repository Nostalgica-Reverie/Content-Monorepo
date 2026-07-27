#include "packwandc/uapi/pwc_syscall.h"
#include "packwandc/kernel/pwc_arch_fs.h"
#include "packwandc/kernel/pwc_arch_wait.h"
#include "packwandc/kernel/pwc_ipc.h"
#include "packwandc/kernel/pwc_sched.h"
#include "packwandc/kernel/pwc_boot_internal.h"
#include "packwandc/kernel/pwc_error.h"
#include "packwandc/kernel/pwc_module_registry.h"

#include <stdatomic.h>

enum {
    /* Concurrent watch streams. One per open pack watch; a workbench watches
     * the selected pack and little else. */
    PWC_FS_MAX_STREAMS = 8,
    /* Idle sleep for a non-blocking backend, in milliseconds. Long enough that
     * the thread costs nothing, short enough that a save feels immediate. */
    PWC_FS_STREAM_IDLE_MS = 50,
};

static bool pwc_fs_separator(uint8_t value) { return value == (uint8_t) '/' || value == (uint8_t) '\\'; }

pwc_status pwc_fs_validate_relative(const uint8_t *path, size_t path_len) {
    if (path == nullptr && path_len != 0u) {
        return PWC_EINVAL;
    }
    if (path_len == 0u) {
        return PWC_OK;
    }
    if (pwc_fs_separator(path[0]) || (path_len >= 2u && path[1] == (uint8_t) ':')) {
        return PWC_EPERM;
    }
    size_t component_start = 0u;
    for (size_t index = 0u; index <= path_len; ++index) {
        if (index == path_len || pwc_fs_separator(path[index])) {
            const size_t length = index - component_start;
            if (length == 0u || (length == 1u && path[component_start] == (uint8_t) '.') ||
                (length == 2u && path[component_start] == (uint8_t) '.' &&
                 path[component_start + 1u] == (uint8_t) '.')) {
                return PWC_EPERM;
            }
            component_start = index + 1u;
        }
    }
    return PWC_OK;
}

pwc_status pwc_fs_read(const uint8_t *root,
                       size_t root_len,
                       const uint8_t *path,
                       size_t path_len,
                       uint8_t *buffer,
                       size_t capacity,
                       size_t *out_len) {
    if (root == nullptr || root_len == 0u || buffer == nullptr || capacity == 0u || out_len == nullptr) {
        return PWC_EINVAL;
    }
    PWC_TRY(pwc_fs_validate_relative(path, path_len));
    return pwc_arch_fs_read(root, root_len, path, path_len, buffer, capacity, out_len);
}

pwc_status pwc_fs_atomic_write(const uint8_t *root,
                               size_t root_len,
                               const uint8_t *path,
                               size_t path_len,
                               const uint8_t *content,
                               size_t content_len) {
    if (root == nullptr || root_len == 0u || content == nullptr) {
        return PWC_EINVAL;
    }
    PWC_TRY(pwc_fs_validate_relative(path, path_len));
    return pwc_arch_fs_atomic_write(root, root_len, path, path_len, content, content_len);
}

pwc_status pwc_fs_watch_open(const uint8_t *root, size_t root_len, pwc_handle_t *out) {
    if (root == nullptr || root_len == 0u || out == nullptr) {
        return PWC_EINVAL;
    }
    pwc_handle_table *const handles = pwc_kernel_handles();
    if (handles == nullptr) {
        return PWC_ECANCELED;
    }
    uintptr_t native = 0u;
    PWC_TRY(pwc_arch_fs_watch_open(root, root_len, &native));
    const pwc_status opened =
        pwc_handle_open(handles, PWC_OBJECT_FS_WATCH, PWC_RIGHT_WAIT | PWC_RIGHT_CLOSE, out);
    if (opened != PWC_OK) {
        (void) pwc_arch_fs_watch_close(native);
        return opened;
    }
    return pwc_handle_payload_set(handles, *out, PWC_OBJECT_FS_WATCH, native);
}

pwc_status pwc_fs_watch_read(pwc_handle_t watch, size_t *out_events) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    if (handles == nullptr || out_events == nullptr) {
        return PWC_EINVAL;
    }
    uintptr_t native = 0u;
    PWC_TRY(pwc_handle_payload_get(handles, watch, PWC_OBJECT_FS_WATCH, &native));
    return pwc_arch_fs_watch_read(native, out_events);
}

pwc_status pwc_fs_watch_close(pwc_handle_t watch) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    if (handles == nullptr) {
        return PWC_ECANCELED;
    }
    uintptr_t native = 0u;
    PWC_TRY(pwc_handle_payload_get(handles, watch, PWC_OBJECT_FS_WATCH, &native));
    const pwc_status closed_native = pwc_arch_fs_watch_close(native);
    const pwc_status closed_handle = pwc_handle_close_table(handles, watch);
    return closed_native != PWC_OK ? closed_native : closed_handle;
}

/* --- watch streaming (packwandc.md 5.3) ---------------------------------
 *
 * This is what removes the UI's polling loop, which is the phase 2 criterion
 * `pwc_fs_watch_read` alone could never meet: that call *blocks* until a change
 * arrives, so a caller either dedicates a thread to it or polls. Polling is
 * what the UI did.
 *
 * A dedicated poller thread does the blocking read and publishes each settled
 * batch as a framed message on a port, so the Rust side just drains a port.
 * Pollers exist for exactly this shape of work (packwandc.md 3.6).
 *
 * CANCELLATION
 *
 * The poller is blocked inside the OS and cannot be woken by any of the
 * kernel's own primitives. Closing the watch is what unblocks it: the platform
 * read fails, the loop sees the stream is closed, and the thread returns.
 * pwc_sched_shutdown then joins it. That is why kernel/boot.c shuts modules
 * down *before* the scheduler -- reversing the two hangs shutdown forever on a
 * poller nothing has released.
 */

/* One live stream. Fixed table for the same reason as everything else here:
 * there is no allocator (packwandc.md 3.4). */
typedef struct pwc_fs_stream {
    uintptr_t native;
    pwc_handle_t port;
    atomic_bool active;
} pwc_fs_stream;

static struct {
    pwc_fs_stream slots[PWC_FS_MAX_STREAMS];
} pwc_fs_streams;

static void pwc_fs_stream_poller(void *arg) {
    pwc_fs_stream *const stream = (pwc_fs_stream *) arg;

    while (atomic_load_explicit(&stream->active, memory_order_acquire)) {
        size_t events = 0u;
        const pwc_status status = pwc_arch_fs_watch_read(stream->native, &events);

        if (status == PWC_EAGAIN) {
            /* Non-blocking backend with nothing pending (Linux). Sleep briefly
             * rather than spin: this thread exists to be idle most of the time. */
            (void) pwc_arch_wait_timeout(PWC_FS_STREAM_IDLE_MS);
            continue;
        }
        if (status != PWC_OK) {
            /* The watch was closed underneath us, which is how cancellation is
             * signalled -- not an error worth recording on every teardown. */
            break;
        }
        if (events == 0u) {
            continue;
        }

        /* The payload is the coalesced count, little-endian. The arch layer
         * reports how many changes settled, not which files: giving the UI a
         * "something under this root changed, N times" signal is what it acts
         * on, and per-path reporting needs the wd-to-path map packwandc.md 5.3
         * records as outstanding. */
        const uint32_t count = events > UINT32_MAX ? UINT32_MAX : (uint32_t) events;
        const uint8_t message[4] = {
            (uint8_t) (count & 0xffu),
            (uint8_t) ((count >> 8u) & 0xffu),
            (uint8_t) ((count >> 16u) & 0xffu),
            (uint8_t) ((count >> 24u) & 0xffu),
        };
        /* Discarded deliberately: a full port means the UI is behind, and
         * dropping a change notification is recoverable (the next one still
         * arrives) whereas blocking this thread is not. */
        (void) pwc_ipc_send(stream->port, message, sizeof(message));
    }

    atomic_store_explicit(&stream->active, false, memory_order_release);
}

pwc_status pwc_fs_watch_stream(pwc_handle_t watch, pwc_handle_t port) {
    pwc_handle_table *const handles = pwc_kernel_handles();
    pwc_sched *const sched = pwc_kernel_sched();
    if (handles == nullptr || sched == nullptr) {
        return PWC_FAIL(PWC_ECANCELED, "pwfs", "the kernel is not booted");
    }

    uintptr_t native = 0u;
    PWC_TRY(pwc_handle_payload_get(handles, watch, PWC_OBJECT_FS_WATCH, &native));
    /* The port must be writable by us before a thread starts depending on it. */
    PWC_TRY(pwc_handle_validate(handles, port, PWC_RIGHT_WRITE));

    for (size_t i = 0u; i < (size_t) PWC_FS_MAX_STREAMS; ++i) {
        bool expected = false;
        /* The compare-exchange is the claim: two concurrent stream requests
         * cannot take the same slot, and a slot is never handed out while its
         * previous poller is still winding down. */
        if (!atomic_compare_exchange_strong_explicit(&pwc_fs_streams.slots[i].active,
                                                     &expected,
                                                     true,
                                                     memory_order_acq_rel,
                                                     memory_order_relaxed)) {
            continue;
        }
        pwc_fs_streams.slots[i].native = native;
        pwc_fs_streams.slots[i].port = port;

        const pwc_status spawned =
            pwc_sched_spawn_poller(sched, pwc_fs_stream_poller, &pwc_fs_streams.slots[i]);
        if (spawned != PWC_OK) {
            atomic_store_explicit(&pwc_fs_streams.slots[i].active, false, memory_order_release);
            return spawned;
        }
        return PWC_OK;
    }

    return PWC_FAIL_PLATFORM(PWC_ENOMEM, "pwfs", "no free watch stream slots", PWC_FS_MAX_STREAMS);
}

/* --- module descriptor (packwandc.md 3.5) ------------------------------- */

static pwc_status pwc_pwfs_init(pwc_module_ctx *ctx) {
    /* No state to build: pwfs is stateless, and every object it hands out
     * lives in the kernel handle table rather than in the module. The
     * descriptor still earns its place -- it puts pwfs in the boot order and
     * gives it a teardown hook for when that stops being true. */
    ctx->state = nullptr;
    PWC_NOTE(
        PWC_TRACE_LEVEL_INFO, "pwfs", "module initialised: rooted filesystem access and recursive watching");
    return PWC_OK;
}

static void pwc_pwfs_exit(pwc_module_ctx *ctx) {
    ctx->state = nullptr;
    PWC_NOTE(PWC_TRACE_LEVEL_INFO, "pwfs", "module shut down");
}

const pwc_module pwc_module_pwfs = {
    .name = "pwfs",
    .abi_version = PWC_ABI_VERSION_MAJOR,
    .depends = nullptr,
    .init = pwc_pwfs_init,
    .exit = pwc_pwfs_exit,
};
