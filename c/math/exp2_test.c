#include "c/math/math.h"
#include "c/util/flag.h"
#include "c/util/util.h"

#include <math.h>
#include <stdlib.h>
#include <time.h>

typedef void (*func)(int n, float *restrict outs, const float *restrict xs);

struct func_info {
    int order;
    func func;
    float error; // The amount of error we expect, in cents.
};

static const struct func_info kFuncs[] = {
    {2, ufxr_exp2_2, 2.989197f}, //
    {3, ufxr_exp2_3, 0.129604f}, //
    {4, ufxr_exp2_4, 0.004721f}, //
    {5, ufxr_exp2_5, 0.000572f}, //
    {6, ufxr_exp2_6, 0.000286f}, //
};

// Extra margin for error, a ratio.
static const float kErrorMargin = 1.005f;

int main(int argc, char **argv) {
    bool benchmark = false;
    int size = 1 << 20;
    int itercount = 1000;
    flag_bool(&benchmark, "benchmark", "run benchmarks");
    flag_int(&size, "size", "array size");
    flag_int(&itercount, "iterations", "benchmark iterations");
    argc = flag_parse(argc, argv);
    if (size < 1) {
        die(0, "invalid size");
    }
    if ((size_t)size > (size_t)-1 / sizeof(float)) {
        die(0, "size too large");
    }

    float *input = xmalloc(size * sizeof(float));
    float *output = xmalloc(size * sizeof(float));
    for (int i = 0; i < size; i++) {
        input[i] = (float)(i - size / 2) * (30.0f / (float)size);
    }

    bool success = true;
    for (size_t i = 0; i < ARRAY_SIZE(kFuncs); i++) {
        func f = kFuncs[i].func;
        f(size, output, input);
        float worst_error = 0.0f;
        float worst_x = 0.0f;
        float worst_y = 0.0f;
        for (int i = 0; i < size; i++) {
            float x = input[i];
            float y = output[i];
            float error = 1200.0f * fabsf(log2f(y) - x);
            if (error > worst_error) {
                worst_error = error;
                worst_x = x;
                worst_y = y;
            }
        }
        printf("Worst error: %.6f cent\n", (double)worst_error);
        printf("    %.4f -> %.4f\n", (double)worst_x, (double)worst_y);
        float max_error = kFuncs[i].error * kErrorMargin;
        if (worst_error > max_error) {
            fprintf(stderr, "Error: Error exceeds maximum error for order %d\n",
                    kFuncs[i].order);
            success = false;
        }

        if (benchmark) {
            struct timespec t0, t1;
            clock_gettime(CLOCK_MONOTONIC, &t0);
            for (int i = 0; i < itercount; i++) {
                f(size, output, input);
            }
            clock_gettime(CLOCK_MONOTONIC, &t1);
            double dt =
                1e9 * (t1.tv_sec - t0.tv_sec) + (t1.tv_nsec - t0.tv_nsec);
            printf("Time per sample: %.1f ns\n",
                   dt / (double)(itercount * size));
        }
    }

    if (!success) {
        fputs("Failed - error too large\n", stderr);
        exit(1);
    }

    return 0;
}
