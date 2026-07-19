/* hashutil - batch multi-algorithm file hasher.
 *
 * Reads newline-delimited file paths from stdin, hashes each with
 * sha256, sha512, md5, and the CurseForge murmur2 variant in a single
 * read pass, and writes one NDJSON line per file to stdout:
 *
 *   {"path":"mods/foo.jar","sha256":"...","sha512":"...","md5":"...","murmur2":"...","error":null}
 *
 * sha256/sha512/md5 are lowercase hex; murmur2 is a decimal unsigned
 * 32-bit integer -- matching apps/packwand/core/hash.go's HashToString
 * output exactly, so results can be compared byte-for-byte with the Go
 * implementation without conversion.
 *
 * A per-file read failure is reported via the "error" field, not a
 * process abort, so one bad path doesn't lose an entire batch. Exit
 * code is 0 unless a fatal (non-per-file) condition occurs: bad flags.
 *
 * This tool is a pure mechanism: it has no knowledge of pack structure
 * or exclusion rules, which stay in the Go caller. See c.md section 1
 * for the design rationale.
 *
 * Usage:
 *   hashutil [--algos=sha256,sha512,md5,murmur2] < paths.txt
 *   hashutil --selftest
 *
 * Targets C23. Built with `-std=c2x` (not the later `-std=c23` spelling):
 * as of writing, neither of this repo's two verified toolchains (gcc
 * 13.2.0, clang 16.0.6) accept `-std=c23` -- gcc added that exact flag
 * name in gcc 14, clang in clang 17. `-std=c2x` requests the same
 * standard (it was C23's working-draft flag name pre-ratification) and
 * is accepted by both; this file uses no C23-only library additions, so
 * the flag-name difference has no effect on the code itself.
 */

/* Feature-test macros MUST be defined before any header is included: on
 * glibc, _FILE_OFFSET_BITS and _POSIX_C_SOURCE only take effect if set
 * before the first system header pulls in <features.h> -- setting them
 * later (even earlier in this same file, after another #include) is a
 * silent no-op, not an error. _POSIX_C_SOURCE 200809L (POSIX.1-2008) is
 * required to expose fseeko/ftello/off_t's declarations at all under a
 * strict -std=c2x build: __STRICT_ANSI__ (which -std=c2x implies) hides
 * glibc's POSIX extensions unless a feature-test macro explicitly asks
 * for them, and without it fseeko/ftello are only implicitly declared
 * (undefined behavior, and a hard error under -Werror). This is not
 * needed on Windows, which reaches its 64-bit seek functions through
 * MSVCRT's own `<io.h>`, not glibc's POSIX layer. */
#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200809L
#define _FILE_OFFSET_BITS 64
#endif

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "md5.h"
#include "murmur2.h"
#include "sha256.h"
#include "sha512.h"

#if defined(_WIN32)
#include <io.h>
#define HASHUTIL_FSEEK _fseeki64
#define HASHUTIL_FTELL _ftelli64
typedef int64_t hashutil_offset_t;
#else
#include <sys/types.h> /* off_t */
#define HASHUTIL_FSEEK fseeko
#define HASHUTIL_FTELL ftello
typedef off_t hashutil_offset_t;
#endif

#define HASHUTIL_ALGO_SHA256 (1u << 0)
#define HASHUTIL_ALGO_SHA512 (1u << 1)
#define HASHUTIL_ALGO_MD5 (1u << 2)
#define HASHUTIL_ALGO_MURMUR2 (1u << 3)
#define HASHUTIL_ALGO_ALL                                                                                    \
    (HASHUTIL_ALGO_SHA256 | HASHUTIL_ALGO_SHA512 | HASHUTIL_ALGO_MD5 | HASHUTIL_ALGO_MURMUR2)

/* Digest sizes and their derived hex-string buffer sizes, named so the
 * two never drift independently -- a clang-tidy readability-magic-numbers
 * pass on an earlier version of this file flagged these as unexplained
 * literals; naming them ties each buffer size directly to the digest
 * size it must match, rather than leaving two numbers that happen to
 * agree today. */
