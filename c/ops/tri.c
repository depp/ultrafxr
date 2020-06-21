// tri.c - Triangle waveform.
#include "c/ops/impl.h"

#include <assert.h>

#if !HAVE_FUNC && USE_SSE2
#define HAVE_FUNC 1
#include <xmmintrin.h>
void ufxr_tri(int n, float *restrict outs, const float *restrict xs) {
    assert((n % UFXR_QUANTUM) == 0);
    const __m128 c0 = _mm_set1_ps(2.0f);
    const __m128 c1 = _mm_sub_ps(_mm_set1_ps(0.0f), c0);
    const __m128 c2 = _mm_set1_ps(4.0f);
    for (int i = 0; i < n; i += 4) {
        __m128 x = _mm_load_ps(xs + i);
        x = _mm_mul_ps(_mm_sub_ps(x, _mm_cvtepi32_ps(_mm_cvtps_epi32(x))), c2);
        x = _mm_max_ps(_mm_min_ps(x, _mm_sub_ps(c0, x)), _mm_sub_ps(c1, x));
        _mm_store_ps(outs + i, x);
    }
}
#endif

// Scalar version.
#if !HAVE_FUNC
#include <math.h>
void ufxr_tri(int n, float *restrict outs, const float *restrict xs) {
    assert((n % UFXR_QUANTUM) == 0);
    for (int i = 0; i < n; i++) {
        float x = xs[i];
        x -= rintf(x);
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
