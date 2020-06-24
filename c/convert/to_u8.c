// to_u8.c - Convert to unsigned 8-bit.
#include "c/convert/convert.h"

#include "c/config/config.h"

// Scalar version.
#if !HAVE_FUNC
#include <math.h>
void ufxr_to_u8(int n, void *restrict out, const float *restrict xs) {
    char *pos = out;
    for (int i = 0; i < n; i++) {
        float x = xs[i] * 128.0f + 128.0f;
        unsigned char y;
        if (x < 255.0f) {
            if (x > 0.0f) {
                y = lrintf(x);
            } else {
                y = 0;
            }
        } else {
            y = 255;
        }
        *pos++ = y;
    }
}
#endif
