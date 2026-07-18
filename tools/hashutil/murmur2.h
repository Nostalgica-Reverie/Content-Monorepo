/* MurmurHash2 (32-bit, Austin Appleby, public domain algorithm) plus the
 * CurseForge "CF" variant used by apps/packwand/curseforge/murmur2: strip
 * whitespace bytes, then MurmurHash2 the remainder with seed=1. Byte order
 * for 4-byte reads is explicit little-endian (matches the reference
 * algorithm and this repo's Go implementation, independent of host
 * endianness). Re-derived from the documented algorithm and cross-checked
 * against apps/packwand/curseforge/murmur2's test vectors; not a copy of
 * any specific existing source file. */
#ifndef HASHUTIL_MURMUR2_H
#define HASHUTIL_MURMUR2_H

#include <stddef.h>
#include <stdint.h>

#define MURMUR2_M 0x5bd1e995u
#define MURMUR2_R 24

static uint32_t murmurhash2(const uint8_t *data, size_t len, uint32_t seed) {
    uint32_t h = seed ^ (uint32_t)len;

    while (len >= 4) {
        uint32_t k = (uint32_t)data[0] | ((uint32_t)data[1] << 8) | ((uint32_t)data[2] << 16) |
                     ((uint32_t)data[3] << 24);
        k *= MURMUR2_M;
        k ^= k >> MURMUR2_R;
        k *= MURMUR2_M;
        h *= MURMUR2_M;
        h ^= k;
        data += 4;
        len -= 4;
    }

    switch (len) {
    case 3:
        h ^= (uint32_t)data[2] << 16;
        /* fallthrough */
    case 2:
        h ^= (uint32_t)data[1] << 8;
        /* fallthrough */
    case 1:
        h ^= (uint32_t)data[0];
        h *= MURMUR2_M;
        break;
    default:
        break;
    }

    h ^= h >> 13;
    h *= MURMUR2_M;
    h ^= h >> 15;

    return h;
}

static int murmur2cf_is_whitespace(uint8_t b) { return b == 9 || b == 10 || b == 13 || b == 32; }

/* Strips whitespace bytes from `data` in place into `out` (may be `data`
 * itself), returns the stripped length. Caller then hashes with seed=1. */
static size_t murmur2cf_strip(const uint8_t *data, size_t len, uint8_t *out) {
    size_t n = 0;
    for (size_t i = 0; i < len; ++i) {
        if (!murmur2cf_is_whitespace(data[i])) {
            out[n++] = data[i];
        }
    }
    return n;
}

#endif /* HASHUTIL_MURMUR2_H */