#define SHA256_DIGEST_BYTES 32
#define SHA512_DIGEST_BYTES 64
#define MD5_DIGEST_BYTES 16
#define SHA256_HEX_LEN (SHA256_DIGEST_BYTES * 2 + 1) /* + NUL */
#define SHA512_HEX_LEN (SHA512_DIGEST_BYTES * 2 + 1)
#define MD5_HEX_LEN (MD5_DIGEST_BYTES * 2 + 1)
#define MURMUR2_DEC_LEN 16                  /* max uint32 decimal digits (10) + NUL, rounded up */
#define QUOTED_LEN(hex_len) ((hex_len) + 2) /* + surrounding \"...\" */

static void hex_encode(const uint8_t *bytes, size_t len, char *out) {
    static const char digits[] = "0123456789abcdef";
    for (size_t i = 0; i < len; ++i) {
        out[i * 2] = digits[bytes[i] >> 4];
        out[i * 2 + 1] = digits[bytes[i] & 0xF];
    }
    out[len * 2] = '\0';
}

/* Escapes a file path for embedding in a JSON string literal. Returns a
 * freshly malloc'd, null-terminated string (caller frees) or NULL on
 * allocation failure. Sized for the worst case (every input byte expands
 * to a 6-char \u00XX escape) rather than a fixed cap, so long paths can't
 * be silently truncated. */
static char *json_escape_path(const char *path) {
    static const char hexd[] = "0123456789abcdef";
    size_t len = strlen(path);
    char *out = (char *)malloc(len * 6 + 1);
    if (!out) {
        return NULL;
    }
    size_t o = 0;
    for (const char *p = path; *p != '\0'; ++p) {
        unsigned char c = (unsigned char)*p;
        if (c == '"' || c == '\\') {
            out[o++] = '\\';
            out[o++] = (char)c;
        } else if (c == '\n') {
            out[o++] = '\\';
            out[o++] = 'n';
        } else if (c == '\r') {
            out[o++] = '\\';
            out[o++] = 'r';
        } else if (c < 0x20) {
            out[o++] = '\\';
            out[o++] = 'u';
            out[o++] = '0';
            out[o++] = '0';
            out[o++] = hexd[c >> 4];
            out[o++] = hexd[c & 0xF];
        } else {
            out[o++] = (char)c;
        }
    }
    out[o] = '\0';
    return out;
}

typedef struct {
    uint8_t *data;
    size_t len;
} filebuf_t;

/* Reads the whole file at `path` into a freshly malloc'd buffer. Returns
 * 0 on success, -1 on failure with a short reason in *errmsg. */
static int read_whole_file(const char *path, filebuf_t *out, const char **errmsg) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        *errmsg = "open failed";
        return -1;
    }
    if (HASHUTIL_FSEEK(f, 0, SEEK_END) != 0) {
        (void)fclose(f); /* read-only fd; nothing buffered to lose on close failure */
        *errmsg = "seek failed";
        return -1;
    }
    hashutil_offset_t size = HASHUTIL_FTELL(f);
    if (size < 0) {
        (void)fclose(f);
        *errmsg = "tell failed";
        return -1;
    }
    if (HASHUTIL_FSEEK(f, 0, SEEK_SET) != 0) {
        (void)fclose(f);
        *errmsg = "seek failed";
        return -1;
    }

    uint8_t *buf = (uint8_t *)malloc(size > 0 ? (size_t)size : 1);
    if (!buf) {
        (void)fclose(f);
        *errmsg = "out of memory";
        return -1;
    }

    size_t read_total = 0;
    while (read_total < (size_t)size) {
        size_t n = fread(buf + read_total, 1, (size_t)size - read_total, f);
        if (n == 0) {
            break;
        }
        read_total += n;
    }
    (void)fclose(f);

    if (read_total != (size_t)size) {
        free(buf);
        *errmsg = "short read";
        return -1;
    }

    out->data = buf;
    out->len = read_total;
    return 0;
}

