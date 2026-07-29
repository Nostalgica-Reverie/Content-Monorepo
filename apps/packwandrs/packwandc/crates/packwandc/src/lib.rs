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
//! the subsystem modules arrive in phases 1 and 2 â€” see `packwandc.md` Â§10.

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
///
/// Where the failing path recorded one, it also carries the kernel's detail
/// record â€” the recording module, source location, and the platform error code
/// the OS actually returned. That last field is the one that matters in
/// practice: without it a `PWC_EIO` out of the credential store or a job object
/// is indistinguishable from any other, and the OS's own reason is destroyed by
/// the next call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Error {
    status: i32,
    detail: Option<sys::safe::ErrorDetail>,
}

impl Error {
    /// Wrap a raw status code, capturing the core's detail record for it.
    ///
    /// Returns `None` for [`PWC_OK`](sys::PWC_OK) and any other non-negative
    /// value, since those are not failures.
    ///
    /// Call this immediately after the failing call. The detail lives in a
    /// thread-local that the next failure on this thread overwrites, so a
    /// record whose own `status` disagrees with `status` is treated as a
    /// leftover from an earlier call and dropped rather than misreported.
    #[must_use]
    pub fn from_status(status: i32) -> Option<Self> {
        if status >= sys::PWC_OK {
            return None;
        }
        let detail = sys::safe::last_error().filter(|detail| detail.status == status);
        Some(Self { status, detail })
    }

    /// An error this crate decided on without the core being called.
    ///
    /// Deliberately carries no detail record. The thread-local one belongs to
    /// whatever C path last failed, which for these cases is an unrelated
    /// earlier call â€” attaching it would point a reader at the wrong source
    /// line with full confidence.
    const fn without_detail(status: i32) -> Self {
        Self {
            status,
            detail: None,
        }
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

    /// What the failing path in the C tree reported, if it recorded anything.
    ///
    /// `None` means the path returned a bare status without a detail â€” common
    /// for plain argument-validation rejections, where the status is already
    /// the whole story.
    #[must_use]
    pub const fn detail(self) -> Option<sys::safe::ErrorDetail> {
        self.detail
    }

    /// The OS error code behind this failure, if there was one.
    ///
    /// `GetLastError()` on Windows, `errno` on Linux. `None` when the failure
    /// originated in packwandc's own logic rather than in a platform call.
    #[must_use]
    pub fn platform_code(self) -> Option<i32> {
        self.detail
            .filter(|detail| detail.platform_code != 0)
            .map(|detail| detail.platform_code)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.status)?;
        let Some(detail) = self.detail else {
            return Ok(());
        };
        write!(
            f,
            ": {} [{} at {}:{}]",
            detail.message, detail.module, detail.file, detail.line
        )?;
        if detail.platform_code != 0 {
            write!(f, " (platform code {})", detail.platform_code)?;
        }
        Ok(())
    }
}

impl core::error::Error for Error {}

/// Result alias for packwandc operations.
pub type Result<T> = core::result::Result<T, Error>;

/// One record drained from the core's trace ring â€” the `dmesg` analogue.
pub use sys::safe::TraceRecord;

/// Trace severity constants, mirroring `PWC_TRACE_LEVEL_*` in the C headers.
pub use sys::trace_level;

/// The outcome of running one pw4shell line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellOutcome {
    /// A kernel built-in ran it. Any output was written to the port.
    Handled,
    /// The line parsed cleanly but names no kernel built-in.
    ///
    /// Not an error: `pack`, `mod` and `diag` verbs live in the Rust crates and
    /// cannot be reached from C. The caller dispatches the words itself, having
    /// been spared reimplementing the quoting rules.
    ForHost(Vec<String>),
    /// A blank or comment-only line. A no-op, not a failure â€” a console that
    /// complains when you press enter on an empty prompt is hostile.
    Empty,
}

/// Copy the parsed words out of a native command as owned Strings.
///
/// `arglen` is authoritative over the NUL terminator, and clamped: the kernel
/// guarantees the two agree, but this side must not index past the fixed array
/// on the strength of a length field if that ever stops being true.
fn shell_words(command: &sys::PwcShCommand) -> Vec<String> {
    let argc = (command.argc as usize).min(sys::PWC_SH_MAX_ARGS);
    (0..argc)
        .map(|i| {
            let len = (command.arglen[i] as usize).min(sys::PWC_SH_MAX_ARG - 1);
            String::from_utf8_lossy(&command.argv[i][..len]).into_owned()
        })
        .collect()
}

