// number.h - Number conversions.
#pragma once

#include <stdint.h>

enum {
    // Number converted exactly (and then rounded, if floating-point).
    NUM_OK,
    // Number not converted exactly (because we use a simple algorithm).
    NUM_INEXACT,
    // Number magnitude was too large, converted to infinity.
    NUM_OVERFLOW,
    // Number magnitude was too low, converted to zero.
    NUM_UNDERFLOW,
};

// Convert decimal to double precision floating-point number. Always succeeds
// and returns one of the status codes above. This will only give the correctly
// rounded result for a limited range of input values, as long as there aren't
// more than 15 digits and the exponent is in the range -22 to +22. Outside that
// range, NUM_INEXACT may be returned.
int ufxr_dectofloat(double *restrict out, const uint8_t *restrict digits,
                    uint32_t ndigits, int exponent);
