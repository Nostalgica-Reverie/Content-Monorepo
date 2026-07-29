# packwandc -- the Packwand native core. See packwandc.md.
#
# CMake is authoritative for the C build (packwandc.md 6.3): the Rust crate
# packwandc-sys drives this file rather than compiling the sources itself, so
# the flags that ship are exactly the flags the quality gate ran against.

cmake_minimum_required(VERSION 3.27)

# Select Zig before project() performs its compiler probe. Windows supplies the
# same values through its ABI toolchain file; this is the native fallback for
# direct CMake and Cargo builds on other platforms.
if(NOT DEFINED CMAKE_C_COMPILER)
    find_program(PWC_ZIG NAMES zig zig.exe REQUIRED)
    set(CMAKE_C_COMPILER "${PWC_ZIG}" CACHE FILEPATH "packwandc C compiler")
    set(CMAKE_C_COMPILER_ARG1 cc CACHE STRING "packwandc C compiler argument")
endif()

project(
    packwandc
    VERSION 0.1.0
    DESCRIPTION "Packwand native core: OS-level kernel, modules and bindings"
    LANGUAGES C)

# Zig compiles both the remaining C translation units (through `zig cc`) and
# the progressively ported implementation objects.  C headers remain the ABI
# boundary, so dependent Rust and C code does not need to change as internals
# move to Zig.
find_program(PWC_ZIG NAMES zig zig.exe REQUIRED)

# C23. CMake emits the right flag spelling per compiler version, which matters
# because -std=c23 only exists from clang 17 / gcc 14 (packwandc.md 7.1).
set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF) # no GNU extensions outside arch/

set(CMAKE_EXPORT_COMPILE_COMMANDS ON) # clang-tidy and scan-build consume this

if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
    set(CMAKE_BUILD_TYPE Debug CACHE STRING "" FORCE)
endif()

include(CTest)

# --- options ----------------------------------------------------------------

option(PWC_ENABLE_WINDOW "Build pwwin/pwgpu (phase 5, not adopted -- packwandc.md 5.7)" OFF)
option(PWC_WERROR "Treat warnings as errors" ON)
option(PWC_BUILD_TESTS "Build the unit test suite" ON)

# --- the quality gate, as compile flags -------------------------------------
#
# packwandc.md 7.2 and 7.3. Carried on an INTERFACE target so that every
# packwandc target gets an identical flag set and no target can quietly opt
# out. Third-party code would link the plain library target instead -- there is
# none, by design (packwandc.md 1.1).

add_library(pwc_gate INTERFACE)

if(CMAKE_C_COMPILER_ID MATCHES "Clang|GNU")
    target_compile_options(
        pwc_gate
        INTERFACE
            # 7.2 -- warnings, all errors
            -Wall
            -Wextra
            -Wpedantic
            -Wconversion
            -Wsign-conversion
            -Wshadow
            -Wcast-qual
            -Wcast-align
            -Wstrict-prototypes
            -Wmissing-prototypes
            -Wmissing-declarations
            -Wredundant-decls
            -Wold-style-definition
            -Wbad-function-cast
            -Wundef
            -Wwrite-strings
            -Wpointer-arith
            -Wvla
            -Walloca
            -Wdouble-promotion
            -Wfloat-equal
            -Wformat=2
            -Wformat-security
            -Wswitch-enum
            -Wimplicit-fallthrough
            -Wnull-dereference
            -Wunused-macros
            -Wunused-result
            -Wstrict-overflow=2
            -Wstack-protector
            # 7.3 -- codegen. -fno-strict-aliasing and -fwrapv are Linux's own
            # choices: they trade optimiser freedom for the removal of two
            # large classes of undefined behaviour. This layer marshals OS ABI
            # calls rather than crunching numbers, so the trade is free.
            -fno-strict-aliasing
            -fno-delete-null-pointer-checks
            -fwrapv
            -fstack-protector-strong
            -ftrivial-auto-var-init=zero)

    if(PWC_WERROR)
        target_compile_options(pwc_gate INTERFACE -Werror)
    endif()

    # Reproducible builds: strip the absolute build path out of __FILE__ and
    # debug info (packwandc.md 7.3).
    target_compile_options(pwc_gate INTERFACE "-ffile-prefix-map=${CMAKE_SOURCE_DIR}=packwandc")

    # Hardening flags that are not universally supported. -fcf-protection is
    # x86-only; -fstack-clash-protection is a no-op on Windows targets;
    # _FORTIFY_SOURCE requires optimisation to do anything and warns at -O0.
    #
    # The probe must reject "accepted but unused". clang takes
    # -fstack-clash-protection for x86_64-pc-windows-msvc without complaint at
    # parse time, then emits -Wunused-command-line-argument at compile time,
    # which -Werror turns into a hard build failure. Probing with that warning
    # promoted to an error is what makes the detection honest.
    include(CheckCCompilerFlag)
    foreach(_flag -fstack-clash-protection -fcf-protection=full)
        string(MAKE_C_IDENTIFIER "PWC_HAVE${_flag}" _flag_var)
        set(CMAKE_REQUIRED_FLAGS "-Werror=unused-command-line-argument")
        check_c_compiler_flag("${_flag}" ${_flag_var})
        unset(CMAKE_REQUIRED_FLAGS)
        if(${_flag_var})
            target_compile_options(pwc_gate INTERFACE "${_flag}")
        endif()
    endforeach()

    target_compile_definitions(
        pwc_gate INTERFACE $<$<NOT:$<CONFIG:Debug>>:_FORTIFY_SOURCE=3>)
