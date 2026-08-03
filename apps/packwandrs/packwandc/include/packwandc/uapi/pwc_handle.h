/* packwandc handles.
 *
 * Everything a caller can hold is a handle: a watch, a process, a key entry,
 * an IPC port, an input device. A handle is an index into the kernel's table
 * plus a generation counter.
 *
 * The generation counter is the whole point. Closing a slot increments its
 * generation, so a stale handle resolves to PWC_ESTALE rather than to whatever
 * now occupies the slot. In a language with no borrow checker this converts
 * the classic use-after-free and ABA bugs from memory corruption into a
 * returned error code -- the single highest-value structural defence available
 * to a C layer of this shape.
 *
 * PHASE 0: the types, the encoding, and the invariants are defined here and
 * are part of the frozen ABI. The table itself (kernel/handle.c) is phase 1.
 */
#ifndef PACKWANDC_UAPI_PWC_HANDLE_H
#define PACKWANDC_UAPI_PWC_HANDLE_H

#include "packwandc/uapi/pwc_abi.h"

PWC_BEGIN_DECLS

/* Width of each field in the packed 64-bit wire encoding. Named because the
 * mask and shift constants below must track them -- exactly the case where
 * readability-magic-numbers is right and the suppression does not apply. */
enum {
    PWC_HANDLE_INDEX_BITS = 32,
    PWC_HANDLE_GENERATION_BITS = 32,
};

static_assert(PWC_HANDLE_INDEX_BITS + PWC_HANDLE_GENERATION_BITS == 64,
              "packed handle encoding must fill exactly 64 bits");

typedef struct pwc_handle {
    uint32_t index;
    uint32_t generation;
} pwc_handle_t;

static_assert(sizeof(pwc_handle_t) == 8, "pwc_handle_t is part of the wire ABI");

/* The reserved never-valid handle. Index 0 is never handed out, so a
 * zero-initialised struct is invalid by construction -- callers that forget to
 * initialise get PWC_EBADF rather than slot 0. */
#define PWC_HANDLE_INVALID ((pwc_handle_t){.index = 0u, .generation = 0u})

/* --- rights ------------------------------------------------------------
 *
 * Rights only ever narrow. pwc_handle_dup can drop bits, never add them, so a
 * capability handed to a less-trusted consumer cannot be re-widened. pwfs uses
 * this to give the IDE a handle rooted at a pack directory.
 */
enum {
    PWC_RIGHT_NONE = 0u,
    PWC_RIGHT_READ = 1u << 0u,
    PWC_RIGHT_WRITE = 1u << 1u,
    PWC_RIGHT_WAIT = 1u << 2u,
    PWC_RIGHT_TRANSFER = 1u << 3u, /* may be sent across an IPC port */
    PWC_RIGHT_DUP = 1u << 4u,      /* may be duplicated at all */
    PWC_RIGHT_CLOSE = 1u << 5u,
    PWC_RIGHT_ALL = 0x3fu,
};

static_assert(PWC_RIGHT_ALL == (PWC_RIGHT_READ | PWC_RIGHT_WRITE | PWC_RIGHT_WAIT | PWC_RIGHT_TRANSFER |
                                PWC_RIGHT_DUP | PWC_RIGHT_CLOSE),
              "PWC_RIGHT_ALL must be the union of every defined right");

/* --- encoding ----------------------------------------------------------
 *
 * Handles cross the FFI boundary as a single uint64_t so that the Rust side
 * can hold them in one register-width value with no struct layout dependency.
 */

PWC_NODISCARD static inline uint64_t pwc_handle_pack(pwc_handle_t h) {
    return ((uint64_t) h.generation << (uint64_t) PWC_HANDLE_INDEX_BITS) | (uint64_t) h.index;
}

PWC_NODISCARD static inline pwc_handle_t pwc_handle_unpack(uint64_t packed) {
    return (pwc_handle_t){
        .index = (uint32_t) (packed & 0xffffffffu),
        .generation = (uint32_t) (packed >> (uint64_t) PWC_HANDLE_INDEX_BITS),
    };
}

PWC_NODISCARD static inline bool pwc_handle_is_valid(pwc_handle_t h) { return h.index != 0u; }

PWC_NODISCARD static inline bool pwc_handle_eq(pwc_handle_t a, pwc_handle_t b) {
    return a.index == b.index && a.generation == b.generation;
}

PWC_END_DECLS

#endif /* PACKWANDC_UAPI_PWC_HANDLE_H */
