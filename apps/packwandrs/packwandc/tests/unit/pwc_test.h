/* Minimal in-repo test harness -- see packwandc.md 7.4.
 *
 * No external framework, following the precedent set by the removed
 * tools/hashutil, which carried its vectors and its runner in the binary
 * itself and exposed them as `--selftest`. A dependency-free C layer should
 * not acquire its first dependency in its test tree.
 *
 * Usage:
 *
 *     static void test_thing(void) {
 *         PWC_CHECK_EQ_I(pwc_sys_version(&a, &b), PWC_OK);
 *     }
 *
 *     int main(void) {
 *         PWC_RUN(test_thing);
 *         return pwc_test_report("core");
 *     }
 *
 * Failures print `FAIL file:line: <detail>` to stderr and increment a counter;
 * the process exits non-zero if any check failed. Checks do not abort, so one
 * run reports every failure rather than only the first.
 */
#ifndef PACKWANDC_TESTS_PWC_TEST_H
#define PACKWANDC_TESTS_PWC_TEST_H

#include <stdint.h>
#include <stdio.h>
#include <string.h>

static unsigned pwc_test_failures = 0u;
static unsigned pwc_test_checks = 0u;

#define PWC_TEST_FAIL(...)                                                                                   \
    do {                                                                                                     \
        (void) fprintf(stderr, "FAIL %s:%d: ", __FILE__, __LINE__);                                          \
        (void) fprintf(stderr, __VA_ARGS__);                                                                 \
        (void) fputc('\n', stderr);                                                                          \
        pwc_test_failures++;                                                                                 \
    } while (0)

#define PWC_CHECK(cond)                                                                                      \
    do {                                                                                                     \
        pwc_test_checks++;                                                                                   \
        if (!(cond)) {                                                                                       \
            PWC_TEST_FAIL("expected %s", #cond);                                                             \
        }                                                                                                    \
    } while (0)

/* Integer comparison. Both sides are widened to int64_t so the macro works for
 * any signed integral type without a format-string zoo. */
#define PWC_CHECK_EQ_I(got, want)                                                                            \
    do {                                                                                                     \
        pwc_test_checks++;                                                                                   \
        const int64_t pwc__got = (int64_t) (got);                                                            \
        const int64_t pwc__want = (int64_t) (want);                                                          \
        if (pwc__got != pwc__want) {                                                                         \
            PWC_TEST_FAIL("%s: got %lld, want %lld", #got, (long long) pwc__got, (long long) pwc__want);     \
        }                                                                                                    \
    } while (0)

#define PWC_CHECK_EQ_U(got, want)                                                                            \
    do {                                                                                                     \
        pwc_test_checks++;                                                                                   \
        const uint64_t pwc__got = (uint64_t) (got);                                                          \
        const uint64_t pwc__want = (uint64_t) (want);                                                        \
        if (pwc__got != pwc__want) {                                                                         \
            PWC_TEST_FAIL("%s: got %llu, want %llu",                                                         \
                          #got,                                                                              \
                          (unsigned long long) pwc__got,                                                     \
                          (unsigned long long) pwc__want);                                                   \
        }                                                                                                    \
    } while (0)

/* String comparison. A NULL on either side is a failure, never a crash --
 * several packwandc functions promise never to return NULL and this is how
 * that promise gets tested. */
#define PWC_CHECK_EQ_STR(got, want)                                                                          \
    do {                                                                                                     \
        pwc_test_checks++;                                                                                   \
        const char *pwc__got = (got);                                                                        \
        const char *pwc__want = (want);                                                                      \
        if (pwc__got == nullptr || pwc__want == nullptr) {                                                   \
            PWC_TEST_FAIL("%s: unexpected NULL", #got);                                                      \
        } else if (strcmp(pwc__got, pwc__want) != 0) {                                                       \
            PWC_TEST_FAIL("%s: got \"%s\", want \"%s\"", #got, pwc__got, pwc__want);                         \
        }                                                                                                    \
    } while (0)

#define PWC_RUN(fn)                                                                                          \
    do {                                                                                                     \
        (void) fprintf(stderr, "  run %s\n", #fn);                                                           \
        (fn)();                                                                                              \
    } while (0)

static inline int pwc_test_report(const char *suite) {
    if (pwc_test_failures == 0u) {
        (void) fprintf(stderr, "ok %s: %u checks passed\n", suite, pwc_test_checks);
        return 0;
    }
    (void) fprintf(stderr, "FAILED %s: %u of %u checks failed\n", suite, pwc_test_failures, pwc_test_checks);
    return 1;
}

#endif /* PACKWANDC_TESTS_PWC_TEST_H */
