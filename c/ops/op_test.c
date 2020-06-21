#include "c/ops/ops.h"
#include "c/util/defs.h"
#include "c/util/flag.h"
#include "c/util/util.h"

#include <math.h>
#include <stdlib.h>

// Calculate exponential function error in cents.
static float exp2_err(int n, const float *restrict ys,
                      const float *restrict xs) {
    float max_error = -1.0f;
    for (int i = 0; i < n; i++) {
        float error = 1200.0f * fabsf(log2f(ys[i]) - xs[i]);
        if (error > max_error) {
            max_error = error;
        }
    }
    return max_error;
}

// A simple reference version of the triangle operator. This is not supposed to
// be fast or especially accurate, it is supposed to be obviously correct.
static float tri(float x) {
    x = 4.0f * fmodf(x, 1.0f);
    if (x < 0.0f)
        x += 4.0f;
    if (x < 1.0f)
        return x;
    if (x < 3.0f)
        return 2.0f - x;
    return x - 4.0f;
}

// Calculate triangle function error as maximum difference.
static float tri_err(int n, const float *restrict ys,
                     const float *restrict xs) {
    float max_error = -1.0f;
    for (int i = 0; i < n; i++) {
        float error = fabsf(ys[i] - tri(xs[i]));
        if (error > max_error) {
            max_error = error;
        }
    }
    return max_error;
}

struct func_info {
    char name[8];
    // Evaluate function
    void (*func)(int n, float *restrict outs, const float *restrict xs);
    // Get error for function
    float (*errf)(int n, const float *restrict ys, const float *restrict xs);
    // Maximum permitted error
    float error;
};

#define F(f, g, e) \
    { #f, ufxr_##f, g, e }
static const struct func_info kFuncs[] = {
    F(exp2_2, exp2_err, 2.9888e0),  //
    F(exp2_3, exp2_err, 1.2960e-1), //
    F(exp2_4, exp2_err, 4.7207e-3), //
    F(exp2_5, exp2_err, 5.7220e-4), //
    F(exp2_6, exp2_err, 2.8610e-4), //
    F(tri, tri_err, 2.0f),          //
};
#undef F

// Extra margin for error, a ratio.
static const float kErrorMargin = 0.005f;

int main(int argc, char **argv) {
    int size = 1 << 20;
    flag_int(&size, "size", "array size");
    argc = flag_parse(argc, argv);
    if (size < 1) {
        die(0, "invalid size");
    }
    if ((size % UFXR_QUANTUM) != 0) {
        dief(0, "array size, %d, is not a multiple of array quantum, %d", size,
             UFXR_QUANTUM);
    }

    float *xs = xmalloc(size * sizeof(float));
    float *ys = xmalloc(size * sizeof(float));
    linspace(size, xs, -5.0f, 5.0f);

    bool success = true;
    for (size_t i = 0; i < ARRAY_SIZE(kFuncs); i++) {
        printf("Testing: %s\n", kFuncs[i].name);
        kFuncs[i].func(size, ys, xs);
        float error = kFuncs[i].errf(size, ys, xs);
        printf("Error: %.4e\n", (double)error);
        printf("Max error: %.4e\n", (double)kFuncs[i].error);
        if (error > kFuncs[i].error * (1.0f + kErrorMargin)) {
            puts("****FAIL****");
            success = false;
        } else if (error < kFuncs[i].error * (1.0f - kErrorMargin)) {
            puts("****IMPROVED****");
        }
        putc('\n', stdout);
        fflush(stdout);
    }

    if (!success) {
        puts("****FAIL****");
        exit(1);
    }

    return 0;
}
