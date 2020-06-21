// sin1_2.c - Quadratic sin approximation.
#include "c/ops/impl.h"

#include <assert.h>

#if !HAVE_FUNC && USE_SSE4_1
#define HAVE_FUNC 1
#include <smmintrin.h>
void ufxr_sin1_2(int n, float *restrict outs, const float *restrict xs) {
    assert((n % UFXR_QUANTUM) == 0);
    const __m128 abs = _mm_castsi128_ps(_mm_srli_epi32(_mm_set1_epi32(-1), 1));
    const __m128 c2 = _mm_set1_ps(8.0f);
    const __m128 c3 = _mm_set1_ps(16.0f);
    for (int i = 0; i < n; i += 4) {
        __m128 x = _mm_load_ps(xs + i);
        x = _mm_sub_ps(
            x, _mm_round_ps(x, _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC));
        x = _mm_mul_ps(x, _mm_sub_ps(c2, _mm_mul_ps(c3, _mm_and_ps(x, abs))));
        _mm_store_ps(outs + i, x);
    }
}
#endif

#if !HAVE_FUNC && USE_SSE2
#define HAVE_FUNC 1
#include <xmmintrin.h>
void ufxr_sin1_2(int n, float *restrict outs, const float *restrict xs) {
    assert((n % UFXR_QUANTUM) == 0);
    const __m128 abs = _mm_castsi128_ps(_mm_srli_epi32(_mm_set1_epi32(-1), 1));
    const __m128 c2 = _mm_set1_ps(8.0f);
    const __m128 c3 = _mm_set1_ps(16.0f);
    for (int i = 0; i < n; i += 4) {
        __m128 x = _mm_load_ps(xs + i);
        x = _mm_sub_ps(x, _mm_cvtepi32_ps(_mm_cvtps_epi32(x)));
        x = _mm_mul_ps(x, _mm_sub_ps(c2, _mm_mul_ps(c3, _mm_and_ps(x, abs))));
        _mm_store_ps(outs + i, x);
    }
}
#endif

// Scalar version.
#if !HAVE_FUNC
#include <math.h>
void ufxr_sin1_2(int n, float *restrict outs, const float *restrict xs) {
    assert((n % UFXR_QUANTUM) == 0);
    for (int i = 0; i < n; i++) {
        float x = xs[i];
        x -= rintf(x);
        outs[i] = x * (8.0f - 16.0f * fabsf(x));
    }
}
#endif
