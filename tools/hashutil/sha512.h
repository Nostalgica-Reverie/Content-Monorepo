/* SHA-512 (FIPS 180-4), single-header, dependency-free.
 * Original implementation against the FIPS 180-4 spec; K/H constants
 * generated from their defining formulas (fractional bits of cube/square
 * roots of the first primes), not transcribed from another source. */
#ifndef HASHUTIL_SHA512_H
#define HASHUTIL_SHA512_H

#include <stddef.h>
#include <stdint.h>
#include <string.h>

typedef struct {
    uint64_t state[8];
    uint64_t bitlen_lo, bitlen_hi; /* 128-bit total length, in bits */
    uint8_t data[128];
    uint32_t datalen;
} sha512_ctx;

static const uint64_t sha512_k[80] = {
    0x428a2f98d728ae22ULL, 0x7137449123ef65cdULL, 0xb5c0fbcfec4d3b2fULL, 0xe9b5dba58189dbbcULL,
    0x3956c25bf348b538ULL, 0x59f111f1b605d019ULL, 0x923f82a4af194f9bULL, 0xab1c5ed5da6d8118ULL,
    0xd807aa98a3030242ULL, 0x12835b0145706fbeULL, 0x243185be4ee4b28cULL, 0x550c7dc3d5ffb4e2ULL,
    0x72be5d74f27b896fULL, 0x80deb1fe3b1696b1ULL, 0x9bdc06a725c71235ULL, 0xc19bf174cf692694ULL,
    0xe49b69c19ef14ad2ULL, 0xefbe4786384f25e3ULL, 0x0fc19dc68b8cd5b5ULL, 0x240ca1cc77ac9c65ULL,
    0x2de92c6f592b0275ULL, 0x4a7484aa6ea6e483ULL, 0x5cb0a9dcbd41fbd4ULL, 0x76f988da831153b5ULL,
    0x983e5152ee66dfabULL, 0xa831c66d2db43210ULL, 0xb00327c898fb213fULL, 0xbf597fc7beef0ee4ULL,
    0xc6e00bf33da88fc2ULL, 0xd5a79147930aa725ULL, 0x06ca6351e003826fULL, 0x142929670a0e6e70ULL,
    0x27b70a8546d22ffcULL, 0x2e1b21385c26c926ULL, 0x4d2c6dfc5ac42aedULL, 0x53380d139d95b3dfULL,
    0x650a73548baf63deULL, 0x766a0abb3c77b2a8ULL, 0x81c2c92e47edaee6ULL, 0x92722c851482353bULL,
    0xa2bfe8a14cf10364ULL, 0xa81a664bbc423001ULL, 0xc24b8b70d0f89791ULL, 0xc76c51a30654be30ULL,
    0xd192e819d6ef5218ULL, 0xd69906245565a910ULL, 0xf40e35855771202aULL, 0x106aa07032bbd1b8ULL,
    0x19a4c116b8d2d0c8ULL, 0x1e376c085141ab53ULL, 0x2748774cdf8eeb99ULL, 0x34b0bcb5e19b48a8ULL,
    0x391c0cb3c5c95a63ULL, 0x4ed8aa4ae3418acbULL, 0x5b9cca4f7763e373ULL, 0x682e6ff3d6b2b8a3ULL,
    0x748f82ee5defb2fcULL, 0x78a5636f43172f60ULL, 0x84c87814a1f0ab72ULL, 0x8cc702081a6439ecULL,
    0x90befffa23631e28ULL, 0xa4506cebde82bde9ULL, 0xbef9a3f7b2c67915ULL, 0xc67178f2e372532bULL,
    0xca273eceea26619cULL, 0xd186b8c721c0c207ULL, 0xeada7dd6cde0eb1eULL, 0xf57d4f7fee6ed178ULL,
    0x06f067aa72176fbaULL, 0x0a637dc5a2c898a6ULL, 0x113f9804bef90daeULL, 0x1b710b35131c471bULL,
    0x28db77f523047d84ULL, 0x32caab7b40c72493ULL, 0x3c9ebe0a15c9bebcULL, 0x431d67c49c100d4cULL,
    0x4cc5d4becb3e42b6ULL, 0x597f299cfc657e2aULL, 0x5fcb6fab3ad6faecULL, 0x6c44198c4a475817ULL,
};

#define SHA512_ROTR(a, b) (((a) >> (b)) | ((a) << (64 - (b))))