/// Tokenise a pw4shell line without running it.
///
/// Use this rather than splitting on whitespace in the UI: quoting, escapes and
/// comments are defined once, in the kernel, and a console that disagrees with
/// its own backend about what `"a b"` means is worse than one with no
/// completion at all.
///
/// # Errors
///
/// Returns [`Error`] if the line is malformed â€” an unterminated quote, an
/// unknown escape, an over-long word, or more than one line.
pub fn shell_parse(line: &str) -> Result<Vec<String>> {
    match sys::safe::sh_parse(line.as_bytes()) {
        Ok(command) => Ok(shell_words(&command)),
        Err(status) => {
            Err(Error::from_status(status)
                .unwrap_or_else(|| Error::without_detail(sys::PWC_EINVAL)))
        }
    }
}

/// Parse and run one pw4shell line.
///
/// Output lines are written to `port` as individual framed messages, so a
/// reader taking one frame gets one line. Pass `None` to discard output.
///
/// # Errors
///
/// Returns [`Error`] only for a malformed line or a failing built-in. A line
/// the kernel does not implement is [`ShellOutcome::ForHost`], not an error.
pub fn shell_exec(line: &str, port: Option<&Port>) -> Result<ShellOutcome> {
    let raw = port.map_or_else(sys::PwcHandle::default, Port::handle);
    let (status, command) = sys::safe::sh_exec(raw, line.as_bytes());

    if status == sys::PWC_ENOSYS {
        return Ok(ShellOutcome::ForHost(shell_words(&command)));
    }
    if let Some(error) = Error::from_status(status) {
        return Err(error);
    }
    if command.argc == 0 {
        return Ok(ShellOutcome::Empty);
    }
    Ok(ShellOutcome::Handled)
}

/// Drain one trace record from the core, oldest first.
///
/// Returns `Ok(None)` when the ring is empty. Intended to be called in a loop
/// until it yields `None`.
///
/// **Single consumer.** The ring has one read cursor, so two callers draining
/// concurrently split the stream between them rather than each seeing all of
/// it. [`packwandc_host`] owns the drain for the desktop process; nothing else
/// should call this.
///
/// # Errors
///
/// Returns [`Error`] if the core is not booted, or if it reports a record
/// layout this build does not understand.
///
/// [`packwandc_host`]: https://docs.rs/packwandc-host
pub fn trace_drain() -> Result<Option<TraceRecord>> {
    sys::safe::ktrace_drain().map_err(|status| {
        Error::from_status(status).unwrap_or_else(|| Error::without_detail(sys::PWC_EIO))
    })
}

/// How many trace records the core has discarded because the ring was full.
///
/// Cumulative since boot. A drop leaves nothing behind in the stream, so a
/// consumer that never checks this cannot tell a quiet period from an
/// overflowing one.
///
/// # Errors
///
/// Returns [`Error`] if the core is not booted.
pub fn trace_dropped() -> Result<u64> {
    sys::safe::ktrace_dropped().map_err(|status| {
        Error::from_status(status).unwrap_or_else(|| Error::without_detail(sys::PWC_EIO))
    })
}

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
/// impossible â€” so in practice it does not fail.
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
        Err(Error::without_detail(sys::PWC_ENOSYS))
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
    /// # Deprecated
    ///
    /// Use [`Port`] instead. A port owns two resources â€” a handle table slot
    /// and a ring slot from the fixed pool of
    /// [`PWC_IPC_MAX_PORTS`](sys::PWC_IPC_MAX_PORTS) â€” and `Handle`'s `Drop`
    /// releases only the first. Dropping a port opened this way therefore
    /// leaks a ring slot, and after enough of them port creation fails
    /// permanently with `PWC_ENOMEM`.
    ///
    /// # Errors
    ///
    /// Returns the native status when the core is not booted or its fixed
    /// handle table is full.
    #[deprecated(note = "use Port::open: Handle's Drop leaks the port's ring slot")]
    pub fn port() -> Result<Self> {
        let mut raw = sys::PwcHandle::default();
        check(sys::safe::port_create(&mut raw))?;
        Ok(Self { raw })
    }

    /// Wait for this handle and return the reported event bits.
    ///
    /// Note that the core has no readiness source wired up yet, so this
    /// currently always sleeps out its timeout and then reports
    /// [`PWC_ETIMEDOUT`](sys::PWC_ETIMEDOUT) â€” see the header comment on
    /// `kernel/wait.c` for what an epoll/IOCP-backed answer requires. Invalid
    /// handles are still rejected up front rather than after the timeout.
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
            return Err(Error::without_detail(sys::PWC_EAGAIN));
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

