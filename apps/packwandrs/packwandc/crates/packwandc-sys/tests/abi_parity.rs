//! Cross-language ABI parity checks.
//!
//! These are the tests that would have caught a silent FFI mismatch: rather
//! than trusting that the Rust constants in `packwandc-sys` match the C ones,
//! they ask the C library itself and compare. A drift between
//! `uapi/pwc_status.h` and `lib.rs` fails here rather than in a caller that
//! mysteriously stops recognising an error code.

use std::ffi::CStr;

use packwandc_sys as sys;

/// The kernel is a single process-wide instance, so any test that boots it has
/// to be serialised against every other one. Cargo runs the tests in this
/// binary on parallel threads, and a second pwc_boot returns PWC_EAGAIN rather
/// than quietly sharing — which showed up as an unrelated-looking -4 the moment
/// a third booting test was added.
static KERNEL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
fn error_detail_layout_matches_the_wire_abi() {
    // The C side static_asserts this same 40. Both sides asserting it is the
    // point: the struct is read field-by-field across the FFI boundary, so a
    // silent layout drift would not fail to compile — it would hand Rust a
    // pointer read out of the middle of another field.
    assert_eq!(core::mem::size_of::<sys::PwcErrorDetail>(), 40);
    assert_eq!(core::mem::align_of::<sys::PwcErrorDetail>(), 8);
}

#[test]
fn last_error_is_recorded_and_readable() {
    let _guard = KERNEL_LOCK.lock().expect("kernel test lock");
    // Guards against the failure mode this feature actually had: a getter
    // returning a static that nothing ever wrote. That version compiled,
    // linked, and read plausibly at every call site.
    let config = sys::PwcBootConfig {
        handle_capacity: 8,
        worker_count: 1,
    };
    assert_eq!(sys::safe::boot(config.handle_capacity, config.worker_count), sys::PWC_OK);

    let mut port = sys::PwcHandle::default();
    assert_eq!(sys::safe::port_create(&mut port), sys::PWC_OK);
    assert_eq!(sys::safe::handle_close(port), sys::PWC_OK);

    // Closing a stale handle must report the generation mismatch, not succeed.
    let status = sys::safe::handle_close(port);
    assert_eq!(status, sys::PWC_ESTALE);

    let detail = sys::safe::last_error().expect("the core must record a detail for ESTALE");
    assert_eq!(detail.status, sys::PWC_ESTALE);
    assert_eq!(detail.module, "core");
    assert!(detail.line > 0, "line was {}", detail.line);
    // -ffile-prefix-map rewrites __FILE__ to a repo-relative path, so this is
    // stable across machines rather than an absolute build path.
    assert!(
        detail.file.contains("handle.c"),
        "unexpected file {:?}",
        detail.file
    );
    assert!(!detail.message.is_empty());

    sys::safe::shutdown();
}

#[test]
fn trace_record_layout_matches_the_wire_abi() {
    // The C side static_asserts the same 56. Both sides assert it because the
    // struct is read field-by-field across FFI: a layout drift would not fail
    // to compile, it would hand Rust a pointer read from the middle of another
    // field.
    assert_eq!(core::mem::size_of::<sys::PwcTraceRecord>(), 56);
    assert_eq!(core::mem::align_of::<sys::PwcTraceRecord>(), 8);
}

#[test]
fn ktrace_drains_recorded_failures() {
    let _guard = KERNEL_LOCK.lock().expect("kernel test lock");
    assert_eq!(sys::safe::boot(8, 1), sys::PWC_OK);

    // Boot is not silent: each module's init and the kernel itself emit INFO
    // notes, so those are cleared before this test's own record is observed.
    // Asserting a non-zero count keeps module bring-up observable rather than
    // letting it quietly stop happening.
    let mut boot_notes = 0usize;
    while sys::safe::ktrace_drain()
        .expect("drain must not error on a booted core")
        .is_some()
    {
        boot_notes += 1;
    }
    assert!(boot_notes > 0, "boot must trace its own module bring-up");

    // Any recorded failure is also traced — the detail record and the ring
    // share one choke point in kernel/status.c.
    let stale = sys::PwcHandle {
        index: 4000,
        generation: 7,
    };
    assert_eq!(sys::safe::handle_close(stale), sys::PWC_EBADF);

    let record = sys::safe::ktrace_drain()
        .expect("drain must not error")
        .expect("the failure above must have been traced");
    assert_eq!(record.status, sys::PWC_EBADF);
    assert_eq!(record.level, sys::trace_level::ERROR);
    assert_eq!(record.module, "core");
    assert!(record.line > 0, "line was {}", record.line);
    assert!(
        record.file.contains("handle.c"),
        "unexpected file {:?}",
        record.file
    );

    // Drained back to empty, and nothing was dropped at this volume.
    assert_eq!(sys::safe::ktrace_drain().expect("drain must not error"), None);
    assert_eq!(sys::safe::ktrace_dropped().expect("drop count readable"), 0);

    sys::safe::shutdown();
}

#[test]
fn sh_command_layout_matches_the_wire_abi() {
    // 8 header bytes, PWC_SH_MAX_ARGS lengths, then the word array. Asserted on
    // both sides because the struct is read field-by-field across FFI.
    let expected = 8 + (4 * sys::PWC_SH_MAX_ARGS) + (sys::PWC_SH_MAX_ARGS * sys::PWC_SH_MAX_ARG);
    assert_eq!(core::mem::size_of::<sys::PwcShCommand>(), expected);
}

#[test]
fn sh_parse_applies_the_kernels_quoting_rules() {
    // The point of exposing the parser: the UI must not reimplement quoting.
    let words = sys::safe::sh_parse(b"echo \"hello world\" --flag")
        .expect("a well-formed line must parse");
    assert_eq!(words.argc, 3);
    assert_eq!(&words.argv[1][..words.arglen[1] as usize], b"hello world");

    // Malformed input is rejected rather than silently truncated.
    assert!(sys::safe::sh_parse(b"echo \"unterminated").is_err());
    assert!(sys::safe::sh_parse(b"echo \"bad \\q\"").is_err());
}

#[test]
fn sh_exec_runs_builtins_and_defers_the_rest() {
    let _guard = KERNEL_LOCK.lock().expect("kernel test lock");
    assert_eq!(sys::safe::boot(8, 1), sys::PWC_OK);

    let mut port = sys::PwcHandle::default();
    assert_eq!(sys::safe::port_create(&mut port), sys::PWC_OK);

    // A built-in runs in the kernel and writes its output to the port.
    let (status, _) = sys::safe::sh_exec(port, b"echo hello");
    assert_eq!(status, sys::PWC_OK);

    let mut buffer = [0u8; 256];
    let mut len = 0usize;
    assert_eq!(
        sys::safe::ipc_recv(port, &mut buffer, &mut len),
        sys::PWC_OK
    );
    assert_eq!(&buffer[..len], b"hello");

    // A host verb parses cleanly and is handed back rather than run.
    let (status, command) = sys::safe::sh_exec(port, b"pack list --side client");
    assert_eq!(status, sys::PWC_ENOSYS);
    assert_eq!(command.argc, 4);
    assert_eq!(&command.argv[0][..command.arglen[0] as usize], b"pack");

    // There is no path to process execution: an external-looking command is
    // simply not a built-in, not an attempt to spawn anything.
    let (status, _) = sys::safe::sh_exec(port, b"/bin/sh -c whoami");
    assert_eq!(status, sys::PWC_ENOSYS);

    sys::safe::shutdown();
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
