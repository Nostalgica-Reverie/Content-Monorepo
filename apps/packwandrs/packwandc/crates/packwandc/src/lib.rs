//! Safe Rust SDK over the packwandc native core.
//!
//! This crate re-establishes memory safety immediately above the one `unsafe`
//! crate in the repository ([`packwandc_sys`]). It is itself
//! `#![forbid(unsafe_code)]`, which means every `unsafe` block backing this
//! API lives in exactly one auditable place.
//!
//! Two things happen here and nowhere else:
//!
//! - **Status codes become [`Result`].** C returns errno-shaped integers;
//!   callers in this workspace should never see one.
//! - **Handles become owned types with [`Drop`].** A handle that goes out of
//!   scope is closed. Forgetting `pwc_close` stops being possible.
//!
//! # Phase 0
//!
//! Only the version and status surface exists so far. Handles, waiting, and
//! the subsystem modules arrive in phases 1 and 2 — see `packwandc.md` §10.

#![forbid(unsafe_code)]

use core::fmt;

use packwandc_sys as sys;

/// The ABI version a packwandc build implements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbiVersion {
    /// Incremented only on a breaking change.
    pub major: u32,
    /// Incremented on backward-compatible additions.
    pub minor: u32,
}

impl fmt::Display for AbiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// An error returned by the native core.
///
/// Wraps the raw [`pwc_status`](sys::PWC_OK) integer and pairs it with the
/// stable identifier the kernel reports for it, so log output names the code
/// rather than printing a bare negative number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Error {
    status: i32,
}

impl Error {
    /// Wrap a raw status code.
    ///
    /// Returns `None` for [`PWC_OK`](sys::PWC_OK) and any other non-negative
    /// value, since those are not failures.
    #[must_use]
    pub fn from_status(status: i32) -> Option<Self> {
        (status < sys::PWC_OK).then_some(Self { status })
    }

    /// The raw status code.
    #[must_use]
    pub const fn status(self) -> i32 {
        self.status
    }

    /// The kernel's stable identifier for this code, such as `"PWC_EINVAL"`.
    ///
    /// Never empty: an unrecognised code reports `"PWC_EUNKNOWN"`.
    ///
    /// The string comes from the kernel itself rather than from a table
    /// mirrored on this side, so it cannot drift from the C definition.
    #[must_use]
    pub fn name(self) -> &'static str {
        sys::safe::status_name(self.status)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.status)
    }
}

impl core::error::Error for Error {}

