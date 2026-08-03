/* packwandc ABI fundamentals -- version, linkage, attributes.
 *
 * This is the root uapi header: every other public header includes it, and it
 * includes nothing from include/packwandc/kernel/.
 */
#ifndef PACKWANDC_UAPI_PWC_ABI_H
#define PACKWANDC_UAPI_PWC_ABI_H

#include <stddef.h>
#include <stdint.h>

/* --- version ----------------------------------------------------------- */

/* Bumped only on a breaking change. The host refuses to boot on a mismatch. */
#define PWC_ABI_VERSION_MAJOR 0u
/* Bumped on backward-compatible additions, e.g. appending a syscall.
 * 2: appended pwc_ktrace_drain and pwc_ktrace_dropped (syscalls 7 and 8). */
#define PWC_ABI_VERSION_MINOR 2u

/* --- linkage ----------------------------------------------------------- */

#ifdef __cplusplus
#define PWC_BEGIN_DECLS extern "C" {
#define PWC_END_DECLS   }
#else
#define PWC_BEGIN_DECLS
#define PWC_END_DECLS
#endif

/* packwandc is always linked statically into the host process
 * 3.5: no dlopen, no runtime module loading), so there is no dllexport story
 * and no visibility attribute to apply. PWC_API exists to mark the public
 * surface for readers and for scripts/gate-uapi.sh, not to change linkage. */
#define PWC_API

/* --- attributes -------------------------------------------------------- */

/* C23 spellings, verified on clang 16. These are wrapped
 * rather than used bare so that the one place needing a compiler check, if a
 * pre-C23 toolchain ever has to be supported, is this file.
 *
 * `constexpr` is deliberately absent: it landed in clang 19 and packwandc does
 * not use it. Compile-time constants are enum constants plus static_assert. */
#define PWC_NODISCARD    [[nodiscard]]
#define PWC_MAYBE_UNUSED [[maybe_unused]]
#define PWC_FALLTHROUGH  [[fallthrough]]

/* Marks a struct as part of the wire ABI: no padding surprises, no reordering.
 * Every such struct additionally begins with a uint32_t struct_size and is
 * checked by a static_assert on its total size. */
#if defined(_MSC_VER) && !defined(__clang__)
#define PWC_ABI_PACKED_BEGIN __pragma(pack(push, 8))
#define PWC_ABI_PACKED_END   __pragma(pack(pop))
#else
#define PWC_ABI_PACKED_BEGIN _Pragma("pack(push, 8)")
#define PWC_ABI_PACKED_END   _Pragma("pack(pop)")
#endif

/* --- ABI sanity -------------------------------------------------------- */

/* packwandc assumes a flat 64-bit address space and IEEE-754-free kernel code.
 * These hold on every target, and failing loudly here beats
 * discovering it in a marshalling bug. */
static_assert(sizeof(void *) == 8, "packwandc targets 64-bit platforms only");
static_assert(sizeof(uint32_t) == 4, "uint32_t must be exactly 4 bytes");
static_assert(sizeof(uint64_t) == 8, "uint64_t must be exactly 8 bytes");
static_assert((uint8_t) -1 == 255u, "char must be 8 bits");

#endif /* PACKWANDC_UAPI_PWC_ABI_H */
