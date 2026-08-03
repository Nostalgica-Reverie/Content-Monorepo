/* pw4shell dispatch and built-ins. C never spawns processes. */

#include "packwandc/kernel/pwc_boot_internal.h"
#include "packwandc/kernel/pwc_error.h"
#include "packwandc/kernel/pwc_ipc.h"
#include "packwandc/kernel/pwc_module_registry.h"
#include "packwandc/kernel/pwc_sh_internal.h"
#include "packwandc/uapi/pwc_syscall.h"

#include <stdio.h>
#include <string.h>

enum {
    /* One rendered output line. Bounded like everything else here: no
     * allocation is available. */
    PWSH_LINE_MAX = 512,
};

bool pwsh_word_is(const pwc_sh_command *cmd, size_t index, const char *literal) {
    if (cmd == nullptr || literal == nullptr || index >= (size_t) cmd->argc) {
        return false;
    }
    const size_t length = strlen(literal);
    if ((size_t) cmd->arglen[index] != length) {
        return false;
    }
    return memcmp(cmd->argv[index], literal, length) == 0;
}

void pwsh_emit(const pwsh_sink *sink, const char *text) {
    if (sink == nullptr || !sink->has_port || text == nullptr) {
        return;
    }
    const size_t length = strlen(text);
    /* Discarded on purpose. A full port drops the line; it must not turn the
     * command being reported on into a failure (the trace ring's
     * drop-rather-than-stall rule, applied to output). */
    (void) pwc_ipc_send(sink->port, (const uint8_t *) text, length);
}

/* --- built-ins ---------------------------------------------------------- */

typedef pwc_status (*pwsh_handler)(const pwc_sh_command *cmd, const pwsh_sink *out);

typedef struct pwsh_builtin {
    const char *verb;
    const char *usage;
    const char *summary;
    pwsh_handler run;
} pwsh_builtin;

static pwc_status pwsh_cmd_version(const pwc_sh_command *cmd, const pwsh_sink *out) {
    (void) cmd;
    char line[PWSH_LINE_MAX] = {0};
    (void) snprintf(line,
                    sizeof(line),
                    "packwandc ABI %u.%u",
                    (unsigned) PWC_ABI_VERSION_MAJOR,
                    (unsigned) PWC_ABI_VERSION_MINOR);
    pwsh_emit(out, line);
    return PWC_OK;
}

static pwc_status pwsh_cmd_echo(const pwc_sh_command *cmd, const pwsh_sink *out) {
    char line[PWSH_LINE_MAX] = {0};
    size_t used = 0u;
    for (uint32_t i = 1u; i < cmd->argc; ++i) {
        const size_t word = (size_t) cmd->arglen[i];
        /* +1 for the separating space, +1 for the terminator. */
        if (used + (used == 0u ? 0u : 1u) + word + 1u > sizeof(line)) {
            return PWC_FAIL(PWC_EOVERFLOW, "pwsh", "echo output exceeds the line limit");
        }
        if (used != 0u) {
            line[used] = ' ';
            ++used;
        }
        memcpy(&line[used], cmd->argv[i], word);
        used += word;
    }
    line[used] = '\0';
    pwsh_emit(out, line);
    return PWC_OK;
}

static pwc_status pwsh_cmd_status(const pwc_sh_command *cmd, const pwsh_sink *out) {
    if (cmd->argc != 2u) {
        pwsh_emit(out, "usage: status <code>");
        return PWC_FAIL(PWC_EINVAL, "pwsh", "status takes exactly one argument");
    }
    /* Hand-parsed rather than via atoi, which the banned-construct gate
     * rejects for having no way to report failure. */
    const uint8_t *const text = cmd->argv[1];
    size_t index = 0u;
    int32_t sign = 1;
    if (text[0] == (uint8_t) '-') {
        sign = -1;
        index = 1u;
    }
    if (text[index] == 0u) {
        return PWC_FAIL(PWC_EINVAL, "pwsh", "status: not a number");
    }
    int32_t value = 0;
    for (; index < (size_t) cmd->arglen[1]; ++index) {
        const uint8_t digit = text[index];
        if (digit < (uint8_t) '0' || digit > (uint8_t) '9') {
            return PWC_FAIL(PWC_EINVAL, "pwsh", "status: not a number");
        }
        if (value > 99999) {
            return PWC_FAIL(PWC_EOVERFLOW, "pwsh", "status: number too large");
        }
        value = (value * 10) + (int32_t) (digit - (uint8_t) '0');
    }
    const pwc_status code = (pwc_status) (sign * value);

    char line[PWSH_LINE_MAX] = {0};
    (void) snprintf(
        line, sizeof(line), "%s (%d): %s", pwc_status_name(code), (int) code, pwc_status_describe(code));
    pwsh_emit(out, line);
    return PWC_OK;
}

