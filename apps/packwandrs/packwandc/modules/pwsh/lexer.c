/* pw4shell tokeniser -- the parse boundary.
 *
 * This is the only place in pw4shell that touches untrusted bytes, so it is
 * where the fuzz harness points (tests/fuzz/fuzz_sh_parse.c) and where the
 * rules below are enforced rather than assumed by later stages.
 *
 * GRAMMAR
 *
 *   line     := word* comment?
 *   word     := bare | '"' escaped* '"' | '\'' literal* '\''
 *   comment  := '#' <to end of line>
 *
 * Double quotes honour \" \\ \n \t \r \0-free escapes; single quotes are fully
 * literal, so a Windows path can be written '..\dir' without doubling.
 *
 * WHAT IS REJECTED, AND WHY IT IS REJECTED HERE
 *
 * Every limit is a hard error rather than a truncation. A tokeniser that
 * silently drops the tail of an over-long word hands the dispatcher a command
 * that looks well-formed and means something else -- `mod remove sodium-extra`
 * truncating to `mod remove sodium` is the shape of that bug. Refusing the
 * whole line is the only safe answer.
 *
 * Embedded NULs are rejected for the same reason: the words are handed out
 * NUL-terminated, so a NUL inside one would truncate it for any C consumer
 * while the length field still claimed the original size.
 */

#include "packwandc/kernel/pwc_error.h"
#include "packwandc/kernel/pwc_sh_internal.h"

#include <string.h>

static bool pwsh_is_space(uint8_t c) {
    return c == (uint8_t) ' ' || c == (uint8_t) '\t' || c == (uint8_t) '\r';
}

/* Translate the escape after a backslash inside double quotes.
 * Returns false for an escape that is not recognised, which is an error rather
 * than a pass-through: silently keeping "\q" as "\q" makes a typo look
 * deliberate. */
static bool pwsh_unescape(uint8_t c, uint8_t *out) {
    switch (c) {
        case (uint8_t) 'n':
            *out = (uint8_t) '\n';
            return true;
        case (uint8_t) 't':
            *out = (uint8_t) '\t';
            return true;
        case (uint8_t) 'r':
            *out = (uint8_t) '\r';
            return true;
        case (uint8_t) '"':
        case (uint8_t) '\'':
        case (uint8_t) '\\':
            *out = c;
            return true;
        default:
            return false;
    }
}

/* Append one byte to the word being built. */
static pwc_status pwsh_push(pwc_sh_command *out, size_t index, uint8_t c) {
    const uint32_t length = out->arglen[index];
    /* -1 leaves room for the NUL terminator. */
    if (length + 1u >= (uint32_t) PWC_SH_MAX_ARG) {
        return PWC_FAIL(PWC_EOVERFLOW, "pwsh", "word is longer than the maximum argument length");
    }
    out->argv[index][length] = c;
    out->arglen[index] = length + 1u;
    return PWC_OK;
}

/* Consume one word starting at *cursor, which must not be whitespace. */
static pwc_status
pwsh_scan_word(const uint8_t *line, size_t length, size_t *cursor, pwc_sh_command *out, size_t index) {
    /* 0 = bare, '"' or '\'' = inside that quote. */
    uint8_t quote = 0u;
    bool produced = false;

    while (*cursor < length) {
        const uint8_t c = line[*cursor];

        if (c == 0u) {
            return PWC_FAIL(PWC_EINVAL, "pwsh", "embedded NUL in input");
        }
        /* '\n' ends the word as well as the line. It is deliberately not in
         * pwsh_is_space, because the caller has to *see* it to reject a
         * multi-line command -- but if it were not broken on here, an unquoted
         * newline would be absorbed into the word and "one\ntwo" would parse as
         * the single word "one\ntwo" rather than being refused. */
        if (quote == 0u && (pwsh_is_space(c) || c == (uint8_t) '\n' || c == (uint8_t) '#')) {
            break; /* end of word; '\n' and '#' are handled by the caller */
        }
        if (quote == 0u && (c == (uint8_t) '"' || c == (uint8_t) '\'')) {
            quote = c;
            /* An empty quoted string is a real, empty argument. */
            produced = true;
            ++*cursor;
            continue;
        }
        if (quote != 0u && c == quote) {
            quote = 0u;
            ++*cursor;
            continue;
        }
        if (quote == (uint8_t) '"' && c == (uint8_t) '\\') {
            if (*cursor + 1u >= length) {
                return PWC_FAIL(PWC_EINVAL, "pwsh", "trailing backslash with nothing to escape");
            }
            uint8_t decoded = 0u;
            if (!pwsh_unescape(line[*cursor + 1u], &decoded)) {
                return PWC_FAIL(PWC_EINVAL, "pwsh", "unrecognised escape sequence");
            }
            PWC_TRY(pwsh_push(out, index, decoded));
            produced = true;
            *cursor += 2u;
            continue;
        }

        PWC_TRY(pwsh_push(out, index, c));
        produced = true;
        ++*cursor;
    }

    if (quote != 0u) {
        return PWC_FAIL(PWC_EINVAL, "pwsh", "unterminated quote");
    }
    if (!produced) {
        return PWC_FAIL(PWC_EINVAL, "pwsh", "empty word");
    }
    out->argv[index][out->arglen[index]] = 0u;
    return PWC_OK;
}

pwc_status pwc_sh_parse(const uint8_t *line, size_t length, pwc_sh_command *out) {
    if (out == nullptr || (line == nullptr && length != 0u)) {
        return PWC_FAIL(PWC_EINVAL, "pwsh", "pwc_sh_parse: null line or out");
    }
    if (length > (size_t) PWC_SH_MAX_LINE) {
        return PWC_FAIL(PWC_EOVERFLOW, "pwsh", "line exceeds the maximum length");
    }

    /* Zeroed up front so that every unused slot is a defined empty word rather
     * than whatever the caller's stack held. */
    memset(out, 0, sizeof(*out));
    out->struct_size = (uint32_t) sizeof(pwc_sh_command);

    size_t cursor = 0u;
    while (cursor < length) {
        const uint8_t c = line[cursor];
        if (pwsh_is_space(c)) {
            ++cursor;
            continue;
        }
        if (c == (uint8_t) '\n') {
            /* One command per line: a newline ends this command rather than
             * starting another, so a caller feeding a whole file gets a clear
             * error instead of a silently ignored tail. */
            if (cursor + 1u < length) {
                return PWC_FAIL(PWC_EINVAL, "pwsh", "more than one line in a single command");
            }
            break;
        }
        if (c == (uint8_t) '#') {
            break; /* comment runs to end of line */
        }
        if (out->argc >= (uint32_t) PWC_SH_MAX_ARGS) {
            return PWC_FAIL(PWC_EOVERFLOW, "pwsh", "too many words in one command");
        }
        PWC_TRY(pwsh_scan_word(line, length, &cursor, out, out->argc));
        ++out->argc;
    }

    /* argc == 0 is a blank or comment-only line. That is valid input and not
     * an error -- the dispatcher treats it as a no-op. */
    return PWC_OK;
}
