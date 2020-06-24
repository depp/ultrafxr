// to_les24.c - Convert to little-endian signed 24-bit.
#include "c/convert/convert.h"

#include "c/config/config.h"

#if __BYTE_ORDER__ && __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
#include <string.h>
void ufxr_to_lef32(int n, void *restrict out, const float *restrict xs) {
    memcpy(out, xs, n * sizeof(float));
}
#elif __BYTE_ORDER__ && __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
void ufxr_to_lef32(int n, void *restrict out, const float *restrict xs) {
    char *optr = out;
    for (int i = 0; i < n; i++) {
        union {
            float f;
            unsigned u;
        } d;
        d.f = xs[i];
        d.u = __builtin_bswap32(d.u);
        memcpy(optr, &d, 4);
        optr += 4;
    }
}
#else
#error "unknown endian"
#endif