static pwc_status pwsh_cmd_trace(const pwc_sh_command *cmd, const pwsh_sink *out) {
    if (cmd->argc == 2u && pwsh_word_is(cmd, 1u, "drops")) {
        uint64_t dropped = 0u;
        PWC_TRY(pwc_ktrace_dropped(&dropped));
        char line[PWSH_LINE_MAX] = {0};
        (void) snprintf(line, sizeof(line), "%llu record(s) dropped", (unsigned long long) dropped);
        pwsh_emit(out, line);
        return PWC_OK;
    }
    if (cmd->argc == 2u && pwsh_word_is(cmd, 1u, "drain")) {
        uint32_t count = 0u;
        pwc_trace_record record = {0};
        while (pwc_ktrace_drain(&record) == PWC_OK) {
            char line[PWSH_LINE_MAX] = {0};
            (void) snprintf(line,
                            sizeof(line),
                            "[%llu] %s %s:%u %s",
                            (unsigned long long) record.sequence,
                            record.module,
                            record.file,
                            (unsigned) record.line,
                            record.message);
            pwsh_emit(out, line);
            ++count;
        }
        if (count == 0u) {
            pwsh_emit(out, "trace is empty");
        }
        return PWC_OK;
    }
    pwsh_emit(out, "usage: trace drain | trace drops");
    return PWC_FAIL(PWC_EINVAL, "pwsh", "unknown trace subcommand");
}

/* Zig's Clang 21 frontend rejects an incomplete tentative array definition;
 * keep the forward declaration complete. The trailing sentinel is included. */
static const pwsh_builtin pwsh_builtins[6];

static pwc_status pwsh_cmd_help(const pwc_sh_command *cmd, const pwsh_sink *out) {
    (void) cmd;
    pwsh_emit(out, "pw4shell built-ins:");
    for (size_t i = 0u; pwsh_builtins[i].verb != nullptr; ++i) {
        char line[PWSH_LINE_MAX] = {0};
        (void) snprintf(line, sizeof(line), "  %-24s %s", pwsh_builtins[i].usage, pwsh_builtins[i].summary);
        pwsh_emit(out, line);
    }
    pwsh_emit(out, "Packwand CLI verbs are handled by the host; try 'packwand --help'.");
    return PWC_OK;
}

/* NULL-terminated so pwsh_cmd_help can walk it without a separate count that
 * could fall out of step with the table. */
static const pwsh_builtin pwsh_builtins[] = {
    {"help", "help", "list the built-in commands", pwsh_cmd_help},
    {"version", "version", "report the packwandc ABI version", pwsh_cmd_version},
    {"echo", "echo <words...>", "echo the arguments back", pwsh_cmd_echo},
    {"status", "status <code>", "explain a pwc_status code", pwsh_cmd_status},
    {"trace", "trace drain|drops", "read the kernel trace ring", pwsh_cmd_trace},
    {nullptr, nullptr, nullptr, nullptr},
};

/* --- syscall surface ---------------------------------------------------- */

pwc_status pwc_sh_exec(pwc_handle_t port, const uint8_t *line, size_t length, pwc_sh_command *out) {
    if (out == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "pwsh", "pwc_sh_exec: null command out");
    }
    PWC_TRY(pwc_sh_parse(line, length, out));

    const pwsh_sink sink = {.port = port, .has_port = pwc_handle_is_valid(port)};

    /* A blank or comment-only line is a no-op, not an error: a console that
     * complains when you press enter on an empty prompt is hostile. */
    if (out->argc == 0u) {
        return PWC_OK;
    }

    for (size_t i = 0u; pwsh_builtins[i].verb != nullptr; ++i) {
        if (pwsh_word_is(out, 0u, pwsh_builtins[i].verb)) {
            return pwsh_builtins[i].run(out, &sink);
        }
    }

    /* Parsed and valid, but not ours to run. The caller has `out` and can
     * dispatch it; PWC_ENOSYS distinguishes this from a malformed line, which
     * would have failed in pwc_sh_parse above. */
    return PWC_ENOSYS;
}

/* --- module descriptor -------------------------------------------------- */

static pwc_status pwc_pwsh_init(pwc_module_ctx *ctx) {
    ctx->state = nullptr;
    PWC_NOTE(PWC_TRACE_LEVEL_INFO, "pwsh", "module initialised: pw4shell command language");
    return PWC_OK;
}

static void pwc_pwsh_exit(pwc_module_ctx *ctx) {
    ctx->state = nullptr;
    PWC_NOTE(PWC_TRACE_LEVEL_INFO, "pwsh", "module shut down");
}

/* No declared dependencies.
 *
 * pw4shell writes output through pwipc, but pwipc is kernel infrastructure
 * (kernel/ipc.c), not a registered module -- the port table is initialised by
 * pwc_boot before any module runs. Naming it here would deadlock bring-up:
 * pwc_modules_init resolves `depends` against registered module names only, so
 * a dependency nothing provides never becomes ready and boot fails with
 * PWC_EINVAL after making no progress. */
const pwc_module pwc_module_pwsh = {
    .name = "pwsh",
    .abi_version = PWC_ABI_VERSION_MAJOR,
    .depends = nullptr,
    .init = pwc_pwsh_init,
    .exit = pwc_pwsh_exit,
};
