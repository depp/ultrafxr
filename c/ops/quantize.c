// quantize.c - Audio quantization.
#include "c/ops/impl.h"

#include <math.h>

void ufxr_quantize_i16(int n, short *restrict outs, const float *restrict xs) {
    CHECK2(n, outs, xs);
    for (int i = 0; i < n; i++) {
        float x = xs[i] * 32768.0f;
        int y;
        if (x < 32767.0f) {
            if (x > -32768.0f) {
                y = lrintf(x);
            } else {
                y = -32768;
            }
        } else {
            y = 32767;
        }
        outs[i] = y;
    }
}
