//! Raw FFI bindings to the packwandc native core.
//!
//! This is the application's only `unsafe` boundary.
//!
//! The generated bindings stay raw; the SDK owns validation and ergonomics.
//!
//! The safe SDK keeps the unsafe surface concentrated in this crate.
//! The raw layer contains no error handling or owned types; those live above it.

#![no_std]

mod bindings;

pub use bindings::*;

/// A kernel object handle with a slot index and generation counter.
/// Stale handles resolve to [`PWC_ESTALE`] after their slot is reused.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PwcHandle {
    /// Table slot. Index 0 is never handed out, so a zeroed handle is invalid
    /// by construction.
    pub index: u32,
    /// Incremented every time the slot is freed.
    pub generation: u32,
}

/// Details for the calling thread's most recent native failure.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PwcErrorDetail {
    /// `size_of::<PwcErrorDetail>()` as the core saw it, for forward compat.
    pub struct_size: u32,
    /// The status this record was recorded for.
    pub status: i32,
    /// `GetLastError()`/`errno`/D-Bus code, or 0 when there was none.
    pub platform_code: i32,
    /// `__LINE__` of the recording site.
    pub line: u32,
    /// Static, never NULL: `"core"`, `"pwfs"`, `"arch/win32"`.
    pub module: *const core::ffi::c_char,
    /// Static, never NULL.
    pub message: *const core::ffi::c_char,
    /// Static, never NULL. Already repo-relative via `-ffile-prefix-map`.
    pub file: *const core::ffi::c_char,
}

/// One record drained from the kernel trace ring.
///
/// Layout is part of the wire ABI and is asserted in C (`uapi/pwc_trace.h`)
/// and in this crate's tests.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PwcTraceRecord {
    /// `size_of::<PwcTraceRecord>()` as the core saw it, for forward compat.
    pub struct_size: u32,
    /// One of the `PWC_TRACE_LEVEL_*` constants.
    pub level: u32,
    /// Monotonic across the ring's lifetime; a gap means records were dropped.
    pub sequence: u64,
    /// The status being reported, or `PWC_OK`.
    pub status: i32,
    /// `GetLastError()`/`errno`/D-Bus code, or 0 when there was none.
    pub platform_code: i32,
    /// Source line of the emitting site.
    pub line: u32,
    /// Padding, so the pointers below are not silently aligned.
    pub reserved: u32,
    /// Static, never NULL.
    pub module: *const core::ffi::c_char,
    /// Static, never NULL.
    pub message: *const core::ffi::c_char,
    /// Static, never NULL. Already repo-relative via `-ffile-prefix-map`.
    pub file: *const core::ffi::c_char,
}

/// Trace severity, mirroring the `PWC_TRACE_LEVEL_*` enum in `uapi/pwc_trace.h`.
///
/// Ordered, so a consumer can filter with a single `>=`.
pub mod trace_level {
    /// Verbose detail.
    pub const DEBUG: u32 = 0;
    /// Normal operation.
    pub const INFO: u32 = 1;
    /// Something recoverable.
    pub const WARN: u32 = 2;
    /// A recorded failure.
    pub const ERROR: u32 = 3;
}

/// Largest single pwipc message, in bytes. Mirrors `PWC_IPC_MAX_MESSAGE`.
pub const PWC_IPC_MAX_MESSAGE: usize = 4096;
/// Concurrent pwipc ports. Mirrors `PWC_IPC_MAX_PORTS`.
pub const PWC_IPC_MAX_PORTS: usize = 8;

/// Maximum words in one parsed pw4shell command.
pub const PWC_SH_MAX_ARGS: usize = 16;
/// Maximum bytes in one pw4shell word, NUL included.
pub const PWC_SH_MAX_ARG: usize = 128;
/// Maximum bytes in one pw4shell input line.
pub const PWC_SH_MAX_LINE: usize = 1024;

