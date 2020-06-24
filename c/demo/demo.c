#include "c/io/error.h"
#include "c/io/wave.h"
#include "c/ops/ops.h"
#include "c/util/flag.h"
#include "c/util/util.h"

#include <limits.h>
#include <math.h>
#include <stdlib.h>

enum {
    kRateMin = 8000,
    kRateMax = 192000,
};

int main(int argc, char **argv) {
    // Parse arguments.
    float f0 = 100.0f, f1 = 5000.0f;
    int samplerate = 48000;
    float length = 1.0f;
    const char *outpath = NULL;
    flag_float(&f0, "f0", "starting frequency, Hz");
    flag_float(&f1, "f1", "ending frequency, Hz");
    flag_int(&samplerate, "rate", "sample rate, Hz");
    flag_float(&length, "length", "audio length in seconds");
    flag_string(&outpath, "out", "output wav file");
    argc = flag_parse(argc, argv);
    if (argc != 0) {
        die_usagef("unexpected argument %s", quote_str(argv[0]));
    }
    if (samplerate < kRateMin) {
        die_usagef("sample rate %d is too small, must be in the range %d-%d",
                   samplerate, kRateMin, kRateMax);
    } else if (samplerate > kRateMax) {
        die_usagef("sample rate %d is too large, must be in the range %d-%d",
                   samplerate, kRateMin, kRateMax);
    }
    float nsamplef = rintf((float)samplerate * length);
    if (nsamplef < 1.0f) {
        die_usagef("length %fs is too short", (double)length);
    } else if (nsamplef > INT_MAX / 4) {
        die_usagef("length %fs is too long", (double)length);
    }
    int n = nsamplef;
    n = (n + UFXR_QUANTUM - 1) & ~(UFXR_QUANTUM - 1);
    if (outpath == NULL) {
        die_usage("missing required option -out");
    }

    // Generate samples.
    float *x1 = xmalloc(sizeof(float) * n);
    float *x2 = xmalloc(sizeof(float) * n);
    float d0 = 0.5f * f0 / (float)samplerate;
    float d1 = 0.5f * f1 / (float)samplerate;
    linspace(n, x1, log2f(d0), log2f(d1));
    ufxr_exp2_3(n, x2, x1);
    ufxr_osc(n, x1, x2);
    ufxr_sin1_2(n, x2, x1);

    // Write output.
    struct ufxr_wavewriter w;
    struct ufxr_error err;
    struct ufxr_waveinfo info = {
        .samplerate = samplerate,
        .channels = 1,
        .format = kUFXRFormatS16,
        .length = n,
    };
    if (!ufxr_wavewriter_create(&w, outpath, &info, &err)) {
        die(0, "error");
    }
    if (!ufxr_wavewriter_write(&w, x2, n, &err)) {
        die(0, "error");
    }
    if (!ufxr_wavewriter_finish(&w, &err)) {
        die(0, "error");
    }
    ufxr_wavewriter_destroy(&w);
    free(x1);
    free(x2);
    return 0;
}
