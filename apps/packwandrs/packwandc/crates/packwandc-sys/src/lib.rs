//! Raw FFI bindings to the packwandc native core.
//!
//! This crate is the single `unsafe` boundary of the whole application
//! (packwandc.md 6.1). It has two parts and no third:
//!
//! - the raw `extern "C"` block in [`bindings`], exactly as the C headers
//!   declare it; and
//! - [`safe`], a minimal shim holding the *only* `unsafe` blocks in the
//!   repository.
//!
//! [`safe`] exists because calling an `extern "C"` function is itself an
//! `unsafe` operation. Without it the safe SDK could not be
//! `#![forbid(unsafe_code)]` and the `unsafe` surface would be split across
//! two crates instead of concentrated in one — which is the entire point of
//! this layering.
//!
//! Nothing else belongs here. No error handling, no owned types, no
//! convenience: those go one level up, in the `packwandc` crate. A helper in a
//! `-sys` crate is a helper nobody audits.

#![no_std]

mod bindings;

pub use bindings::*;

/// A kernel object handle: an index into the kernel's table plus a generation
/// counter.
///
/// The generation counter is why this is two fields rather than one integer.
/// Closing a slot increments its generation, so a handle held across a close
/// resolves to [`PWC_ESTALE`] rather than to whatever now occupies the slot —
/// turning use-after-free and ABA into a returned error instead of memory
/// corruption. See packwandc.md 3.2.
///
/// Layout is part of the wire ABI and is asserted in C
/// (`uapi/pwc_handle.h`) and in this crate's tests.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PwcHandle {
    /// Table slot. Index 0 is never handed out, so a zeroed handle is invalid
    /// by construction.
    pub index: u32,
    /// Incremented every time the slot is freed.
    pub generation: u32,
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
pub const PWC_ABI_VERSION_MINOR: u32 = 1;

/// Configuration consumed once by [`pwc_boot`].
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PwcBootConfig {
    /// Fixed number of handle-table slots; zero is invalid.
    pub handle_capacity: u32,
    /// Fixed worker-pool size requested by the host.
    pub worker_count: u32,
}

unsafe extern "C" {
    /// Boot the native core.
    pub fn pwc_boot(config: *const PwcBootConfig) -> i32;
    /// Tear down the native core.
    pub fn pwc_shutdown();
}
/// The only `unsafe` blocks in the repository.
///
/// Each function here wraps exactly one C call whose safety obligations are
/// discharged by its Rust signature alone — a `&mut u32` is always a valid
/// pointer, a returned `const char *` documented as a never-NULL static
/// string is always a valid `&'static CStr`. Anything whose safety depends on
/// caller-supplied invariants does **not** get a shim; it stays raw and is
/// wrapped by an owned type in the `packwandc` crate, where the invariant can
/// be enforced by construction.
///
/// Keeping this module tiny is the point. Every function added here is
/// `unsafe` code that must be audited by hand; the ambition is that it never
/// grows faster than the syscall table.
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

    /// Create a port into Rust-owned output storage.
    pub fn port_create(out: &mut crate::PwcHandle) -> i32 {
        // SAFETY: out is valid, aligned, writable, and not retained by C.
        unsafe { crate::pwc_ipc_port_create(out) }
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
}