else()
    message(WARNING "packwandc: compiler '${CMAKE_C_COMPILER_ID}' is not a gated configuration. "
                    "The quality gate in packwandc.md 7 assumes clang or gcc.")
endif()

# --- sanitizers -------------------------------------------------------------
#
# Linux only, and not by choice (packwandc.md 7.4): clang rejects
# -fsanitize=thread outright for x86_64-pc-windows-msvc, and the MinGW clang
# install ships no compiler-rt to link ASan against. Windows verification is
# build + unit tests, never sanitized.

set(PWC_SANITIZE "" CACHE STRING "Sanitizers: none, asan-ubsan, tsan")
set_property(CACHE PWC_SANITIZE PROPERTY STRINGS "" "asan-ubsan" "tsan")

if(PWC_SANITIZE)
    if(WIN32)
        message(FATAL_ERROR
                "packwandc: PWC_SANITIZE=${PWC_SANITIZE} is not available on Windows. "
                "TSan does not exist for the MSVC target and no compiler-rt is installed. "
                "Run sanitizer legs on Linux -- see packwandc.md 7.4.")
    endif()

    if(PWC_SANITIZE STREQUAL "asan-ubsan")
        set(_pwc_san_flags -fsanitize=address,undefined -fno-sanitize-recover=all
                           -fno-omit-frame-pointer)
    elseif(PWC_SANITIZE STREQUAL "tsan")
        set(_pwc_san_flags -fsanitize=thread -fno-omit-frame-pointer)
    else()
        message(FATAL_ERROR "packwandc: unknown PWC_SANITIZE value '${PWC_SANITIZE}'")
    endif()

    target_compile_options(pwc_gate INTERFACE ${_pwc_san_flags} -g)
    target_link_options(pwc_gate INTERFACE ${_pwc_san_flags})
endif()

# --- coverage ---------------------------------------------------------------
#
# Reported, not enforced, until phase 1 produces real numbers (packwandc.md
# 7.4 and 12.3).

option(PWC_COVERAGE "Instrument for llvm-cov coverage reporting" OFF)
if(PWC_COVERAGE)
    target_compile_options(pwc_gate INTERFACE -fprofile-instr-generate -fcoverage-mapping)
    target_link_options(pwc_gate INTERFACE -fprofile-instr-generate -fcoverage-mapping)
endif()

# --- the library ------------------------------------------------------------

