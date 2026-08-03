# CMake toolchain: build packwandc for the MSVC ABI using Zig's clang frontend.
#
# WHY THIS EXISTS
#
# Rust's host toolchain on Windows is x86_64-pc-windows-msvc, but the clang
# typically on PATH here is a MinGW build that defaults to
# x86_64-w64-windows-gnu. Objects from that default CANNOT link into Rust's
# output. This file pins clang to the MSVC ABI and points it at the MSVC and
# Windows SDK headers and import libraries explicitly, so the build does not
# depend on having run vcvarsall.bat first.
#
# Override any of the cache variables below if your install differs:
#   cmake -DPWC_MSVC_ROOT=... -DPWC_WINSDK_ROOT=... -DPWC_WINSDK_VERSION=...

set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_SYSTEM_PROCESSOR AMD64)

# --- locate MSVC and the Windows SDK ----------------------------------------

if(NOT DEFINED PWC_MSVC_ROOT)
    file(GLOB _pwc_msvc_candidates
         "C:/Program Files/Microsoft Visual Studio/2022/*/VC/Tools/MSVC/*"
         "C:/Program Files (x86)/Microsoft Visual Studio/2022/*/VC/Tools/MSVC/*")
    list(SORT _pwc_msvc_candidates)
    list(REVERSE _pwc_msvc_candidates) # highest version first
    list(LENGTH _pwc_msvc_candidates _pwc_msvc_count)
    if(_pwc_msvc_count EQUAL 0)
        message(FATAL_ERROR
                "packwandc: no MSVC toolset found. Install the Visual Studio Build Tools "
                "'Desktop development with C++' workload, or pass -DPWC_MSVC_ROOT=<path>.")
    endif()
    list(GET _pwc_msvc_candidates 0 PWC_MSVC_ROOT)
endif()

if(NOT DEFINED PWC_WINSDK_ROOT)
    set(PWC_WINSDK_ROOT "C:/Program Files (x86)/Windows Kits/10")
endif()

if(NOT DEFINED PWC_WINSDK_VERSION)
    file(GLOB _pwc_sdk_candidates "${PWC_WINSDK_ROOT}/Include/*")
    list(SORT _pwc_sdk_candidates)
    list(REVERSE _pwc_sdk_candidates)
    list(LENGTH _pwc_sdk_candidates _pwc_sdk_count)
    if(_pwc_sdk_count EQUAL 0)
        message(FATAL_ERROR
                "packwandc: no Windows SDK found under ${PWC_WINSDK_ROOT}/Include. "
                "Pass -DPWC_WINSDK_ROOT=<path> -DPWC_WINSDK_VERSION=<version>.")
    endif()
    list(GET _pwc_sdk_candidates 0 _pwc_sdk_newest)
    get_filename_component(PWC_WINSDK_VERSION "${_pwc_sdk_newest}" NAME)
endif()

message(STATUS "packwandc: MSVC toolset  ${PWC_MSVC_ROOT}")
message(STATUS "packwandc: Windows SDK   ${PWC_WINSDK_ROOT} (${PWC_WINSDK_VERSION})")

# --- compiler ---------------------------------------------------------------

find_program(PWC_LLD_LINK NAMES lld-link lld-link.exe REQUIRED)
# NOTE: llvm-lib, not `lib` or `ar`. GNU ar produces archives MSVC's linker
# will not accept, and plain `link.exe` on PATH under Git Bash resolves to
# coreutils' `link`.
find_program(PWC_LLVM_LIB NAMES llvm-lib llvm-lib.exe REQUIRED)

find_program(PWC_ZIG NAMES zig zig.exe REQUIRED)
# CMake 3.27 does not consistently carry CMAKE_C_COMPILER_ARG1 through its
# compiler-identification probe. Keep Zig's `cc` subcommand in the compiler
# list so every probe and build invocation is `zig cc ...`.
set(CMAKE_C_COMPILER "${PWC_ZIG};cc" CACHE STRING "packwandc C compiler" FORCE)
set(CMAKE_C_COMPILER_ARG1 cc CACHE STRING "packwandc C compiler argument" FORCE)
set(CMAKE_C_COMPILER_TARGET x86_64-pc-windows-msvc)
set(CMAKE_LINKER "${PWC_LLD_LINK}")
set(CMAKE_AR "${PWC_LLVM_LIB}")