/* `enabled` (not `algos`+`bit`, checked by the caller) so this doesn't
 * take two adjacent same-ish-typed integer parameters that could be
 * swapped by mistake at a call site -- flagged by
 * bugprone-easily-swappable-parameters on an earlier version of this
 * function. */
static void field(char *out, size_t out_cap, bool enabled, const char *hex_or_dec) {
    int n = enabled ? snprintf(out, out_cap, "\"%s\"", hex_or_dec) : snprintf(out, out_cap, "null");
    /* n < 0 is a genuine encoding error; n >= (int)out_cap means
     * truncation. Both indicate a buffer-sizing bug in this file (every
     * caller's out_cap is derived from the exact digest size), not a
     * runtime condition to recover from -- abort loudly rather than
     * silently emit truncated/malformed JSON. */
    if (n < 0 || (size_t)n >= out_cap) {
        /* %llu, not %zu: MinGW-w64's legacy-mode format checker doesn't
         * recognize %zu without extra flags, even though the runtime
         * supports it -- %llu with an explicit cast is portable across
         * every toolchain this file targets without special-casing. */
        (void)fprintf(stderr, "hashutil: internal error: field buffer too small (n=%d, cap=%llu)\n", n,
                      (unsigned long long)out_cap);
        abort();
    }
}

static void hash_one(const char *path, unsigned algos) {
    char *path_escaped = json_escape_path(path);
    if (!path_escaped) {
        /* Pathological OOM sizing a small escape buffer: nothing safe to
         * print (the raw path could itself break the JSON), so skip this
         * line rather than risk corrupting the whole NDJSON stream. */
        (void)fprintf(stderr, "hashutil: out of memory escaping path, skipping\n");
        return;
    }

    filebuf_t fb;
    const char *errmsg = NULL;
    if (read_whole_file(path, &fb, &errmsg) != 0) {
        printf("{\"path\":\"%s\",\"sha256\":null,\"sha512\":null,\"md5\":null,\"murmur2\":null,"
               "\"error\":\"%s\"}\n",
               path_escaped, errmsg);
        free(path_escaped);
        return;
    }

    char sha256_hex[SHA256_HEX_LEN] = {0};
    char sha512_hex[SHA512_HEX_LEN] = {0};
    char md5_hex[MD5_HEX_LEN] = {0};
    char murmur2_dec[MURMUR2_DEC_LEN] = {0};

    if (algos & HASHUTIL_ALGO_SHA256) {
        sha256_ctx ctx;
        uint8_t digest[SHA256_DIGEST_BYTES];
        sha256_init(&ctx);
        sha256_update(&ctx, fb.data, fb.len);
        sha256_final(&ctx, digest);
        hex_encode(digest, sizeof(digest), sha256_hex);
    }
    if (algos & HASHUTIL_ALGO_SHA512) {
        sha512_ctx ctx;
        uint8_t digest[SHA512_DIGEST_BYTES];
        sha512_init(&ctx);
        sha512_update(&ctx, fb.data, fb.len);
        sha512_final(&ctx, digest);
        hex_encode(digest, sizeof(digest), sha512_hex);
    }
    if (algos & HASHUTIL_ALGO_MD5) {
        md5_ctx ctx;
        uint8_t digest[MD5_DIGEST_BYTES];
        md5_init(&ctx);
        md5_update(&ctx, fb.data, fb.len);
        md5_final(&ctx, digest);
        hex_encode(digest, sizeof(digest), md5_hex);
    }
    if (algos & HASHUTIL_ALGO_MURMUR2) {
        /* Strips in place (murmur2cf_strip allows out == data): this block
         * must stay the LAST consumer of fb.data, since it compacts the
         * buffer. Saves a second whole-file allocation. */
        size_t stripped_len = murmur2cf_strip(fb.data, fb.len, fb.data);
        uint32_t h = murmurhash2(fb.data, stripped_len, 1);
        (void)snprintf(murmur2_dec, sizeof(murmur2_dec), "%u", h);
    }

    free(fb.data);

    char f_sha256[QUOTED_LEN(SHA256_HEX_LEN)];
    char f_sha512[QUOTED_LEN(SHA512_HEX_LEN)];
    char f_md5[QUOTED_LEN(MD5_HEX_LEN)];
    char f_murmur2[QUOTED_LEN(MURMUR2_DEC_LEN)];
    field(f_sha256, sizeof(f_sha256), (algos & HASHUTIL_ALGO_SHA256) != 0, sha256_hex);
    field(f_sha512, sizeof(f_sha512), (algos & HASHUTIL_ALGO_SHA512) != 0, sha512_hex);
    field(f_md5, sizeof(f_md5), (algos & HASHUTIL_ALGO_MD5) != 0, md5_hex);
    field(f_murmur2, sizeof(f_murmur2), (algos & HASHUTIL_ALGO_MURMUR2) != 0, murmur2_dec);

    printf("{\"path\":\"%s\",\"sha256\":%s,\"sha512\":%s,\"md5\":%s,\"murmur2\":%s,\"error\":null}\n",
           path_escaped, f_sha256, f_sha512, f_md5, f_murmur2);
    free(path_escaped);
}

