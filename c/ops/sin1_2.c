// sin1_2.c - Quadratic sin approximation.
#include "c/ops/ops.h"

#include <assert.h>

// Scalar version.
#if !HAVE_FUNC
#include <math.h>
void ufxr_sin1_2(int n, float *restrict outs, const float *restrict xs) {
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
        outs[i] = x * (8.0f - 16.0f * fabsf(x));
    }
}
#endif