/// One fixed-size tokenised pw4shell command.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PwcShCommand {
    /// `size_of::<PwcShCommand>()` as the core saw it, for forward compat.
    pub struct_size: u32,
    /// Words present. 0 for a blank or comment-only line.
    pub argc: u32,
    /// Byte length of each word, excluding the NUL.
    pub arglen: [u32; PWC_SH_MAX_ARGS],
    /// The words themselves.
    pub argv: [[u8; PWC_SH_MAX_ARG]; PWC_SH_MAX_ARGS],
}

impl core::fmt::Debug for PwcShCommand {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // The default derive would dump 2 KiB of mostly-zero bytes.
        f.debug_struct("PwcShCommand")
            .field("argc", &self.argc)
            .finish_non_exhaustive()
    }
}

/// One entry in a native wait request.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PwcWaitEnt {
    /// Handle to validate and wait on.
    pub handle: PwcHandle,
    /// Requested event bits.
    pub events: u32,
    /// Returned ready-event bits.
    pub revents: u32,
}
/// Status codes returned by every fallible packwandc entry point.
///
/// Errno-shaped: [`PWC_OK`] is zero and every failure is negative, so `st < 0`
/// is a complete failure test. Mirrors `PWC_STATUS_LIST` in
/// `uapi/pwc_status.h`; `tests/status_parity.rs` asserts the two agree.
pub mod status {
    /// Success.
    pub const PWC_OK: i32 = 0;
    /// Invalid argument.
    pub const PWC_EINVAL: i32 = -1;
    /// No such object.
    pub const PWC_ENOENT: i32 = -2;
    /// Operation not permitted by the handle's rights.
    pub const PWC_EPERM: i32 = -3;
    /// Would block; retry.
    pub const PWC_EAGAIN: i32 = -4;
    /// Allocation failed or an arena was exhausted.
    pub const PWC_ENOMEM: i32 = -5;
    /// Unknown handle.
    pub const PWC_EBADF: i32 = -6;
    /// Handle generation mismatch: the object was freed.
    pub const PWC_ESTALE: i32 = -7;
    /// Syscall not implemented on this platform.
    pub const PWC_ENOSYS: i32 = -8;
    /// Platform I/O failure.
    pub const PWC_EIO: i32 = -9;
    /// Deadline expired.
    pub const PWC_ETIMEDOUT: i32 = -10;
    /// Operation cancelled.
    pub const PWC_ECANCELED: i32 = -11;
    /// Value or buffer too large.
    pub const PWC_EOVERFLOW: i32 = -12;
}

pub use status::*;

/// ABI major version this binding was generated against.
///
/// Checked against the running kernel at boot; a mismatch is fatal.
pub const PWC_ABI_VERSION_MAJOR: u32 = 0;
/// ABI minor version this binding was generated against.
///
/// Mirrors `PWC_ABI_VERSION_MINOR` in `uapi/pwc_abi.h` and must be bumped with
/// it; `abi_parity.rs` compares the two against the linked core.
pub const PWC_ABI_VERSION_MINOR: u32 = 2;

/// Configuration consumed once by [`pwc_boot`].
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PwcBootConfig {
    /// Fixed number of handle-table slots; zero is invalid.
    pub handle_capacity: u32,
    /// Fixed worker-pool size requested by the host.
    pub worker_count: u32,
}

/// One keyboard or mouse packet from the native Raw Input queue.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PwcRawInputEvent {
    /// ABI size of this record.
    pub struct_size: u32,
    /// Packet kind (`PWC_RAW_INPUT_KEYBOARD` or `PWC_RAW_INPUT_MOUSE`).
    pub kind: u32,
    /// Windows message timestamp in milliseconds.
    pub timestamp_ms: u32,
    /// Hardware keyboard scan code.
    pub make_code: u16,
    /// Native keyboard or mouse flags.
    pub flags: u16,
    /// Keyboard virtual-key code.
    pub virtual_key: u16,
    /// Native mouse button flags.
    pub button_flags: u16,
    /// Unaccelerated relative mouse X delta.
    pub delta_x: i32,
    /// Unaccelerated relative mouse Y delta.
    pub delta_y: i32,
    /// Signed mouse-wheel delta.
    pub wheel_delta: i16,
    /// Reserved for ABI-compatible expansion.
    pub reserved: u16,
}