static unsigned parse_algos(const char *spec) {
    unsigned algos = 0;
    char buf[128];
    size_t len = strlen(spec);
    if (len >= sizeof(buf)) {
        len = sizeof(buf) - 1;
    }
    memcpy(buf, spec, len);
    buf[len] = '\0';

    char *tok = strtok(buf, ",");
    while (tok) {
        if (strcmp(tok, "sha256") == 0) {
            algos |= HASHUTIL_ALGO_SHA256;
        } else if (strcmp(tok, "sha512") == 0) {
            algos |= HASHUTIL_ALGO_SHA512;
        } else if (strcmp(tok, "md5") == 0) {
            algos |= HASHUTIL_ALGO_MD5;
        } else if (strcmp(tok, "murmur2") == 0) {
            algos |= HASHUTIL_ALGO_MURMUR2;
        } else {
            (void)fprintf(stderr, "hashutil: unknown algorithm '%s'\n", tok);
            exit(2);
        }
        tok = strtok(NULL, ",");
    }
    return algos;
}

/* Hashes `data`/`len` with all three algorithms and compares against the
 * expected hex strings, incrementing *failures and printing a diagnostic
 * on mismatch. Shared by both vector tables in selftest() below -- pulled
 * out specifically because inlining it in both loops pushed selftest()
 * over clang-tidy's cognitive-complexity threshold; this is a real
 * simplification (one hash-and-compare implementation instead of two
 * near-identical copies), not just a way to dodge the check. */
static void check_hashes(const uint8_t *data, size_t len, const char *label, const char *expect_sha256,
                         const char *expect_sha512, const char *expect_md5, int *failures) {
    sha256_ctx sctx;
    uint8_t sdigest[SHA256_DIGEST_BYTES];
    char shex[SHA256_HEX_LEN];
    sha256_init(&sctx);
    sha256_update(&sctx, data, len);
    sha256_final(&sctx, sdigest);
    hex_encode(sdigest, sizeof(sdigest), shex);
    if (strcmp(shex, expect_sha256) != 0) {
        (void)fprintf(stderr, "FAIL sha256(%s): got %s want %s\n", label, shex, expect_sha256);
        (*failures)++;
    }

    sha512_ctx s5ctx;
    uint8_t s5digest[SHA512_DIGEST_BYTES];
    char s5hex[SHA512_HEX_LEN];
    sha512_init(&s5ctx);
    sha512_update(&s5ctx, data, len);
    sha512_final(&s5ctx, s5digest);
    hex_encode(s5digest, sizeof(s5digest), s5hex);
    if (strcmp(s5hex, expect_sha512) != 0) {
        (void)fprintf(stderr, "FAIL sha512(%s): got %s want %s\n", label, s5hex, expect_sha512);
        (*failures)++;
    }

    md5_ctx mctx;
    uint8_t mdigest[MD5_DIGEST_BYTES];
    char mhex[MD5_HEX_LEN];
    md5_init(&mctx);
    md5_update(&mctx, data, len);
    md5_final(&mctx, mdigest);
    hex_encode(mdigest, sizeof(mdigest), mhex);
    if (strcmp(mhex, expect_md5) != 0) {
        (void)fprintf(stderr, "FAIL md5(%s): got %s want %s\n", label, mhex, expect_md5);
        (*failures)++;
    }
}