/// Result alias for packwandc operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Convert a raw status into a `Result`, discarding the success value.
fn check(status: i32) -> Result<()> {
    match Error::from_status(status) {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// The ABI major version this build was compiled against.
///
/// Compare against [`abi_version`] to detect a core built from different
/// headers than the bindings.
#[must_use]
pub const fn expected_abi_major() -> u32 {
    sys::PWC_ABI_VERSION_MAJOR
}

/// The ABI minor version this build was compiled against.
#[must_use]
pub const fn expected_abi_minor() -> u32 {
    sys::PWC_ABI_VERSION_MINOR
}

/// Report the ABI version of the linked native core.
///
/// # Errors
///
/// Returns [`Error`] if the native call fails, which for this syscall can only
/// happen on an argument validation failure that this wrapper makes
/// impossible — so in practice it does not fail.
pub fn abi_version() -> Result<AbiVersion> {
    let mut major = 0u32;
    let mut minor = 0u32;

    // The single unsafe call in this path lives in packwandc-sys; this crate
    // forbids unsafe, so the binding is invoked through a thin shim there.
    check(sys_version(&mut major, &mut minor))?;

    Ok(AbiVersion { major, minor })
}

/// Verify that the linked core speaks an ABI this build understands.
///
/// A major-version mismatch is unrecoverable: the syscall numbering or the
/// struct layouts have changed underneath us.
///
/// # Errors
///
/// Returns [`Error`] with [`PWC_ENOSYS`](sys::PWC_ENOSYS) on a major mismatch.
pub fn check_abi_compatibility() -> Result<AbiVersion> {
    let version = abi_version()?;

    if version.major == sys::PWC_ABI_VERSION_MAJOR {
        Ok(version)
    } else {
        Err(Error {
            status: sys::PWC_ENOSYS,
        })
    }
}

// The FFI shim. `#![forbid(unsafe_code)]` applies to this crate, so the call
// is delegated to packwandc-sys, which owns the unsafe.
fn sys_version(major: &mut u32, minor: &mut u32) -> i32 {
    sys::safe::version(major, minor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_is_not_an_error() {
        assert!(Error::from_status(sys::PWC_OK).is_none());
        assert!(Error::from_status(1).is_none());
    }

    #[test]
    fn negative_is_an_error_and_names_itself() {
        let err = Error::from_status(sys::PWC_ESTALE).expect("negative status is an error");
        assert_eq!(err.status(), sys::PWC_ESTALE);
        assert_eq!(err.name(), "PWC_ESTALE");
        assert_eq!(err.to_string(), "PWC_ESTALE (-7)");
    }

    #[test]
    fn abi_version_is_readable() {
        let version = abi_version().expect("version syscall cannot fail here");
        assert_eq!(version.major, sys::PWC_ABI_VERSION_MAJOR);
        assert_eq!(version.minor, sys::PWC_ABI_VERSION_MINOR);
    }

    #[test]
    fn abi_compatibility_holds_against_the_linked_core() {
        assert!(check_abi_compatibility().is_ok());
    }
}

/// An owned kernel handle. Dropping it closes the native resource.
#[derive(Debug)]
pub struct Handle {
    raw: sys::PwcHandle,
}

impl Handle {
    /// Creates a new IPC port handle.
    ///
    /// # Errors
    ///
    /// Returns the native status when the core is not booted or its fixed
    /// handle table is full.
    pub fn port() -> Result<Self> {
        let mut raw = sys::PwcHandle::default();
        check(sys::safe::port_create(&mut raw))?;
        Ok(Self { raw })
    }

    /// Wait for this handle and return the reported event bits.
    ///
    /// # Errors
    ///
    /// Returns the native timeout, rights, or stale-handle status.
    pub fn wait(&self, events: u32, timeout_ms: i64) -> Result<u32> {
        let mut entries = [sys::PwcWaitEnt {
            handle: self.raw,
            events,
            revents: 0,
        }];
        let mut ready = 0usize;
        check(sys::safe::wait(&mut entries, timeout_ms, &mut ready))?;
        if ready != 1 {
            return Err(Error {
                status: sys::PWC_EAGAIN,
            });
        }
        Ok(entries[0].revents)
    }
    /// Duplicate this handle with a subset of its current rights.
    ///
    /// # Errors
    ///
    /// Returns an error if the requested rights widen the original capability.
    pub fn duplicate(&self, rights: u32) -> Result<Self> {
        let mut raw = sys::PwcHandle::default();
        check(sys::safe::handle_dup(self.raw, rights, &mut raw))?;
        Ok(Self { raw })
    }

    /// Explicitly close this handle and return the native close result.
    ///
    /// The handle remains inert afterwards; its `Drop` performs no additional
    /// kernel operation because the table rejects stale handles safely.
    pub fn close(self) -> Result<()> {
        check(sys::safe::handle_close(self.raw))
    }
    /// The ABI handle value for diagnostics and FFI interop.
    #[must_use]
    pub const fn raw(&self) -> sys::PwcHandle {
        self.raw
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = sys::safe::handle_close(self.raw);
    }
}

/// Native ownership of a launched process tree.
#[derive(Debug)]
pub struct ProcessTree {
    raw: Option<sys::PwcHandle>,
}

impl ProcessTree {
    /// Adopt a newly spawned process into the platform tree owner.
    ///
    /// # Errors
    ///
    /// Returns an error when the process cannot be opened or assigned.
    pub fn adopt(pid: u32) -> Result<Self> {
        let mut raw = sys::PwcHandle::default();
        check(sys::safe::proc_adopt(pid, &mut raw))?;
        Ok(Self { raw: Some(raw) })
    }

    /// Atomically terminate the owned process tree.
    ///
    /// # Errors
    ///
    /// Returns the translated platform or stale-handle error.
    pub fn kill(&mut self) -> Result<()> {
        let raw = self.raw.take().ok_or(Error {
            status: sys::PWC_EBADF,
        })?;
        check(sys::safe::proc_kill(raw))
    }
}

impl Drop for ProcessTree {
    fn drop(&mut self) {
        if let Some(raw) = self.raw.take() {
            let _ = sys::safe::proc_kill(raw);
        }
    }
}

/// Native operating-system credential storage for Packwand secrets.
#[derive(Debug, Default, Clone, Copy)]
pub struct KeyStore;

impl KeyStore {
    /// Persist a secret in the operating-system credential manager.
    ///
    /// # Errors
    ///
    /// Returns a native storage or size error.
    pub fn save(self, secret: &[u8]) -> Result<()> {
        check(sys::safe::keys_save(secret))
    }

    /// Load the stored secret into Rust-owned memory.
    ///
    /// # Errors
    ///
    /// Returns a native storage error. A missing credential is `Ok(None)`.
    pub fn load(self) -> Result<Option<Vec<u8>>> {
        const MAX_SECRET: usize = 2560;
        let mut bytes = vec![0u8; MAX_SECRET];
        let mut length = 0usize;
        let status = sys::safe::keys_load(&mut bytes, &mut length);
        if status == sys::PWC_ENOENT {
            return Ok(None);
        }
        check(status)?;
        bytes.truncate(length);
        Ok(Some(bytes))
    }

    /// Remove the stored credential. Missing credentials are successful.
    ///
    /// # Errors
    ///
    /// Returns a native credential-manager error.
    pub fn clear(self) -> Result<()> {
        check(sys::safe::keys_clear())
    }
}

/// Validate that a UTF-8 path is strictly relative and contains no escape components.
///
/// # Errors
///
/// Returns `PWC_EPERM` for absolute, drive-qualified, empty-component, dot, or
/// parent-directory paths.
pub fn validate_relative_path(path: &str) -> Result<()> {
    check(sys::safe::fs_validate_relative(path.as_bytes()))
}

/// Read a file beneath a native filesystem root.
///
/// # Errors
///
/// Returns confinement, not-found, size, or platform I/O errors.
pub fn fs_read(root: &str, path: &str) -> Result<Vec<u8>> {
    let mut probe = [0u8; 1];
    let mut length = 0usize;
    let status = sys::safe::fs_read(root.as_bytes(), path.as_bytes(), &mut probe, &mut length);
    if status == sys::PWC_OK {
        return Ok(probe[..length].to_vec());
    }
    if status != sys::PWC_EOVERFLOW {
        return Err(Error::from_status(status).expect("negative status"));
    }
    let mut bytes = vec![0u8; length];
    check(sys::safe::fs_read(
        root.as_bytes(),
        path.as_bytes(),
        &mut bytes,
        &mut length,
    ))?;
    bytes.truncate(length);
    Ok(bytes)
}

/// Durably replace a file beneath a native filesystem root.
///
/// # Errors
///
/// Returns confinement or platform I/O errors.
pub fn fs_atomic_write(root: &str, path: &str, content: &[u8]) -> Result<()> {
    check(sys::safe::fs_atomic_write(
        root.as_bytes(),
        path.as_bytes(),
        content,
    ))
}

#[cfg(all(test, windows))]
mod windows_native_tests {
    use super::*;
    static KERNEL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn rooted_atomic_write_and_read_roundtrip() {
        let root = std::env::temp_dir().join(format!("packwandc-fs-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("create native fs test root");
        let root_text = root.to_str().expect("Windows temp path is UTF-8");
        fs_atomic_write(root_text, "roundtrip.bin", b"packwandc-native")
            .expect("native atomic write succeeds");
        assert_eq!(
            fs_read(root_text, "roundtrip.bin").expect("native read succeeds"),
            b"packwandc-native"
        );
        assert_eq!(
            fs_atomic_write(root_text, "../escape", b"no")
                .expect_err("parent escape is rejected")
                .status(),
            sys::PWC_EPERM
        );
        std::fs::remove_dir_all(root).expect("remove native fs test root");
    }
    #[test]
    fn recursive_watch_observes_external_change() {
        let _guard = KERNEL_LOCK.lock().expect("kernel test lock");
        let root = std::env::temp_dir().join(format!("packwandc-watch-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("create watch root");
        assert_eq!(sys::safe::boot(64, 1), sys::PWC_OK);
        let watch = FsWatch::open(root.to_str().expect("Windows temp path is UTF-8"))
            .expect("open recursive watch");
        let changed = root.join("external.txt");
        let writer = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            std::fs::write(changed, b"changed outside editor").expect("write watched file");
        });
        assert!(watch.read_changes().expect("read native changes") > 0);
        writer.join().expect("writer thread");
        drop(watch);
        sys::safe::shutdown();
        std::fs::remove_dir_all(root).expect("remove watch root");
    }
    #[test]
    fn job_object_kills_spawned_grandchild() {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};
        let _guard = KERNEL_LOCK.lock().expect("kernel test lock");
        assert_eq!(sys::safe::boot(64, 1), sys::PWC_OK);
        let gate = std::env::temp_dir().join(format!("packwandc-job-gate-{}", std::process::id()));
        let _ = std::fs::remove_file(&gate);
        let escaped_gate = gate.to_string_lossy().replace(char::from(39), "''");
        let script = format!(
            "while (!(Test-Path -LiteralPath '{escaped_gate}')) {{ Start-Sleep -Milliseconds 10 }}; $p = Start-Process powershell -ArgumentList '-NoProfile','-Command','Start-Sleep -Seconds 60' -PassThru; Write-Output $p.Id; Wait-Process -Id $p.Id"
        );
        let mut parent = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn parent process");
        let mut tree = ProcessTree::adopt(parent.id()).expect("adopt parent into Job Object");
        std::fs::write(&gate, b"go").expect("release parent");
        let mut line = String::new();
        BufReader::new(parent.stdout.take().expect("parent stdout"))
            .read_line(&mut line)
            .expect("read grandchild pid");
        let grandchild_pid: u32 = line.trim().parse().expect("numeric grandchild pid");
        assert!(process_exists(grandchild_pid).expect("query live grandchild"));
        tree.kill().expect("close kill-on-close Job Object");
        let _ = parent.wait();
        for _ in 0..50 {
            if !process_exists(grandchild_pid).expect("query killed grandchild") {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(!process_exists(grandchild_pid).expect("final grandchild query"));
        let _ = std::fs::remove_file(gate);
        sys::safe::shutdown();
    }
}

/// Recursive native filesystem watch.
#[derive(Debug)]
pub struct FsWatch {
    raw: Option<sys::PwcHandle>,
}

impl FsWatch {
    /// Begin watching a root recursively.
    ///
    /// # Errors
    ///
    /// Returns a native path or platform watcher error.
    pub fn open(root: &str) -> Result<Self> {
        let mut raw = sys::PwcHandle::default();
        check(sys::safe::fs_watch_open(root.as_bytes(), &mut raw))?;
        Ok(Self { raw: Some(raw) })
    }

    /// Create a cancellation capability for a thread blocked in [`Self::read_changes`].
    #[must_use]
    pub fn canceller(&self) -> FsWatchCanceller {
        FsWatchCanceller { raw: self.raw }
    }
    /// Block until a coalesced batch of filesystem changes is available.
    ///
    /// # Errors
    ///
    /// Returns a native watcher or stale-handle error.
    pub fn read_changes(&self) -> Result<usize> {
        let raw = self.raw.ok_or(Error {
            status: sys::PWC_EBADF,
        })?;
        let mut events = 0usize;
        check(sys::safe::fs_watch_read(raw, &mut events))?;
        Ok(events)
    }
}

/// Cancellation capability for a native filesystem watch.
#[derive(Debug)]
pub struct FsWatchCanceller {
    raw: Option<sys::PwcHandle>,
}

impl FsWatchCanceller {
    /// Close the underlying native watch, unblocking its reader.
    pub fn cancel(&mut self) {
        if let Some(raw) = self.raw.take() {
            let _ = sys::safe::fs_watch_close(raw);
        }
    }
}
impl Drop for FsWatch {
    fn drop(&mut self) {
        if let Some(raw) = self.raw.take() {
            let _ = sys::safe::fs_watch_close(raw);
        }
    }
}

/// Query whether a process is still alive using the native platform API.
///
/// # Errors
///
/// Returns an argument or platform-query error.
pub fn process_exists(pid: u32) -> Result<bool> {
    let mut alive = 0u32;
    check(sys::safe::proc_exists(pid, &mut alive))?;
    Ok(alive != 0)
}