/// Keyboard Raw Input packet.
pub const PWC_RAW_INPUT_KEYBOARD: u32 = 1;
/// Mouse Raw Input packet.
pub const PWC_RAW_INPUT_MOUSE: u32 = 2;
unsafe extern "C" {
    /// Boot the native core.
    pub fn pwc_boot(config: *const PwcBootConfig) -> i32;
    /// Tear down the native core.
    pub fn pwc_shutdown();
    /// Register keyboard and mouse Raw Input for one application window.
    pub fn pwc_raw_input_start(native_window: usize) -> i32;
    /// Stop Raw Input and detach the window subclass.
    pub fn pwc_raw_input_stop();
    /// Pop one packet from the bounded native queue.
    pub fn pwc_raw_input_read(out: *mut PwcRawInputEvent) -> i32;
    /// Read the cumulative queue-overflow counter.
    pub fn pwc_raw_input_dropped(out: *mut u64) -> i32;
}
/// Safe wrappers for the raw C calls.
pub mod safe {
    use core::ffi::CStr;

    /// Report the ABI version the linked core implements.
    ///
    /// Returns the raw status; both out-params are written only on success.
    pub fn version(major: &mut u32, minor: &mut u32) -> i32 {
        // SAFETY: `major` and `minor` are live, aligned, writable `u32`s for
        // the duration of the call, which is exactly what the C signature
        // requires. The callee writes them only on success and never retains
        // the pointers.
        unsafe { crate::pwc_sys_version(major, minor) }
    }

    /// The kernel's stable identifier for a status code, e.g. `"PWC_EINVAL"`.
    ///
    /// Never empty: an unrecognised code yields `"PWC_EUNKNOWN"`.
    pub fn status_name(status: i32) -> &'static str {
        // SAFETY: pwc_sys_status_name is documented (uapi/syscalls.def) to
        // return a non-NULL pointer to a NUL-terminated string literal with
        // static storage duration, for every possible input including
        // unknown codes. `packwandc-sys/tests/abi_parity.rs` asserts the
        // never-NULL promise holds for i32::MIN and for every defined code.
        let ptr = unsafe { crate::pwc_sys_status_name(status) };
        debug_assert!(!ptr.is_null(), "pwc_sys_status_name returned NULL");
        if ptr.is_null() {
            return "PWC_EUNKNOWN";
        }