/* Known-vector self-check, run in CI under ASan+UBSan. Returns 0 on
 * success. sha256/sha512/md5 vectors are the standard "" and "abc"
 * FIPS/RFC test strings; murmur2 vectors are from go-murmur's own test
 * file (murmur2_test.go), so the raw (non-CF) algorithm is checked
 * against the exact library apps/packwand depends on. */
static int selftest(void) {
    int failures = 0;

    struct {
        const char *input;
        const char *expect_sha256;
        const char *expect_sha512;
        const char *expect_md5;
    } vecs[] = {
        {"", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
         "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b"
         "0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
         "d41d8cd98f00b204e9800998ecf8427e"},
        {"abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
         "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a"
         "836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
         "900150983cd24fb0d6963f7d28e17f72"},
    };

    for (size_t i = 0; i < sizeof(vecs) / sizeof(vecs[0]); ++i) {
        check_hashes((const uint8_t *)vecs[i].input, strlen(vecs[i].input), vecs[i].input,
                     vecs[i].expect_sha256, vecs[i].expect_sha512, vecs[i].expect_md5, &failures);
    }

    /* {"foo",123,...}..{"blam",777,...} are from go-murmur's own test file
     * (murmur2_test.go); {"",42,...}..{"abcdefgh",42,...} were generated by
     * calling the real go-murmur MurmurHash2 (not reimplemented or
     * hand-derived) at lengths 0..8, deliberately covering every tail-byte
     * remainder (len%4 in {0,1,2,3}) the algorithm's switch statement
     * branches on, plus one exact-block (len=4) and one multi-block
     * (len=8) case. */
    struct {
        const char *input;
        uint32_t seed;
        uint32_t expect;
    } mvecs[] = {
        {"foo", 123, 1412061192u},     {"zztop", 123, 1878194508u}, {"foobarbaz", 234, 1777016281u},
        {"blam", 777, 1668928339u},    {"", 42, 275804818u},        {"a", 42, 1148686264u},
        {"ab", 42, 3855859532u},       {"abc", 42, 3658290176u},    {"abcd", 42, 2000215727u},
        {"abcde", 42, 2887277479u},    {"abcdef", 42, 2614891300u}, {"abcdefg", 42, 3594543523u},
        {"abcdefgh", 42, 2219068749u},
    };
    for (size_t i = 0; i < sizeof(mvecs) / sizeof(mvecs[0]); ++i) {
        uint32_t got = murmurhash2((const uint8_t *)mvecs[i].input, strlen(mvecs[i].input), mvecs[i].seed);
        if (got != mvecs[i].expect) {
            (void)fprintf(stderr, "FAIL murmurhash2(%s, %u): got %u want %u\n", mvecs[i].input, mvecs[i].seed,
                          got, mvecs[i].expect);
            failures++;
        }
    }

    /* Block-boundary vectors: sha256/md5 use 64-byte blocks, sha512 uses
     * 128-byte blocks, and each pads differently depending on whether the
     * final partial block has room for the 0x80 byte + length field or
     * needs an extra block -- lengths straddling those boundaries are
     * exactly where an off-by-one in padding logic would surface and the
     * fixed "" / "abc" vectors above can't catch. Input is a deterministic
     * ramp (byte i = i & 0xFF); expected digests were computed
     * independently in Python (hashlib) against that exact same
     * generator, not derived from this file's own implementation. */
    {
        static const struct {
            size_t len;
            const char *sha256;
            const char *sha512;
            const char *md5;
        } bvecs[] = {
            {0, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
             "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2"
             "f63"
             "b931bd47417a81a538327af927da3e",
             "d41d8cd98f00b204e9800998ecf8427e"},
            {1, "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d",
             "b8244d028981d693af7b456af8efa4cad63d282e19ff14942c246e50d9351d22704a802a71c3580b6370de4ceb293c3"
             "24a"
             "8423342557d4e5c38438f0e36910ee",
             "93b885adfe0da089cdf634904fd59f71"},
            {54, "675f28acc0b90a72d1c3a570fe83ac565555db358cf01826dc8eefb2bf7ca0f3",
             "6d7644db575c5c238da02cc4259996cf163a3a3b5eccc4fc62442ddf01aa05ef0c4edbe3e6d220df189c984aa55726a"
             "492"
             "2efe004832f2d8887f0b8a9267db40",
             "33dd62a2df6538daf1cf821d9cde61f9"},
            {55, "463eb28e72f82e0a96c0a4cc53690c571281131f672aa229e0d45ae59b598b59",
             "6856647f269c2ee3d8128f0b25427659d880641ef343300dd3cd4679168f58d6527fda70b4ebc854e2065e172b7d58c"
             "153"
             "6992c0810599259ba84a2b40c65414",
             "6912ee65fff2d9f9ce2508cddf8bcda0"},
            {56, "da2ae4d6b36748f2a318f23e7ab1dfdf45acdc9d049bd80e59de82a60895f562",
             "8b12b2f6fe400a51d29656e2b8c42a1bbfe6fcf3e425da430db05d1a2dda14790dee20fa8b22d8762afffe4988a5c98"
             "a44"
             "30d22a17e41e23d90fa61ab75671a9",
             "51fdd1acda72405dfdfa03fcb85896d7"},
            {57, "2fe741af801cc238602ac0ec6a7b0c3a8a87c7fc7d7f02a3fe03d1c12eac4d8f",
             "92cb9f2e4eee07c7b32b06cf4917fbe54365f55247cc9b5bc4478d9fada52b07d1c302b3959d0ca9a75a629653ea7c2"
             "45a"
             "8fbba2a265cda4ea70ac5a860a6f3d",
             "5320ef4c17ef34a0cf2db763338d25eb"},
            {63, "29af2686fd53374a36b0846694cc342177e428d1647515f078784d69cdb9e488",
             "9dc9c5598e55dc42955695320839788e353f1d7f6ba74df74c80a8a52f463c0697f57f68835d1418f4ce9b6530cd79b"
             "d0f"
             "4c6f7e13c93feb1218c0b65c2c0561",
             "48a6295221902e8e0938f773a7185e72"},
            {64, "fdeab9acf3710362bd2658cdc9a29e8f9c757fcf9811603a8c447cd1d9151108",
             "ee4320ebaf3fdb4f2c832b137200c08e235e0fa7bbd0eb1740c7063ba8a0d151da77e003398e1714a955d475b05e3e9"
             "50b"
             "639503b452ec185de4229bc4873949",
             "b2d3f56bc197fd985d5965079b5e7148"},
            {65, "4bfd2c8b6f1eec7a2afeb48b934ee4b2694182027e6d0fc075074f2fabb31781",
             "02856cef735f9acec6b9e33f0fbc8f9804d2aa54187f382b8ae842e5d3696c07459aad2a5aed25ea5e117eb1c7ba35d"
             "a6a"
             "7a8adce9e6afe3ad79e9fa42d5bba8",
             "8bd7053801c768420faf816fadba971c"},
            {111, "60780e9451bdc43cf4530ffc95cbb0c4eb24dae2c39f55f334d679e076c08065",
             "a1a111449b198d9b1f538bad7f3fc1022b3a5b1a5e90a0bc860de8512746cbc31599e6c834de3a3235327af0b51ff57"
             "bf7"
             "acf1974a73014d9c3953812edc7c8d",
             "4fad3ab7d8546851ec1bb63ea7e6f5a8"},
            {112, "09373f127d34e61dbbaa8bc4499c87074f2ddb10e1b465f506d7d70a15011979",
             "c5fbd731d19d2ae1180f001be72c2c1aaba1d7b094b3748880e24593b8e117a750e11c1bd867cc2f96dace8c8b74abd"
             "2d5"
             "c4f236be444e77d30d1916174070b9",
             "d1fec2ac3715e791ca5f489f300381b3"},
            {113, "13aaa9b5fb739cdb0e2af99d9ac0a409390adc4d1cb9b41f1ef94f8552060e92",
             "61b2e77db697dfe5571fff3ed06bd60c41e1e7b7c08a80de01cb16526d9a9a52d690dfbe792278a60f6e2b4c57a97c7"
             "297"
             "73f26e258d2393890c985d645f6715",
             "f62807c995735b44699bb8179100ce87"},
            {127, "92ca0fa6651ee2f97b884b7246a562fa71250fedefe5ebf270d31c546bfea976",
             "eab89674feaa34e27aebeeff3c0a4d70070bb872d5e9f186cf1dbbdee517b6e35724d629ff025a5b07185e911ada7e3"
             "c8a"
             "cf830aa0e4f71777bd2d44f504f7f0",
             "8402b21e7bc7906493bae0dac017f1f9"},
            {128, "471fb943aa23c511f6f72f8d1652d9c880cfa392ad80503120547703e56a2be5",
             "1dffd5e3adb71d45d2245939665521ae001a317a03720a45732ba1900ca3b8351fc5c9b4ca513eba6f80bc7b1d1fdad"
             "4ab"
             "d13491cb824d61b08d8c0e1561b3f7",
             "37eff01866ba3f538421b30b7cbefcac"},
            {129, "5099c6a56203f9687f7d33f4bfdf576d31dc91f6b695ecea38b2770c87631135",
             "1d9da57fbbdab09afb3506ab2d223d06109d65c1c8ad197f50138f714bc4c3f2fe5787922639c680acad1c651f95599"
             "0425"
             "954ce2cba0c5cc83f2667d878eb0f",
             "46f986692847558fc38b0cece591c20f"},
            {255, "3f8591112c6bbe5c963965954e293108b7208ed2af893e500d859368c654eabe",
             "15025c9d135861ff5a549df0bfd6c398fd126613496d4e97627651e68b7b1f80407f187d7978464f0f78bfeea787600"
             "faa"
             "ebbe991eddb60671cd0ce874f0a744",
             "11b7aaa64c413d2f0fccf893881c46a2"},
            {256, "40aff2e9d2d8922e47afd4648e6967497158785fbd1da870e7110266bf944880",
             "1e7b80bc8edc552c8feeb2780e111477e5bc70465fac1a77b29b35980c3f0ce4a036a6c9462036824bd56801e62af7e"
             "9fe"
             "ba5c22ed8a5af877bf7de117dcac6d",
             "e2c865db4162bed963bfaa9ef6ac18f0"},
            {1000, "a8af099bf2e878609558dbf69d8f88f4a31040a8cf84b549a0cfa912f12ffc3f",
             "6cd2eda9bf9c0597129029b0054b81e433f6b8b7b499a75eb705efd74bac194149835b1d1a14c48be696e4d588456d5"
             "12a"
             "22eae7aa1b57be2b56eae7d35e08cb",
             "cbecbdb0fdd5cec1e242493b6008cc79"},
            {10000, "3421d9aa928a94decb191ab8e8b76c1d8434bf602c5b3ba10ad42f54c8199c34",
             "04fb3e81ecd46e1af625bb1f156475a3384a7e4384abfa43f846bc3412f9041d89a275635d79c6ab88d2e0fcaa13898"
             "5ff"
             "d4721e42b3c62f05b1ebe060349a99",
             "dc50add066871756c3f0260f0aa76cd2"},
        };

        uint8_t *buf = (uint8_t *)malloc(10000);
        if (!buf) {
            (void)fprintf(stderr, "FAIL boundary vectors: out of memory\n");
            failures++;
        } else {
            for (size_t v = 0; v < sizeof(bvecs) / sizeof(bvecs[0]); ++v) {
                size_t n = bvecs[v].len;
                for (size_t k = 0; k < n; ++k) {
                    buf[k] = (uint8_t)(k & 0xFF);
                }
                char label[32];
                (void)snprintf(label, sizeof(label), "ramp[%llu]", (unsigned long long)n);
                check_hashes(buf, n, label, bvecs[v].sha256, bvecs[v].sha512, bvecs[v].md5, &failures);
            }
            free(buf);
        }
    }

    /* CF-variant plumbing: whitespace stripping must not change the hash
     * of already-whitespace-free input, and must strip \t\n\r and space. */
    {
        const uint8_t plain[] = "foo";
        uint8_t stripped[16];
        size_t n = murmur2cf_strip(plain, 3, stripped);
        if (n != 3 || memcmp(stripped, plain, 3) != 0) {
            (void)fprintf(stderr, "FAIL murmur2cf_strip: no-op case changed input\n");
            failures++;
        }

        const uint8_t padded[] = "f\too\n";
        n = murmur2cf_strip(padded, sizeof(padded) - 1, stripped);
        if (n != 3 || memcmp(stripped, plain, 3) != 0) {
            (void)fprintf(stderr, "FAIL murmur2cf_strip: whitespace not stripped correctly\n");
            failures++;
        }
    }

    if (failures == 0) {
        (void)fprintf(stderr, "hashutil: all self-tests passed\n");
    }
    return failures;
}