/// An owned pwipc port: a framed-message channel with a fixed-size ring.
///
/// A port owns *two* resources â€” a handle table slot and one of the
/// [`PWC_IPC_MAX_PORTS`](sys::PWC_IPC_MAX_PORTS) ring slots â€” and both are
/// released on drop. That is the whole reason this type exists rather than a
/// bare [`Handle`]: closing only the handle leaks the ring slot, and the pool
/// is small enough that a leak per command exhausts it quickly.
#[derive(Debug)]
pub struct Port {
    raw: Option<sys::PwcHandle>,
}

impl Port {
    /// Open a port.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the core is not booted, the handle table is full,
    /// or every ring slot is already in use.
    pub fn open() -> Result<Self> {
        let mut raw = sys::PwcHandle::default();
        check(sys::safe::port_create(&mut raw))?;
        Ok(Self { raw: Some(raw) })
    }

    /// The underlying handle, for passing to calls that take one.
    #[must_use]
    pub fn handle(&self) -> sys::PwcHandle {
        self.raw.unwrap_or_default()
    }

    /// Append one framed message.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] with `PWC_EOVERFLOW` when the ring is full. The
    /// message is not queued â€” a partial frame is never published.
    pub fn send(&self, data: &[u8]) -> Result<()> {
        check(sys::safe::ipc_send(self.handle(), data))
    }

    /// Pop the oldest framed message, or `None` when the port is empty.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the port is closed or the message is larger than
    /// [`PWC_IPC_MAX_MESSAGE`](sys::PWC_IPC_MAX_MESSAGE).
    pub fn recv(&self) -> Result<Option<Vec<u8>>> {
        let mut buffer = vec![0u8; sys::PWC_IPC_MAX_MESSAGE];
        let mut length = 0usize;
        let status = sys::safe::ipc_recv(self.handle(), &mut buffer, &mut length);
        if status == sys::PWC_EAGAIN {
            return Ok(None);
        }
        check(status)?;
        buffer.truncate(length);
        Ok(Some(buffer))
    }

    /// Drain every queued message, lossily decoding each as UTF-8.
    ///
    /// Lossy because the messages are shell output destined for a log view:
    /// mangling one byte is better than dropping a whole line of diagnostics
    /// over an encoding question nobody can act on.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if a receive fails for a reason other than emptiness.
    pub fn drain_lines(&self) -> Result<Vec<String>> {
        let mut lines = Vec::new();
        while let Some(bytes) = self.recv()? {
            lines.push(String::from_utf8_lossy(&bytes).into_owned());
        }
        Ok(lines)
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        if let Some(raw) = self.raw.take() {
            // Releases the ring slot as well as the handle. A plain
            // handle_close here would leak the slot for the process lifetime.
            let _ = sys::safe::ipc_port_close(raw);
        }
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
        let raw = self
            .raw
            .take()
            .ok_or(Error::without_detail(sys::PWC_EBADF))?;
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

/// Device-level input packet captured for the focused Packwand window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawInputEvent {
    /// Native packet category.
    pub kind: RawInputKind,
    /// Windows message timestamp in milliseconds.
    pub timestamp_ms: u32,
    /// Keyboard scan code.
    pub make_code: u16,
    /// Native keyboard flags.
    pub flags: u16,
    /// Keyboard virtual-key code.
    pub virtual_key: u16,
    /// Native mouse button flags.
    pub button_flags: u16,
    /// Unaccelerated relative mouse X movement.
    pub delta_x: i32,
    /// Unaccelerated relative mouse Y movement.
    pub delta_y: i32,
    /// Mouse wheel movement.
    pub wheel_delta: i16,
}

/// Native Raw Input packet category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawInputKind {
    /// Keyboard scan-code packet.
    Keyboard,
    /// Mouse delta/button packet.
    Mouse,
}

/// Start Raw Input capture for one platform window.
///
/// # Errors
///
/// Returns a native argument/platform error, or `PWC_ENOSYS` off Windows.
pub fn raw_input_start(native_window: usize) -> Result<()> {
    check(sys::safe::raw_input_start(native_window))
}

/// Stop Raw Input capture. Safe when capture is already stopped.
pub fn raw_input_stop() {
    sys::safe::raw_input_stop();
}

