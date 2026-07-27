//! Cross-language ABI parity checks.
//!
//! These are the tests that would have caught a silent FFI mismatch: rather
//! than trusting that the Rust constants in `packwandc-sys` match the C ones,
//! they ask the C library itself and compare. A drift between
//! `uapi/pwc_status.h` and `lib.rs` fails here rather than in a caller that
//! mysteriously stops recognising an error code.

use std::ffi::CStr;

use packwandc_sys as sys;

/// Ask C for a status code's name. Safe because `pwc_sys_status_name` is
/// documented never to return NULL and to return a pointer to a string
/// literal with static lifetime.
fn status_name(code: i32) -> String {
    let ptr = unsafe { sys::pwc_sys_status_name(code) };
    assert!(!ptr.is_null(), "pwc_sys_status_name({code}) returned NULL");
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

#[test]
fn abi_version_matches_the_compiled_kernel() {
    let mut major = u32::MAX;
    let mut minor = u32::MAX;

    let st = unsafe { sys::pwc_sys_version(&raw mut major, &raw mut minor) };

    assert_eq!(st, sys::PWC_OK);
    assert_eq!(
        major,
        sys::PWC_ABI_VERSION_MAJOR,
        "ABI major drifted from the C headers"
    );
    assert_eq!(
        minor,
        sys::PWC_ABI_VERSION_MINOR,
        "ABI minor drifted from the C headers"
    );
}

#[test]
fn version_syscall_rejects_null() {
    let mut scratch = 0u32;

    assert_eq!(
        unsafe { sys::pwc_sys_version(core::ptr::null_mut(), &raw mut scratch) },
        sys::PWC_EINVAL
    );
    assert_eq!(
        unsafe { sys::pwc_sys_version(&raw mut scratch, core::ptr::null_mut()) },
        sys::PWC_EINVAL
    );
}

#[test]
fn every_status_constant_matches_c() {
    // (Rust constant, its C spelling). If a code is added to PWC_STATUS_LIST
    // in C but not here, the count assertion at the bottom catches it.
    let table: &[(i32, &str)] = &[
        (sys::PWC_OK, "PWC_OK"),
        (sys::PWC_EINVAL, "PWC_EINVAL"),
        (sys::PWC_ENOENT, "PWC_ENOENT"),
        (sys::PWC_EPERM, "PWC_EPERM"),
        (sys::PWC_EAGAIN, "PWC_EAGAIN"),
        (sys::PWC_ENOMEM, "PWC_ENOMEM"),
        (sys::PWC_EBADF, "PWC_EBADF"),
        (sys::PWC_ESTALE, "PWC_ESTALE"),
        (sys::PWC_ENOSYS, "PWC_ENOSYS"),
        (sys::PWC_EIO, "PWC_EIO"),
        (sys::PWC_ETIMEDOUT, "PWC_ETIMEDOUT"),
        (sys::PWC_ECANCELED, "PWC_ECANCELED"),
        (sys::PWC_EOVERFLOW, "PWC_EOVERFLOW"),
    ];

    for &(code, expected) in table {
        assert_eq!(
            status_name(code),
            expected,
            "status {code} disagrees across the FFI boundary"
        );
    }

    // Walk downward past the last known code. The first unknown one proves we
    // have the complete set: if C gained a code Rust does not know about, this
    // finds it instead of silently ignoring it.
    let lowest = table
        .iter()
        .map(|&(c, _)| c)
        .min()
        .expect("table is not empty");
    assert_eq!(
        status_name(lowest - 1),
        "PWC_EUNKNOWN",
        "C defines a status code below {lowest} that packwandc-sys does not mirror"
    );
}

#[test]
fn unknown_status_is_never_null() {
    // The never-NULL promise is load-bearing: callers log this directly.
    assert_eq!(status_name(-9999), "PWC_EUNKNOWN");
    assert_eq!(status_name(i32::MIN), "PWC_EUNKNOWN");
}

#[test]
fn handle_layout_matches_the_wire_abi() {
    assert_eq!(core::mem::size_of::<sys::PwcHandle>(), 8);
    assert_eq!(core::mem::align_of::<sys::PwcHandle>(), 4);

    // Zeroed must be invalid by construction, matching PWC_HANDLE_INVALID.
    let zeroed = sys::PwcHandle::default();
    assert_eq!(zeroed.index, 0);
    assert_eq!(zeroed.generation, 0);
}

#[test]
fn syscall_count_matches_the_ledger() {
    // tests/golden/syscalls.txt is the frozen ledger; SYSCALL_COUNT is
    // generated from the same file, so this catches a hand-edited bindings.rs.
    let ledger = include_str!("../../../tests/golden/syscalls.txt");
    let entries = ledger.lines().filter(|l| !l.trim().is_empty()).count();

    assert_eq!(
        entries,
        sys::SYSCALL_COUNT,
        "bindings.rs disagrees with the golden ledger"
    );
}
