// to_les24.c - Convert to little-endian signed 24-bit.
#include "c/convert/convert.h"

#include "c/config/config.h"

#if !HAVE_FUNC
#include <math.h>
#include <string.h>

#if __BYTE_ORDER__ && __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
static inline unsigned le32(unsigned x) {
    return x;
}
#elif __BYTE_ORDER__ && __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
static inline unsigned le32(unsigned x) {
    return __builtin_bswap32(x);
}
#else
#error "unknown byte order"
#endif

void ufxr_to_les24(int n, void *restrict out, const float *restrict xs) {
    char *pos = out;
    for (int i = 0; i < n; i++) {
        float x = xs[i] * 8388608.0f;
        int y;
        if (x < 8388607.0f) {
            if (x > -8388608.0f) {
                y = lrintf(x);
            } else {
                y = -8388608;
            }
        } else {
            y = 8388607;
        }
        y = le32(y);
        memcpy(pos, &y, 3);
        pos += 3;
    }
}
#endif