/// Pop one packet without blocking.
///
/// # Errors
///
/// Returns a native error when capture is unavailable or its ABI differs.
pub fn raw_input_read() -> Result<Option<RawInputEvent>> {
    let mut raw = sys::PwcRawInputEvent::default();
    let status = sys::safe::raw_input_read(&mut raw);
    if status == sys::PWC_EAGAIN {
        return Ok(None);
    }
    check(status)?;
    if raw.struct_size as usize != core::mem::size_of::<sys::PwcRawInputEvent>() {
        return Err(Error::without_detail(sys::PWC_ENOSYS));
    }
    let kind = match raw.kind {
        sys::PWC_RAW_INPUT_KEYBOARD => RawInputKind::Keyboard,
        sys::PWC_RAW_INPUT_MOUSE => RawInputKind::Mouse,
        _ => return Err(Error::without_detail(sys::PWC_EINVAL)),
    };
    Ok(Some(RawInputEvent {
        kind,
        timestamp_ms: raw.timestamp_ms,
        make_code: raw.make_code,
        flags: raw.flags,
        virtual_key: raw.virtual_key,
        button_flags: raw.button_flags,
        delta_x: raw.delta_x,
        delta_y: raw.delta_y,
        wheel_delta: raw.wheel_delta,
    }))
}

/// Return the cumulative number of packets dropped by the bounded queue.
///
/// # Errors
///
/// Returns a native status when the counter cannot be read.
pub fn raw_input_dropped() -> Result<u64> {
    let mut dropped = 0;
    check(sys::safe::raw_input_dropped(&mut dropped))?;
    Ok(dropped)
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
    /// The phase 2 criterion: a change on disk reaches the consumer with no
    /// polling loop anywhere in this test.
    ///
    /// The distinction from `recursive_watch_observes_external_change` is the
    /// whole point. That test calls `read_changes`, which *blocks* â€” it works
    /// only because it dedicates this thread to waiting. Here the kernel owns
    /// the blocking thread and the consumer just drains a port, which is what
    /// the UI needed in order to stop polling.
    #[test]
    fn watch_streams_changes_to_a_port_without_polling() {
        let _guard = KERNEL_LOCK.lock().expect("kernel test lock");
        let root = std::env::temp_dir().join(format!("packwandc-stream-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("create watch root");
        assert_eq!(sys::safe::boot(64, 2), sys::PWC_OK);

        let port = Port::open().expect("open port");
        let watch = FsWatch::open(root.to_str().expect("Windows temp path is UTF-8"))
            .expect("open recursive watch");
        watch.stream_to(&port).expect("start the watch poller");

        let changed = root.join("streamed.txt");
        std::fs::write(&changed, b"changed outside editor").expect("write watched file");

        // Waiting for the message to arrive is not polling the *filesystem* â€”
        // nothing here asks the OS whether anything changed. The kernel's
        // poller is blocked in the platform call and pushes when it returns.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut batches = 0usize;
        while std::time::Instant::now() < deadline {
            if let Some(message) = port.recv().expect("drain the watch port") {
                assert_eq!(message.len(), 4, "each batch is a little-endian u32 count");
                batches += 1;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(batches > 0, "no change batch arrived on the port");

        // Closing the watch is the only cancellation there is: it unblocks the
        // poller from inside the OS. Shutdown joins it, so a failure here hangs
        // rather than failing â€” which is itself the signal.
        drop(watch);
        drop(port);
        sys::safe::shutdown();
        let _ = std::fs::remove_dir_all(root);
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
    /// Stream change batches to `port` on a kernel-owned thread.
    ///
    /// This is the non-polling path. [`read_changes`](Self::read_changes)
    /// blocks the calling thread; this hands the blocking to the kernel and
    /// leaves the caller draining a port at its own pace. Each batch is a
    /// little-endian `u32` change count.
    ///
    /// Dropping the watch stops the stream â€” that is the only cancellation
    /// available, because the kernel's thread is blocked inside the OS.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the watch or port handle is invalid, the port is
    /// not writable, or no stream slots are free.
    pub fn stream_to(&self, port: &Port) -> Result<()> {
        let raw = self.raw.ok_or(Error::without_detail(sys::PWC_EBADF))?;
        check(sys::safe::fs_watch_stream(raw, port.handle()))
    }

    /// Block until a coalesced batch of filesystem changes is available.
    ///
    /// # Errors
    ///
    /// Returns a native watcher or stale-handle error.
    pub fn read_changes(&self) -> Result<usize> {
        let raw = self.raw.ok_or(Error::without_detail(sys::PWC_EBADF))?;
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
