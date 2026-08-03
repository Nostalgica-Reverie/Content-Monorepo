/* libFuzzer harness for the pw4shell tokeniser.
 *
 * pwc_sh_parse is the first genuine parsing boundary in the tree: it is handed
 * bytes a user typed, and everything downstream trusts its output. §7.4 makes
 * a harness mandatory at exactly this kind of boundary.
 *
 * WHAT COUNTS AS A FAILURE HERE
 *
 * Not "the parser rejected the input" -- rejection is the common correct
 * answer and most of the corpus should hit it. The harness fails when the
 * parser misbehaves: a sanitizer trips (ASan/UBSan are linked into this
 * binary), or it returns PWC_OK while leaving the command in a state the
 * contract forbids. Those postconditions are asserted below, because a
 * successful parse that yields an over-long or unterminated word is precisely
 * the bug that would let a later consumer read out of bounds.
 *
 * Linux only, like every sanitizer leg: clang ships no
 * compiler-rt for the MinGW install and -fsanitize=fuzzer is unavailable for
 * the MSVC target.
 */

#include "packwandc/kernel/pwc_sh_internal.h"

#include <stddef.h>
#include <stdint.h>
#include <string.h>

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    pwc_sh_command cmd;
    /* Deliberately NOT zeroed: pwc_sh_parse promises to initialise the whole
     * struct, and leaving this uninitialised lets MSan catch it if it ever
     * stops doing so. */
    const pwc_status status = pwc_sh_parse(data, size, &cmd);

    if (status != PWC_OK) {
        return 0; /* rejection is a correct outcome, not a finding */
    }

    /* Postconditions of a successful parse. Each of these failing would mean a
     * later consumer can be handed something it is entitled to assume cannot
     * happen. */
    if (cmd.argc > (uint32_t) PWC_SH_MAX_ARGS) {
        __builtin_trap();
    }
    if (cmd.struct_size != (uint32_t) sizeof(pwc_sh_command)) {
        __builtin_trap();
    }
    for (uint32_t i = 0u; i < cmd.argc; ++i) {
        if (cmd.arglen[i] >= (uint32_t) PWC_SH_MAX_ARG) {
            __builtin_trap();
        }
        /* Every word must be NUL-terminated exactly at its stated length: a
         * mismatch means strlen and arglen disagree, and the two consumers
         * that use each would see different arguments. */
        if (cmd.argv[i][cmd.arglen[i]] != 0u) {
            __builtin_trap();
        }
        if (strlen((const char *) cmd.argv[i]) != (size_t) cmd.arglen[i]) {
            __builtin_trap();
        }
    }
    return 0;
}