/* Reads one newline-terminated line from f into a heap buffer that grows
 * (doubling) as needed -- no fixed cap, so a long file path can't be
 * silently truncated or, worse, split across two hash_one() calls the
 * way a fixed-size fgets() buffer would on overflow (fgets() leaves the
 * remainder of an overlong line in the stream to be read as if it were
 * the start of the next path, desyncing the whole batch with no error).
 * Strips a trailing \r if present (CRLF input). Returns 1 with *out set
 * (caller frees) if a line was read; 0 at EOF with nothing left. */
static int read_line_dynamic(FILE *f, char **out) {
    size_t cap = 256;
    size_t len = 0;
    char *buf = (char *)malloc(cap);
    if (!buf) {
        (void)fprintf(stderr, "hashutil: out of memory reading stdin\n");
        exit(1);
    }

    int c;
    int any = 0;
    while ((c = fgetc(f)) != EOF) {
        any = 1;
        if (c == '\n') {
            break;
        }
        if (len + 1 >= cap) {
            cap *= 2;
            char *grown = (char *)realloc(buf, cap);
            if (!grown) {
                free(buf);
                (void)fprintf(stderr, "hashutil: out of memory reading stdin\n");
                exit(1);
            }
            buf = grown;
        }
        buf[len++] = (char)c;
    }

    if (!any) {
        free(buf);
        return 0;
    }
    if (len > 0 && buf[len - 1] == '\r') {
        len--;
    }
    buf[len] = '\0';
    *out = buf;
    return 1;
}

int main(int argc, char **argv) {
    unsigned algos = HASHUTIL_ALGO_ALL;

    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--selftest") == 0) {
            return selftest() == 0 ? 0 : 1;
        }
        if (strncmp(argv[i], "--algos=", 8) == 0) {
            algos = parse_algos(argv[i] + 8);
        } else {
            (void)fprintf(stderr, "hashutil: unknown argument '%s'\n", argv[i]);
            (void)fprintf(stderr, "usage: hashutil [--algos=sha256,sha512,md5,murmur2] < paths.txt\n");
            return 2;
        }
    }

    char *line;
    while (read_line_dynamic(stdin, &line)) {
        if (line[0] != '\0') {
            hash_one(line, algos);
        }
        free(line);
    }

    return 0;
}