static void sha512_transform(sha512_ctx *ctx, const uint8_t data[128]) {
    uint64_t m[80];
    for (int i = 0, j = 0; i < 16; ++i, j += 8) {
        m[i] = ((uint64_t)data[j] << 56) | ((uint64_t)data[j + 1] << 48) | ((uint64_t)data[j + 2] << 40) |
               ((uint64_t)data[j + 3] << 32) | ((uint64_t)data[j + 4] << 24) | ((uint64_t)data[j + 5] << 16) |
               ((uint64_t)data[j + 6] << 8) | ((uint64_t)data[j + 7]);
    }
    for (int i = 16; i < 80; ++i) {
        uint64_t s0 = SHA512_ROTR(m[i - 15], 1) ^ SHA512_ROTR(m[i - 15], 8) ^ (m[i - 15] >> 7);
        uint64_t s1 = SHA512_ROTR(m[i - 2], 19) ^ SHA512_ROTR(m[i - 2], 61) ^ (m[i - 2] >> 6);
        m[i] = m[i - 16] + s0 + m[i - 7] + s1;
    }

    uint64_t a = ctx->state[0];
    uint64_t b = ctx->state[1];
    uint64_t c = ctx->state[2];
    uint64_t d = ctx->state[3];
    uint64_t e = ctx->state[4];
    uint64_t f = ctx->state[5];
    uint64_t g = ctx->state[6];
    uint64_t h = ctx->state[7];

    for (int i = 0; i < 80; ++i) {
        uint64_t s1 = SHA512_ROTR(e, 14) ^ SHA512_ROTR(e, 18) ^ SHA512_ROTR(e, 41);
        uint64_t ch = (e & f) ^ (~e & g);
        uint64_t t1 = h + s1 + ch + sha512_k[i] + m[i];
        uint64_t s0 = SHA512_ROTR(a, 28) ^ SHA512_ROTR(a, 34) ^ SHA512_ROTR(a, 39);
        uint64_t maj = (a & b) ^ (a & c) ^ (b & c);
        uint64_t t2 = s0 + maj;
        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }

    ctx->state[0] += a;
    ctx->state[1] += b;
    ctx->state[2] += c;
    ctx->state[3] += d;
    ctx->state[4] += e;
    ctx->state[5] += f;
    ctx->state[6] += g;
    ctx->state[7] += h;
}

static void sha512_init(sha512_ctx *ctx) {
    ctx->datalen = 0;
    ctx->bitlen_lo = 0;
    ctx->bitlen_hi = 0;
    ctx->state[0] = 0x6a09e667f3bcc908ULL;
    ctx->state[1] = 0xbb67ae8584caa73bULL;
    ctx->state[2] = 0x3c6ef372fe94f82bULL;
    ctx->state[3] = 0xa54ff53a5f1d36f1ULL;
    ctx->state[4] = 0x510e527fade682d1ULL;
    ctx->state[5] = 0x9b05688c2b3e6c1fULL;
    ctx->state[6] = 0x1f83d9abfb41bd6bULL;
    ctx->state[7] = 0x5be0cd19137e2179ULL;
}

/* One completed 128-byte block's worth of bits, with 128-bit carry. */
static void sha512_count_block(sha512_ctx *ctx) {
    uint64_t old_lo = ctx->bitlen_lo;
    ctx->bitlen_lo += 1024;
    if (ctx->bitlen_lo < old_lo) {
        ctx->bitlen_hi++; /* carry */
    }
}

/* Block-at-a-time (same shape as md5_update): fill a partial block via
 * memcpy, then transform full 128-byte blocks straight from the input, then
 * stash the tail. bitlen counts completed blocks only; sha512_final adds the
 * tail bits, matching the original byte-at-a-time accounting. */
static void sha512_update(sha512_ctx *ctx, const uint8_t *data, size_t len) {
    size_t i = 0;
    if (ctx->datalen > 0) {
        size_t need = 128 - ctx->datalen;
        if (len < need) {
            memcpy(ctx->data + ctx->datalen, data, len);
            ctx->datalen += (uint32_t)len;
            return;
        }
        memcpy(ctx->data + ctx->datalen, data, need);
        sha512_transform(ctx, ctx->data);
        sha512_count_block(ctx);
        ctx->datalen = 0;
        i = need;
    }
    for (; i + 128 <= len; i += 128) {
        sha512_transform(ctx, data + i);
        sha512_count_block(ctx);
    }
    memcpy(ctx->data, data + i, len - i);
    ctx->datalen = (uint32_t)(len - i);
}

static void sha512_final(sha512_ctx *ctx, uint8_t hash[64]) {
    uint32_t i = ctx->datalen;

    if (ctx->datalen < 112) {
        ctx->data[i++] = 0x80;
        while (i < 112) {
            ctx->data[i++] = 0x00;
        }
    } else {
        ctx->data[i++] = 0x80;
        while (i < 128) {
            ctx->data[i++] = 0x00;
        }
        sha512_transform(ctx, ctx->data);
        memset(ctx->data, 0, 112);
    }

    uint64_t old_lo = ctx->bitlen_lo;
    ctx->bitlen_lo += (uint64_t)ctx->datalen * 8;
    if (ctx->bitlen_lo < old_lo) {
        ctx->bitlen_hi++;
    }

    for (int i2 = 0; i2 < 8; ++i2) {
        ctx->data[127 - i2] = (uint8_t)(ctx->bitlen_lo >> (8 * i2));
    }
    for (int i2 = 0; i2 < 8; ++i2) {
        ctx->data[119 - i2] = (uint8_t)(ctx->bitlen_hi >> (8 * i2));
    }
    sha512_transform(ctx, ctx->data);

    for (i = 0; i < 8; ++i) {
        for (uint32_t j = 0; j < 8; ++j) {
            hash[j * 8 + i] = (uint8_t)(ctx->state[j] >> (56 - i * 8));
        }
    }
}

#endif /* HASHUTIL_SHA512_H */
