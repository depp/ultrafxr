// to_les16.c - Convert to little-endian signed 16-bit.
#include "c/convert/convert.h"

#include "c/config/config.h"

// SSE2 version.
#if !HAVE_FUNC && USE_SSE2
#define HAVE_FUNC 1
#include <string.h>
#include <xmmintrin.h>
void ufxr_to_les16(int n, void *restrict out, const float *restrict xs) {
    char *optr = out;
    const float *iptr = xs, *iend = xs + n;
    const __m128 scale = _mm_set1_ps(32768.0f);
    const __m128 max = _mm_set1_ps(32767.0f);
    const __m128 min = _mm_set1_ps(-32768.0f);
    while (((uintptr_t)iptr & 15) != 0) {
        __m128 x = _mm_load_ss(iptr);
        x = _mm_mul_ss(x, scale);
        x = _mm_max_ss(x, min);
        x = _mm_min_ss(x, max);
        int s = _mm_cvt_ss2si(x);
        memcpy(optr, &s, 2);
        iptr += 1;
        optr += 2;
    }
    while (iend - iptr >= 8) {
        __m128 x0 = _mm_load_ps(iptr);
        __m128 x1 = _mm_load_ps(iptr + 4);
        x0 = _mm_mul_ps(x0, scale);
        x1 = _mm_mul_ps(x1, scale);
        x0 = _mm_max_ps(x0, min);
        x1 = _mm_max_ps(x1, min);
        x0 = _mm_min_ps(x0, max);
        x1 = _mm_min_ps(x1, max);
        __m128i s0 = _mm_cvtps_epi32(x0);
        __m128i s1 = _mm_cvtps_epi32(x1);
        __m128i s = _mm_packs_epi32(s0, s1);
        _mm_storeu_si128((void *)optr, s);
        iptr += 8;
        optr += 16;
    }
    while (iptr != iend) {
        __m128 x = _mm_load_ss(iptr);
        x = _mm_mul_ss(x, scale);
        x = _mm_max_ss(x, min);
        x = _mm_min_ss(x, max);
        int s = _mm_cvt_ss2si(x);
        memcpy(optr, &s, 2);
        iptr += 1;
        optr += 2;
    }
}
#endif

// Scalar version.
#if !HAVE_FUNC
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
#endif
