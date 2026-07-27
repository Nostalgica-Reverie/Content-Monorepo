# packwandc -- the Packwand native core. See packwandc.md.
#
# CMake is authoritative for the C build (packwandc.md 6.3): the Rust crate
# packwandc-sys drives this file rather than compiling the sources itself, so
# the flags that ship are exactly the flags the quality gate ran against.

cmake_minimum_required(VERSION 3.27)

project(
    packwandc
    VERSION 0.1.0
    DESCRIPTION "Packwand native core: OS-level kernel, modules and bindings"
    LANGUAGES C)

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

set(PWC_KERNEL_SOURCES kernel/status.c kernel/syscall.c kernel/boot.c kernel/handle.c kernel/ipc.c kernel/wait.c kernel/arena.c kernel/slab.c kernel/sched.c kernel/module.c kernel/ktrace.c kernel/lockdep.c modules/pwproc/pwproc.c modules/pwkeys/pwkeys.c modules/pwfs/pwfs.c)

# Phase 0 has no modules and no arch backends; both directories arrive with
# their first subsystem in phase 2 (packwandc.md 10).

if(WIN32)
    list(APPEND PWC_KERNEL_SOURCES arch/win32/wait.c arch/win32/proc.c arch/win32/keys.c arch/win32/fs.c)
else()
    list(APPEND PWC_KERNEL_SOURCES arch/common/wait.c arch/common/proc.c arch/common/keys.c arch/common/fs.c)
endif()

add_library(packwandc STATIC ${PWC_KERNEL_SOURCES})
add_library(packwandc::packwandc ALIAS packwandc)

target_include_directories(
    packwandc
    PUBLIC $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
           $<INSTALL_INTERFACE:include>)

target_link_libraries(packwandc PRIVATE pwc_gate)
if(WIN32)
    target_link_libraries(packwandc PRIVATE Advapi32)
endif()

set_target_properties(packwandc PROPERTIES C_VISIBILITY_PRESET hidden POSITION_INDEPENDENT_CODE ON)

# --- tests ------------------------------------------------------------------

if(PWC_BUILD_TESTS AND BUILD_TESTING)
    add_executable(pwc_test_core tests/unit/test_core.c)
    target_include_directories(pwc_test_core PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/tests/unit)
    target_link_libraries(pwc_test_core PRIVATE packwandc pwc_gate)
add_test(NAME core COMMAND pwc_test_core)
    add_executable(pwc_test_lockdep_abort tests/unit/test_lockdep_abort.c)
    target_include_directories(pwc_test_lockdep_abort PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/tests/unit)
    target_link_libraries(pwc_test_lockdep_abort PRIVATE packwandc pwc_gate)
    add_test(NAME lockdep_abort COMMAND pwc_test_lockdep_abort)
    set_tests_properties(lockdep_abort PROPERTIES WILL_FAIL TRUE)
endif()

# --- install ----------------------------------------------------------------
#
# packwandc-sys consumes the build tree directly, so this exists for
# completeness and for out-of-tree consumers, not for the normal path.

install(TARGETS packwandc EXPORT packwandcTargets ARCHIVE DESTINATION lib)
install(DIRECTORY include/ DESTINATION include)