set(PWC_KERNEL_SOURCES kernel/status.c kernel/syscall.c kernel/boot.c kernel/handle.c kernel/ipc.c kernel/wait.c kernel/sched.c kernel/module.c kernel/ktrace.c kernel/lockdep.c modules/pwproc/pwproc.c modules/pwkeys/pwkeys.c modules/pwfs/pwfs.c modules/pwsh/pwsh.c modules/pwsh/lexer.c)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(PWC_ZIG_OPTIMIZE Debug)
else()
    # ReleaseSafe retains Zig's runtime overflow checks while matching the
    # core's existing hardening-oriented release profile.
    set(PWC_ZIG_OPTIMIZE ReleaseSafe)
endif()

# CMake has no first-class Zig language support.  Build Zig sources into
# objects explicitly and add them to the ordinary static library; this keeps
# CMake authoritative for the C ABI, tests, and platform link dependencies.
set(PWC_ZIG_MEMORY_OBJECT "${CMAKE_CURRENT_BINARY_DIR}/pwc_memory${CMAKE_C_OUTPUT_EXTENSION}")
add_custom_command(
    OUTPUT "${PWC_ZIG_MEMORY_OBJECT}"
    COMMAND "${PWC_ZIG}" build-obj "-O${PWC_ZIG_OPTIMIZE}"
            "-femit-bin=${PWC_ZIG_MEMORY_OBJECT}"
            -I "${CMAKE_CURRENT_SOURCE_DIR}/include"
            "${CMAKE_CURRENT_SOURCE_DIR}/zig/memory.zig"
    DEPENDS zig/memory.zig
            include/packwandc/kernel/pwc_arena.h
            include/packwandc/kernel/pwc_slab.h
            include/packwandc/uapi/pwc_status.h
    VERBATIM)
set_source_files_properties("${PWC_ZIG_MEMORY_OBJECT}" PROPERTIES GENERATED TRUE EXTERNAL_OBJECT TRUE)

# Exactly one arch backend is linked per build. arch/common is the portable
# fallback: it satisfies the link and returns PWC_ENOSYS, which is the honest
# answer on a platform nobody has written a backend for -- but it must never be
# selected for a platform that has one, or every subsystem silently does
# nothing. That is what the Linux branch below exists to prevent; before it,
# CI (which is Linux-only) exercised the ENOSYS stubs and reported success.
#
# Both directories arrive in phase 1, not phase 2 as packwandc.md 10 originally
# said: pwc_wait needs an arch backend before any subsystem exists.
if(WIN32)
    list(APPEND PWC_KERNEL_SOURCES arch/win32/wait.c arch/win32/proc.c arch/win32/keys.c arch/win32/fs.c arch/win32/thread.c arch/win32/raw_input.c)
elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    list(APPEND PWC_KERNEL_SOURCES arch/linux/wait.c arch/linux/proc.c arch/linux/keys.c arch/linux/fs.c arch/linux/thread.c arch/common/raw_input.c)
else()
    # macOS lands here deliberately. packwandc.md 8.2: arch/darwin cannot be
    # verified by CI or on the reference machine, and the spec's rule is that
    # it should not be written until someone can actually run it.
    list(APPEND PWC_KERNEL_SOURCES arch/common/wait.c arch/common/proc.c arch/common/keys.c arch/common/fs.c arch/common/thread.c arch/common/raw_input.c)
endif()

add_library(packwandc STATIC ${PWC_KERNEL_SOURCES} "${PWC_ZIG_MEMORY_OBJECT}")
add_library(packwandc::packwandc ALIAS packwandc)

target_include_directories(
    packwandc
    PUBLIC $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
           $<INSTALL_INTERFACE:include>)

target_link_libraries(packwandc PRIVATE pwc_gate)
if(WIN32)
    target_link_libraries(packwandc PRIVATE Advapi32 User32)
else()
    # pthreads. glibc 2.34+ folds libpthread into libc, but older ones and musl
    # still need the explicit link, and Threads::Threads resolves both.
    set(THREADS_PREFER_PTHREAD_FLAG ON)
    find_package(Threads REQUIRED)
    target_link_libraries(packwandc PUBLIC Threads::Threads)
endif()

set_target_properties(packwandc PROPERTIES C_VISIBILITY_PRESET hidden POSITION_INDEPENDENT_CODE ON)

# --- tests ------------------------------------------------------------------

