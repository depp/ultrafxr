// tri.c - Triangle waveform.
#include "c/ops/ops.h"

#include <assert.h>

// Scalar version.
#if !HAVE_FUNC
#include <math.h>
void ufxr_tri(int n, float *restrict outs, const float *restrict xs) {
    assert((n % UFXR_QUANTUM) == 0);
    for (int i = 0; i < n; i++) {
        float x = xs[i];
        x -= (float)(int)x;
        float t1 = 0.5f - x;
        float t2 = -0.5f - x;
        if (t1 < x)
            x = t1;
        if (t2 > x)
            x = t2;
        outs[i] = x * 4.0f;
    }
}
#endif