        // SAFETY: as above -- a valid, NUL-terminated, static C string.
        let cstr = unsafe { CStr::from_ptr(ptr) };
        // The kernel emits ASCII identifiers only, so this cannot fail; fall
        // back rather than panic across an FFI boundary if it somehow does.
        cstr.to_str().unwrap_or("PWC_EUNKNOWN")
    }
    /// Boot the core with a fixed handle table and worker count.
    pub fn boot(handle_capacity: u32, worker_count: u32) -> i32 {
        let config = crate::PwcBootConfig {
            handle_capacity,
            worker_count,
        };
        // SAFETY: config is a valid C-compatible value for the duration of the call.
        unsafe { crate::pwc_boot(&config) }
    }

    /// Shut down the core after its owning host is dropped.
    pub fn shutdown() {
        // SAFETY: shutdown takes no pointers and the C core retains no Rust state.
        unsafe { crate::pwc_shutdown() }
    }
    /// Start focused-window Raw Input capture.
    pub fn raw_input_start(native_window: usize) -> i32 {
        // SAFETY: the native side validates the by-value platform handle.
        unsafe { crate::pwc_raw_input_start(native_window) }
    }

    /// Stop focused-window Raw Input capture.
    pub fn raw_input_stop() {
        // SAFETY: this entry point takes no pointers and is idempotent.
        unsafe { crate::pwc_raw_input_stop() }
    }

    /// Pop one Raw Input packet into Rust-owned storage.
    pub fn raw_input_read(out: &mut crate::PwcRawInputEvent) -> i32 {
        // SAFETY: `out` is writable for the call and is not retained.
        unsafe { crate::pwc_raw_input_read(out) }
    }

    /// Read the cumulative number of queue overflows.
    pub fn raw_input_dropped(out: &mut u64) -> i32 {
        // SAFETY: `out` is writable for the call and is not retained.
        unsafe { crate::pwc_raw_input_dropped(out) }
    }

    /// Create a port into Rust-owned output storage.
    pub fn port_create(out: &mut crate::PwcHandle) -> i32 {
        // SAFETY: out is valid, aligned, writable, and not retained by C.
        unsafe { crate::pwc_ipc_port_create(out) }
    }

    /// Append one framed message to a port from caller-owned bytes.
    pub fn ipc_send(port: crate::PwcHandle, data: &[u8]) -> i32 {
        // SAFETY: the slice is readable for the call and is not retained.
        unsafe { crate::pwc_ipc_send(port, data.as_ptr(), data.len()) }
    }

    /// Pop the oldest framed message into caller-owned memory.
    pub fn ipc_recv(port: crate::PwcHandle, buffer: &mut [u8], out_len: &mut usize) -> i32 {
        // SAFETY: the slice is writable and neither pointer is retained.
        unsafe { crate::pwc_ipc_recv(port, buffer.as_mut_ptr(), buffer.len(), out_len) }
    }

    /// Close a port and release its ring slot.
    pub fn ipc_port_close(port: crate::PwcHandle) -> i32 {
        // SAFETY: the handle is passed by value with no pointer invariants.
        unsafe { crate::pwc_ipc_port_close(port) }
    }

    /// Close a by-value native handle.
    pub fn handle_close(handle: crate::PwcHandle) -> i32 {
        // SAFETY: the C ABI takes the handle by value with no pointer preconditions.
        unsafe { crate::pwc_handle_close(handle) }
    }

    /// Wait on one or more entries held in a Rust slice.
    pub fn wait(entries: &mut [crate::PwcWaitEnt], timeout_ms: i64, ready: &mut usize) -> i32 {
        // SAFETY: the slice pointer and length describe writable contiguous storage;
        // C only accesses it for this call and does not retain it.
        unsafe {
            crate::pwc_wait(
                entries.as_mut_ptr().cast::<core::ffi::c_void>(),
                entries.len(),
                timeout_ms,
                ready,
            )
        }
    }
    /// Adopt a process into the platform process-tree owner.
    pub fn proc_adopt(pid: u32, out: &mut crate::PwcHandle) -> i32 {
        // SAFETY: out is writable for the duration of the call and is not retained.
        unsafe { crate::pwc_proc_adopt(pid, out) }
    }

    /// Terminate the process tree and consume its native owner.
    pub fn proc_kill(process: crate::PwcHandle) -> i32 {
        // SAFETY: the handle is passed by value and has no pointer invariants.
        unsafe { crate::pwc_proc_kill(process) }
    }
    /// Store a secret from caller-owned bytes.
    pub fn keys_save(secret: &[u8]) -> i32 {
        // SAFETY: the slice is readable and retained only for the call.
        unsafe { crate::pwc_keys_save(secret.as_ptr(), secret.len()) }
    }

    /// Load a secret into caller-owned memory.
    pub fn keys_load(buffer: &mut [u8], out_len: &mut usize) -> i32 {
        // SAFETY: the slice is writable and neither pointer is retained.
        unsafe { crate::pwc_keys_load(buffer.as_mut_ptr(), buffer.len(), out_len) }
    }

    /// Remove the fixed Packwand credential.
    pub fn keys_clear() -> i32 {
        // SAFETY: this syscall has no pointer arguments.
        unsafe { crate::pwc_keys_clear() }
    }
    /// Validate an untrusted UTF-8 relative path.
    pub fn fs_validate_relative(path: &[u8]) -> i32 {
        // SAFETY: the slice is readable for this call and is not retained.
        unsafe { crate::pwc_fs_validate_relative(path.as_ptr(), path.len()) }
    }
    /// Read a rooted file into caller-owned memory.
    pub fn fs_read(root: &[u8], path: &[u8], buffer: &mut [u8], out_len: &mut usize) -> i32 {
        // SAFETY: all slices remain valid for the call and no pointer is retained.
        unsafe {
            crate::pwc_fs_read(
                root.as_ptr(),
                root.len(),
                path.as_ptr(),
                path.len(),
                buffer.as_mut_ptr(),
                buffer.len(),
                out_len,
            )
        }
    }

    /// Durably replace a rooted file from caller-owned bytes.
    pub fn fs_atomic_write(root: &[u8], path: &[u8], content: &[u8]) -> i32 {
        // SAFETY: all slices are readable for the call and no pointer is retained.
        unsafe {
            crate::pwc_fs_atomic_write(
                root.as_ptr(),
                root.len(),
                path.as_ptr(),
                path.len(),
                content.as_ptr(),
                content.len(),
            )
        }
    }
    /// Open a recursive native filesystem watch.
    pub fn fs_watch_open(root: &[u8], out: &mut crate::PwcHandle) -> i32 {
        // SAFETY: root is readable and out is writable for the call; neither is retained.
        unsafe { crate::pwc_fs_watch_open(root.as_ptr(), root.len(), out) }
    }

    /// Block until the recursive watch observes one or more changes.
    pub fn fs_watch_read(watch: crate::PwcHandle, out_events: &mut usize) -> i32 {
        // SAFETY: the handle is by value and out_events is writable for the call.
        unsafe { crate::pwc_fs_watch_read(watch, out_events) }
    }

    /// Stream a watch's change batches to a port on a kernel-owned thread.
    pub fn fs_watch_stream(watch: crate::PwcHandle, port: crate::PwcHandle) -> i32 {
        // SAFETY: both handles are by value with no pointer invariants.
        unsafe { crate::pwc_fs_watch_stream(watch, port) }
    }

    /// Close a recursive native filesystem watch.
    pub fn fs_watch_close(watch: crate::PwcHandle) -> i32 {
        // SAFETY: the handle is passed by value with no pointer invariants.
        unsafe { crate::pwc_fs_watch_close(watch) }
    }
    /// Query process liveness through the native platform API.
    pub fn proc_exists(pid: u32, out_alive: &mut u32) -> i32 {
        // SAFETY: out_alive is writable for this call and is not retained.
        unsafe { crate::pwc_proc_exists(pid, out_alive) }
    }
    /// Duplicate a handle into Rust-owned output storage with narrowed rights.
    pub fn handle_dup(handle: crate::PwcHandle, rights: u32, out: &mut crate::PwcHandle) -> i32 {
        // SAFETY: out is valid, aligned, writable, and not retained by C.
        unsafe { crate::pwc_handle_dup(handle, rights, out) }
    }

    /// Copy the calling thread's last-error detail record out of the core.
    ///
    /// Returns `None` when the thread has recorded no failure yet, or when the
    /// core hands back a record this build cannot interpret.
    ///
    /// This is the one shim that returns something other than a raw status,
    /// and the copy is the reason it can exist at all rather than an
    /// ergonomic flourish. `pwc_last_error` points at a *mutable* thread-local:
    /// the next failing call on this thread overwrites it, so handing out a
    /// `&'static PwcErrorDetail` would be a lie the borrow checker could not
    /// catch. Snapshotting at the boundary is what discharges that obligation,
    /// and it cannot happen in `packwandc`, which is `#![forbid(unsafe_code)]`
    /// and so cannot dereference the pointer.
    #[must_use]
    pub fn last_error() -> Option<ErrorDetail> {
        // SAFETY: pwc_last_error_get returns a non-NULL pointer to a
        // thread-local with static storage duration, initialised before first
        // use, for every possible call. It is valid for reads for as long as
        // this thread is alive, and the read below completes before any other
        // packwandc call on this thread can overwrite it.
        let ptr = unsafe { crate::pwc_last_error_get() };
        debug_assert!(!ptr.is_null(), "pwc_last_error_get returned NULL");
        if ptr.is_null() {
            return None;
        }
        // SAFETY: as above â€” a valid, aligned, initialised PwcErrorDetail.
        let raw = unsafe { *ptr };

        // A record whose struct_size is not the one this build compiled
        // against came from a core built from different headers. Reading its
        // string pointers would be guesswork, so refuse rather than guess.
        if raw.struct_size as usize != core::mem::size_of::<PwcErrorDetail>() {
            return None;
        }
        // status >= 0 is the "nothing has failed on this thread" sentinel.
        if raw.status >= crate::PWC_OK {
            return None;
        }

        Some(ErrorDetail {
            status: raw.status,
            platform_code: raw.platform_code,
            line: raw.line,
            module: static_str(raw.module)?,
            message: static_str(raw.message)?,
            file: static_str(raw.file)?,
        })
    }

    /// Borrow a never-NULL, static, NUL-terminated C string as `&'static str`.
    ///
    /// Every string in a detail record is a string literal compiled into the
    /// core, so `'static` is accurate rather than assumed. Non-UTF-8 or NULL
    /// yields `None` instead of panicking across the FFI boundary.
    fn static_str(ptr: *const core::ffi::c_char) -> Option<&'static str> {
        if ptr.is_null() {
            return None;
        }
        // SAFETY: the C side documents every detail-record string as a
        // never-NULL pointer to a NUL-terminated literal with static storage
        // duration (kernel/pwc_error.h); pwc_error_record substitutes a literal
        // placeholder rather than storing a NULL.
        let cstr = unsafe { CStr::from_ptr(ptr) };
        cstr.to_str().ok()
    }

    /// Drain one record from the kernel's trace ring, oldest first.
    ///
    /// `Ok(None)` means the ring is empty; `Err` carries the raw status.
    ///
    /// Copied out for the same reason as [`last_error`]: the record lives in a
    /// ring slot that is released back to writers the moment the drain
    /// advances, so a borrow would dangle by design.
    ///
    /// Single-consumer. Two callers draining concurrently silently split the
    /// stream between them rather than each seeing all of it.
    pub fn ktrace_drain() -> Result<Option<TraceRecord>, i32> {
        let mut raw = PwcTraceRecord {
            struct_size: 0,
            level: 0,
            sequence: 0,
            status: 0,
            platform_code: 0,
            line: 0,
            reserved: 0,
            module: core::ptr::null(),
            message: core::ptr::null(),
            file: core::ptr::null(),
        };
        // SAFETY: `raw` is a live, aligned, writable PwcTraceRecord for the
        // duration of the call, which is all the C signature requires. The
        // callee writes it only on success and does not retain the pointer.
        let status = unsafe { crate::pwc_ktrace_drain(&mut raw) };
        if status == crate::PWC_EAGAIN {
            return Ok(None);
        }
        if status < crate::PWC_OK {
            return Err(status);
        }
        if raw.struct_size as usize != core::mem::size_of::<PwcTraceRecord>() {
            // A core built from different headers. Reading its string pointers
            // would be guesswork.
            return Err(crate::PWC_ENOSYS);
        }
        Ok(Some(TraceRecord {
            sequence: raw.sequence,
            level: raw.level,
            status: raw.status,
            platform_code: raw.platform_code,
            line: raw.line,
            module: static_str(raw.module).unwrap_or("?"),
            message: static_str(raw.message).unwrap_or("(no message)"),
            file: static_str(raw.file).unwrap_or("?"),
        }))
    }

    /// Records discarded because the ring was full, cumulative since boot.
    ///
    /// A drop leaves no record behind, so a consumer that never reads this
    /// cannot distinguish a quiet period from an overflowing one.
    pub fn ktrace_dropped() -> Result<u64, i32> {
        let mut dropped = 0u64;
        // SAFETY: `dropped` is a live, writable u64 for the call and is not retained.
        let status = unsafe { crate::pwc_ktrace_dropped(&mut dropped) };
        if status < crate::PWC_OK {
            return Err(status);
        }
        Ok(dropped)
    }

    /// An owned snapshot of one ktrace record.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TraceRecord {
        /// Monotonic; a gap since the previous record means drops occurred.
        pub sequence: u64,
        /// One of the [`trace_level`](crate::trace_level) constants.
        pub level: u32,
        /// The status being reported.
        pub status: i32,
        /// OS error code, or 0.
        pub platform_code: i32,
        /// Source line of the emitting site.
        pub line: u32,
        /// Emitting subsystem.
        pub module: &'static str,
        /// What the emitting site said.
        pub message: &'static str,
        /// Repo-relative source file of the emitting site.
        pub file: &'static str,
    }

    /// Parse and run one pw4shell line.
    ///
    /// Output lines are sent as individual framed messages on `port`. Pass a
    /// default (invalid) handle to discard them.
    ///
    /// Returns the raw status alongside the parsed command. `PWC_ENOSYS` means
    /// the line parsed cleanly but names no kernel built-in â€” the caller is
    /// expected to dispatch `words` itself. A parse failure returns the parse
    /// error and an empty word list.
    pub fn sh_exec(port: crate::PwcHandle, line: &[u8]) -> (i32, PwcShCommand) {
        let mut command = new_sh_command();
        // SAFETY: `line` is readable for the call and not retained; `command`
        // is a live, writable PwcShCommand the callee fully initialises.
        let status = unsafe { crate::pwc_sh_exec(port, line.as_ptr(), line.len(), &mut command) };
        (status, command)
    }

    /// Tokenise one pw4shell line without running it.
    ///
    /// Exposed so a UI can use the kernel's quoting rules rather than
    /// reimplementing them â€” a console that disagrees with its own backend
    /// about what `"a b"` means is worse than one with no completion at all.
    pub fn sh_parse(line: &[u8]) -> Result<PwcShCommand, i32> {
        let mut command = new_sh_command();
        // SAFETY: as above.
        let status = unsafe { crate::pwc_sh_parse(line.as_ptr(), line.len(), &mut command) };
        if status < crate::PWC_OK {
            return Err(status);
        }
        Ok(command)
    }

    fn new_sh_command() -> PwcShCommand {
        PwcShCommand {
            struct_size: 0,
            argc: 0,
            arglen: [0; crate::PWC_SH_MAX_ARGS],
            argv: [[0; crate::PWC_SH_MAX_ARG]; crate::PWC_SH_MAX_ARGS],
        }
    }

    /// An owned snapshot of the core's last-error detail record.
    ///
    /// See [`last_error`] for why this is copied rather than borrowed.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ErrorDetail {
        /// The status this record was recorded for.
        pub status: i32,
        /// `GetLastError()`/`errno`/D-Bus code, or 0 when there was none.
        pub platform_code: i32,
        /// Source line of the recording site in the C tree.
        pub line: u32,
        /// Recording subsystem: `"core"`, `"pwfs"`, `"arch/win32"`.
        pub module: &'static str,
        /// What the recording site said went wrong.
        pub message: &'static str,
        /// Repo-relative source file of the recording site.
        pub file: &'static str,
    }

    use super::{PwcErrorDetail, PwcShCommand, PwcTraceRecord};
}