if(PWC_BUILD_TESTS AND BUILD_TESTING)
    add_executable(pwc_test_core tests/unit/test_core.c)
    target_include_directories(pwc_test_core PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/tests/unit)
    target_link_libraries(pwc_test_core PRIVATE packwandc pwc_gate)
add_test(NAME core COMMAND pwc_test_core)
    # Lockdep aborts only when NDEBUG is absent (packwandc.md 3.6: "debug and
    # sanitizer builds"), so the test is registered only where the behaviour it
    # checks is compiled in. Registering it unconditionally would make release
    # configurations fail for the wrong reason.
    if(NOT CMAKE_BUILD_TYPE STREQUAL "Release" AND NOT CMAKE_BUILD_TYPE STREQUAL "MinSizeRel")
        add_executable(pwc_test_lockdep_abort tests/unit/test_lockdep_abort.c)
        target_include_directories(pwc_test_lockdep_abort PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/tests/unit)
        target_link_libraries(pwc_test_lockdep_abort PRIVATE packwandc pwc_gate)
        add_test(NAME lockdep_abort COMMAND pwc_test_lockdep_abort)

        # Checked on OUTPUT, not exit code. WILL_FAIL alone accepted any
        # non-zero exit, which the test could produce by simply returning --
        # so it passed whether or not lockdep did anything (handoff trap 15).
        #
        # PASS_REGULAR_EXPRESSION makes CTest ignore the exit status, which is
        # what we want: abort() is a signal death whose code varies by
        # platform. FAIL_REGULAR_EXPRESSION takes precedence over it, so a run
        # that reaches the marker fails even though the banner also matched.
        set_tests_properties(
            lockdep_abort
            PROPERTIES PASS_REGULAR_EXPRESSION "lockdep: lock order violation"
                       FAIL_REGULAR_EXPRESSION "NOT REACHED|SETUP FAILED")
    endif()
endif()

# --- fuzzing ----------------------------------------------------------------
#
# packwandc.md 7.4 requires a harness at every parsing boundary. pwc_sh_parse
# is the first one in the tree -- it is handed bytes a user typed.
#
# Linux and clang only, for the same structural reason as the sanitizer legs:
# -fsanitize=fuzzer does not exist for the MSVC target and the MinGW clang
# install ships no compiler-rt. Guarded rather than probed, so a Windows
# developer enabling PWC_FUZZ gets a clear message instead of a link error.

option(PWC_FUZZ "Build the libFuzzer harnesses (Linux + clang only)" OFF)

if(PWC_FUZZ)
    if(WIN32 OR NOT CMAKE_C_COMPILER_ID MATCHES "Clang")
        message(FATAL_ERROR
                "packwandc: PWC_FUZZ needs clang on a non-Windows host. "
                "-fsanitize=fuzzer has no MSVC-target support and the MinGW clang "
                "install has no compiler-rt -- see packwandc.md 7.4.")
    endif()

    add_executable(pwc_fuzz_sh_parse tests/fuzz/fuzz_sh_parse.c)
    target_link_libraries(pwc_fuzz_sh_parse PRIVATE packwandc pwc_gate)
    # ASan and UBSan alongside the fuzzer: the interesting findings are memory
    # errors on inputs the parser accepts, not just hangs.
    target_compile_options(pwc_fuzz_sh_parse PRIVATE -fsanitize=fuzzer,address,undefined
                                                     -fno-sanitize-recover=all)
    target_link_options(pwc_fuzz_sh_parse PRIVATE -fsanitize=fuzzer,address,undefined)

    # A short smoke pass over the committed corpus, for PR CI. The long soak is
    # a scheduled job and does not run here (packwandc.md 9.2).
    add_test(NAME fuzz_sh_parse_smoke
             COMMAND pwc_fuzz_sh_parse -runs=20000
                     ${CMAKE_CURRENT_SOURCE_DIR}/tests/fuzz/corpus/sh_parse)
endif()

# --- install ----------------------------------------------------------------
#
# packwandc-sys consumes the build tree directly, so this exists for
# completeness and for out-of-tree consumers, not for the normal path.

install(TARGETS packwandc EXPORT packwandcTargets ARCHIVE DESTINATION lib)
install(DIRECTORY include/ DESTINATION include)










