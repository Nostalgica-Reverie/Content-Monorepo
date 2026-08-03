//! Boots and owns the packwandc native core for the Packwand desktop process.
//!
//! In the Linux analogy this crate is `init`: the first userland thing the
//! kernel hands control to, responsible for bringing the system up in the
//! right order and for owning it until shutdown. It is the only crate that
//! should ever call the boot and shutdown entry points — everything else takes
//! a already-running core for granted.
//!
//! # Phase 0
//!
//! There is no kernel state to boot yet, so [`Host::start`] currently does one
//! genuinely useful thing: it refuses to run against a native core whose ABI
//! this build does not understand. Arena sizing, the worker pool, module
//! initialisation ordering, and the ktrace drain arrive in phase 1 — see
//! The host-side adapter for the native core.

#![forbid(unsafe_code)]

fn sys_boot() -> i32 {
    packwandc_sys::safe::boot(256, 1)
}

use core::fmt;

use packwandc::AbiVersion;

/// Why the native core could not be brought up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartError {
    /// The linked core implements an ABI major version this build does not
    /// understand. Unrecoverable: syscall numbering or struct layouts have
    /// changed underneath us.
    AbiMismatch {
        /// What this build was compiled against.
        expected_major: u32,
        /// What the linked core reports.
        found: AbiVersion,
    },
    /// The core rejected a startup call.
    Native(packwandc::Error),
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbiMismatch {
                expected_major,
                found,
            } => write!(
                f,
                "packwandc ABI mismatch: this build expects major {expected_major}, \
                 the linked core reports {found}"
            ),
            Self::Native(err) => write!(f, "packwandc failed to start: {err}"),
        }
    }
}

impl core::error::Error for StartError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Native(err) => Some(err),
            Self::AbiMismatch { .. } => None,
        }
    }
}

/// A running packwandc core.
///
/// Held for the lifetime of the process. Dropping it shuts the core down; in
/// phase 0 that is a no-op, but callers should already treat the value as
/// owning the core rather than as a token to discard.
#[derive(Debug)]
pub struct Host {
    abi: AbiVersion,
}

impl Host {
    /// Bring the native core up.
    ///
    /// # Errors
    ///
    /// Returns [`StartError::AbiMismatch`] if the linked core speaks an
    /// incompatible ABI major version, or [`StartError::Native`] if the core
    /// rejects a startup call.
    pub fn start() -> Result<Self, StartError> {
        if let Some(error) = packwandc::Error::from_status(sys_boot()) {
            return Err(StartError::Native(error));
        }
        let abi = packwandc::abi_version().map_err(StartError::Native)?;

        // Fail loudly and immediately rather than discovering the mismatch as
        // a corrupted struct several syscalls later.
        packwandc::check_abi_compatibility().map_err(|_| StartError::AbiMismatch {
            expected_major: packwandc::expected_abi_major(),
            found: abi,
        })?;

        Ok(Self { abi })
    }

    /// The ABI version the running core implements.
    #[must_use]
    pub const fn abi(&self) -> AbiVersion {
        self.abi
    }

    /// Drain the core's trace ring, handing each record to `sink`.
    ///
    /// Returns the number of records drained. Call this periodically — the
    /// ring holds a bounded number of records and discards the newest once
    /// full rather than blocking the kernel thread that produced them, so a
    /// slow drain loses trace, never correctness.
    ///
    /// The host owns this drain because the ring has a single read cursor
    /// (see [`packwandc::trace_drain`]). Two drains racing would split the
    /// stream rather than duplicating it, which is the more confusing failure.
    ///
    /// # Errors
    ///
    /// Returns [`packwandc::Error`] if the core rejects a drain call. Records
    /// already passed to `sink` stay passed — this does not roll back.
    pub fn drain_trace<F>(&self, mut sink: F) -> Result<usize, packwandc::Error>
    where
        F: FnMut(&packwandc::TraceRecord),
    {
        let mut drained = 0usize;
        while let Some(record) = packwandc::trace_drain()? {
            sink(&record);
            drained += 1;
        }
        Ok(drained)
    }

    /// Trace records the core discarded because its ring was full.
    ///
    /// Cumulative since boot. Worth surfacing alongside drained records: a
    /// rising count means the drain is not keeping up and the log has holes,
    /// which is otherwise invisible.
    ///
    /// # Errors
    ///
    /// Returns [`packwandc::Error`] if the core is not booted.
    pub fn trace_dropped(&self) -> Result<u64, packwandc::Error> {
        packwandc::trace_dropped()
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        packwandc_sys::safe::shutdown();
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_against_the_linked_core() {
        let host =
            Host::start().expect("the linked core must be ABI-compatible with its own build");
        assert_eq!(host.abi().major, packwandc::expected_abi_major());
    }

    #[test]
    fn start_error_displays_usefully() {
        let err = StartError::AbiMismatch {
            expected_major: 0,
            found: AbiVersion { major: 9, minor: 3 },
        };
        let text = err.to_string();
        assert!(text.contains("expects major 0"), "{text}");
        assert!(text.contains("9.3"), "{text}");
    }
}
