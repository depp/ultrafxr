// hash.c - Simple hash function implementation.
#include "c/compiler/hash.h"

#include <string.h>

static uint32_t ufxr_read32(const unsigned char *p) {
    // This optimizes down to a single instruction where appropriate, but lets
    // us read unaligned pointers without invoking undefined behavior.
    uint32_t x;
    memcpy(&x, p, sizeof(x));
    return x;
}

uint32_t ufxr_hash(const void *data, size_t size) {
    // Murmur hash 3.
    const uint32_t c1 = 0xcc9e2d51, c2 = 0x1b873593;
    const unsigned char *p = data;
    size_t n = size / 4;
    uint32_t y = 0xc90fdaa2;

    for (size_t i = 0; i < n; i++) {
        uint32_t x = ufxr_read32(p + 4 * i);
        x *= c1;
        x = (x << 15) | (x >> 17);
        x *= c2;
        y ^= x;
        y = (y << 13) | (y >> 19);
        y = y * 5 + 0xe6546b64;
    }

    uint32_t x = 0;
    switch (size & 3) {
    case 3:
        x = p[4 * n + 2] << 16;
        /* fallthrough */
    case 2:
        x |= p[4 * n + 1] << 8;
        /* fallthrough */
    case 1:
        x |= p[4 * n + 0];
        x *= c1;
        x = (x << 15) | (x >> 17);
        x *= c2;
        y ^= x;
    }

    y ^= size;

    uint64_t z = y;
    z *= 0xff51afd7ed558ccdull;
    z ^= z >> 33;
    z *= 0xc4ceb9fe1a85ec53ull;
    z ^= z >> 33;
    y = z;

    return y;
}