# Because the compiler is invoked as `clang` rather than `clang-cl`, CMake
# treats the frontend as GNU-like and would drive the archiver with GNU ar
# syntax ("qc <target> <objects>", then ranlib). llvm-lib speaks MSVC lib.exe
# syntax instead, so the archive rule is replaced wholesale. Using llvm-ar to
# keep the GNU syntax would work with lld-link but produces a GNU-format
# archive that MSVC's link.exe -- which is what Rust may invoke -- does not
# reliably accept.
set(CMAKE_C_CREATE_STATIC_LIBRARY "<CMAKE_AR> /nologo /OUT:<TARGET> <OBJECTS>")
set(CMAKE_C_ARCHIVE_CREATE "<CMAKE_AR> /nologo /OUT:<TARGET> <OBJECTS>")
set(CMAKE_C_ARCHIVE_APPEND "")
set(CMAKE_C_ARCHIVE_FINISH "")

# NOTE: do NOT set CMAKE_TRY_COMPILE_TARGET_TYPE to STATIC_LIBRARY here. It
# makes the compiler probe skip the link step, so CMake never discovers the
# implicit CRT libraries, and then adds -nostdlib -nostartfiles to every real
# link -- which fails on the first printf. The probe needs to link a full
# executable, which it can, because the library search paths are set below.

# --- system include and library paths ---------------------------------------

# These paths contain spaces ("Program Files (x86)"), and *_FLAGS_INIT is a
# single space-separated string rather than a list, so each flag has to carry
# its own embedded quotes. Without them clang sees "-isystemC:/Program",
# "Files", "(x86)/..." as separate arguments.
set(PWC_SYSTEM_INCLUDES
    "-isystem\"${PWC_MSVC_ROOT}/include\""
    "-isystem\"${PWC_WINSDK_ROOT}/Include/${PWC_WINSDK_VERSION}/ucrt\""
    "-isystem\"${PWC_WINSDK_ROOT}/Include/${PWC_WINSDK_VERSION}/um\""
    "-isystem\"${PWC_WINSDK_ROOT}/Include/${PWC_WINSDK_VERSION}/shared\"")
string(JOIN " " PWC_SYSTEM_INCLUDES_STR ${PWC_SYSTEM_INCLUDES})

set(CMAKE_C_FLAGS_INIT "${PWC_SYSTEM_INCLUDES_STR}")

set(PWC_LINK_DIRS
    "-L\"${PWC_MSVC_ROOT}/lib/x64\""
    "-L\"${PWC_WINSDK_ROOT}/Lib/${PWC_WINSDK_VERSION}/ucrt/x64\""
    "-L\"${PWC_WINSDK_ROOT}/Lib/${PWC_WINSDK_VERSION}/um/x64\"")
string(JOIN " " PWC_LINK_DIRS_STR ${PWC_LINK_DIRS})

set(CMAKE_EXE_LINKER_FLAGS_INIT "${PWC_LINK_DIRS_STR}")
set(CMAKE_SHARED_LINKER_FLAGS_INIT "${PWC_LINK_DIRS_STR}")

# The UCRT has to be named explicitly. CMake emits
# `-D_DLL -D_MT -Xclang --dependent-lib=msvcrt[d]`, which pulls in the Visual
# C++ runtime, but NOT the Universal CRT that actually provides printf and
# friends -- so a build links cleanly right up until the first stdio call and
# then fails on __acrt_iob_func and __stdio_common_vfprintf. The debug and
# release UCRTs are separate libraries and must match the msvcrt variant CMake
# selected for the configuration, hence the per-config split.
set(CMAKE_EXE_LINKER_FLAGS_DEBUG_INIT "-lucrtd")
set(CMAKE_EXE_LINKER_FLAGS_RELEASE_INIT "-lucrt")
set(CMAKE_EXE_LINKER_FLAGS_RELWITHDEBINFO_INIT "-lucrt")
set(CMAKE_EXE_LINKER_FLAGS_MINSIZEREL_INIT "-lucrt")

# Search host paths for programs, but never for headers or libraries -- those
# must come from the pinned SDK above.
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM BOTH)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
