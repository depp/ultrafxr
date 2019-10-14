// number_dectofloat.c - Decimal to floating-point conversion.
#include "c/compiler/number.h"

#include <math.h>

enum {
    EXACT_POW10 = 22,
};

// Powers of ten, all with full precision (1e23 is rounded).
static const double POW10[EXACT_POW10] = {
    1e1,  1e2,  1e3,  1e4,  1e5,  1e6,  1e7,  1e8,  1e9,  1e10, 1e11,
    1e12, 1e13, 1e14, 1e15, 1e16, 1e17, 1e18, 1e19, 1e20, 1e21, 1e22,
};

// Compute a power of 10, n must be strictly positive.
static double pow10(int n) {
    if (n <= EXACT_POW10) {
        return POW10[n - 1];
    }
    double x = POW10[EXACT_POW10 - 1];
    n -= EXACT_POW10;
    while (n > EXACT_POW10) {
        x *= POW10[EXACT_POW10 - 1];
        n -= EXACT_POW10;
    }
    return x * POW10[n - 1];
}

int ufxr_dectofloat(double *restrict out, const uint8_t *restrict digits,
                    uint32_t ndigits, int exponent) {
    // Remove leading and trailing zeroes.
    uint32_t pos = 0;
    while (pos < ndigits && digits[pos] == 0) {
        pos++;
    }
    uint32_t end = ndigits;
    while (pos < end && digits[end - 1] == 0) {
        end--;
    }
    // Shortcut for zero, which cannot overflow or underflow.
    if (pos == end) {
        *out = 0.0;
        return 0;
    }
    // Calculate overflow from exponent out of range.
    int dexp = ndigits - pos; // Decimal exponent of leading digit.
    if (exponent > 308 - dexp) {
        *out = 1.0 / 0.0;
        return NUM_OVERFLOW;
    } else if (exponent < 0 && exponent + dexp < -323) {
        *out = 0.0;
        return NUM_UNDERFLOW;
    }
    // Convert digits to an integer. We can convert 15 digits without worrying
    // about overflowing 53 bits. Minor note: the i64 -> double conversion is
    // marginally faster on some architectures. We also just truncate once we
    // get 15 digits.
    int64_t ival = 0;
    const uint32_t precision = 15;
    int result = NUM_OK;
    uint32_t maxpos = end;
    if (end > pos + precision) {
        maxpos = pos + precision;
        result = NUM_INEXACT;
    }
    for (; pos < maxpos; pos++) {
        ival = ival * 10 + digits[pos];
    }
    // Calculate the correct exponent to use.
    int expval = exponent + (int)(ndigits - maxpos);
    double fval = ival;
    if (expval > 0) {
        if (expval > EXACT_POW10) {
            result = NUM_INEXACT;
        }
        fval *= pow10(expval);
        if (isinf(fval)) {
            result = NUM_OVERFLOW;
        }
    } else if (expval < 0) {
        if (-expval > EXACT_POW10) {
            result = NUM_INEXACT;
        }
        fval /= pow10(-expval);
        if (fval == 0.0) {
            result = NUM_UNDERFLOW;
        }
    }
    *out = fval;
    return result;
}
