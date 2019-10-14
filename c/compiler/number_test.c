#include "c/compiler/number.h"
#include "c/compiler/testutil.h"

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

struct testcase {
    const char *number;
    int exponent;
    int rcode;
    double result;
};

static const char *num_code(int code) {
    switch (code) {
    case NUM_OK:
        return "ok";
    case NUM_INEXACT:
        return "inexact";
    case NUM_OVERFLOW:
        return "overflow";
    case NUM_UNDERFLOW:
        return "underflow";
    default:
        return "<unknown>";
    }
}

static bool run_test(size_t index, const struct testcase *restrict t) {
    size_t ndigits = strlen(t->number);
    uint8_t digits[30];
    for (size_t i = 0; i < ndigits; i++) {
        digits[i] = t->number[i] - '0';
    }
    double out;
    int result = ufxr_dectofloat(&out, digits, ndigits, t->exponent);
    if (result == t->rcode && out == t->result) {
        return true;
    }
    fprintf(stderr,
            "%zu: dectofloat(%s, %d): (%s, %g / %s), expect (%s, %g / %s)\n",
            index, t->number, t->exponent, num_code(result), out,
            show_float(out), num_code(t->rcode), t->result,
            show_float(t->result));
    temp_free();
    return false;
}

static const struct testcase TEST_CASES[] = {
    {"3", 0, NUM_OK, 3.0},
    {"12", 0, NUM_OK, 12.0},
    {"5", 1, NUM_OK, 50.0},
    {"999999999999999", -15, NUM_OK, 0.999999999999999},
    {"1", 22, NUM_OK, 1e22},
    {"1", -22, NUM_OK, 1e-22},
    {"1", 400, NUM_OVERFLOW, 1.0 / 0.0},
    {"1", -400, NUM_UNDERFLOW, 0.0},
};

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;
    bool success = true;
    size_t n = sizeof(TEST_CASES) / sizeof(*TEST_CASES);
    for (size_t i = 0; i < n; i++) {
        bool passed = run_test(i, &TEST_CASES[i]);
        if (!passed) {
            success = false;
        }
    }
    if (!success) {
        fputs("FAILED\n", stderr);
        return 1;
    }
    return 0;
}
