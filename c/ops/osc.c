// Oscillator operator.
#include "c/ops/impl.h"

#include <math.h>

void ufxr_osc(int n, float *restrict outs, const float *restrict xs) {
    CHECK2(n, outs, xs);
    float phase = 0.0f;
    for (int i = 0; i < n; i++) {
        phase += xs[i];
        phase -= rintf(phase);
        outs[i] = phase;
    }
}
