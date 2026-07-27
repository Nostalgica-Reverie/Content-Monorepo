//! Drives the CMake build of the packwandc C core and emits link directives.
//!
//! This file deliberately contains no compilation logic. CMake owns the source
//! list, the C standard, the warning set, and the hardening flags
//! (packwandc.md 6.3); duplicating any of that here would create a second
//! source of truth, and the entire value of the quality gate rests on the
//! gated flags being the shipped flags.
//!
//! CMake is invoked directly rather than through the `cmake` crate. That crate
//! derives the compiler from `cc` and passes `-DCMAKE_C_COMPILER=cl.exe` for
//! MSVC targets, which overrides
//! `scripts/toolchain-windows-msvc.cmake` and lands on a compiler that cannot
//! build C23. Shelling out keeps the toolchain file authoritative, and keeps
//! packwandc's build free of dependencies, matching the rule the C layer
//! itself follows.

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo"),
    );
    // crates/packwandc-sys -> packwandc/
    let c_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("packwandc-sys must live at packwandc/crates/packwandc-sys")
        .to_path_buf();

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let build_dir = out_dir.join("cmake");

    let is_windows = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows");

    // Cargo's `debug` profile maps to CMake's Debug; everything else gets
    // optimisation plus debug info, since a stripped native frame in a
    // backtrace is worthless.
    let build_type = if std::env::var("PROFILE").as_deref() == Ok("debug") {
        "Debug"
    } else {
        "RelWithDebInfo"
    };

    let mut configure = Command::new("cmake");
    configure
        .arg("-S")
        .arg(&c_root)
        .arg("-B")
        .arg(&build_dir)
        // Ninja is required: the Visual Studio generator would ignore the
        // toolchain file's compiler selection.
        .arg("-G")
        .arg("Ninja")
        .arg(format!("-DCMAKE_BUILD_TYPE={build_type}"))
        // Tests are built and run by CTest via `just test-packwandc`, not as a
        // side effect of `cargo build`.
        .arg("-DPWC_BUILD_TESTS=OFF")
        .arg("-DBUILD_TESTING=OFF");

    if is_windows {
        // The C side must be built for the MSVC ABI to link against Rust's
        // x86_64-pc-windows-msvc output. The toolchain file pins clang to that
        // target and locates the MSVC and Windows SDK roots, so no vcvars
        // shell is needed. See packwandc.md 8.1.
        let toolchain = c_root.join("scripts").join("toolchain-windows-msvc.cmake");
        configure.arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", toolchain.display()));
        println!("cargo:rerun-if-changed={}", toolchain.display());
    }

    run(configure, "cmake configure");

    let mut build = Command::new("cmake");
    build
        .arg("--build")
        .arg(&build_dir)
        .arg("--target")
        .arg("packwandc");
    run(build, "cmake build");

    println!("cargo:rustc-link-search=native={}", build_dir.display());
    println!("cargo:rustc-link-lib=static=packwandc");
    if is_windows {
        println!("cargo:rustc-link-lib=Advapi32");
    }

    for path in ["CMakeLists.txt", "kernel", "include", "modules", "arch"] {
        let watched = c_root.join(path);
        if watched.exists() {
            println!("cargo:rerun-if-changed={}", watched.display());
        }
    }
}

fn run(mut command: Command, what: &str) {
    let status = command.status().unwrap_or_else(|err| {
        panic!(
            "packwandc-sys: failed to run {what}: {err}\n\
             CMake >= 3.27 and Ninja must be on PATH to build this workspace \
             (see packwandc.md 6.3 and 12.2)."
        )
    });

    assert!(
        status.success(),
        "packwandc-sys: {what} failed with {status}"
    );
}
