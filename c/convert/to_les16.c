// to_les16.c - Convert to little-endian signed 16-bit.
#include "c/convert/convert.h"

#include <math.h>
#include <string.h>

#if __BYTE_ORDER__ && __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
static inline unsigned short le16(unsigned short x) {
    return x;
}
#elif __BYTE_ORDER__ && __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
static inline unsigned short le16(unsigned short x) {
    return __builtin_bswap16(x);
}
#else
#error "unknown byte order"
#endif

void ufxr_to_les16(int n, void *restrict out, const float *restrict xs) {
    char *pos = out;
    for (int i = 0; i < n; i++) {
        float x = xs[i] * 32768.0f;
        short y;
        if (x < 32767.0f) {
            if (x > -32768.0f) {
                y = lrintf(x);
            } else {
                y = -32768;
            }
        } else {
            y = 32767;
        }
        y = le16(y);
        memcpy(pos, &y, 2);
        pos += 2;
    }
}
