// c/ops/ops.h - Low-level signal processing operators.
#pragma once

// All inputs to these functions must have a size which is a multiple of
// UFXR_QUANTUM.
#define UFXR_QUANTUM 4

// Minimum alignment for buffers.
#define UFXR_ALIGN 16

// Compute out = 2^x. Available in 2nd order to 6th order.
//
// Since this function is used for converting note values to frequencies, error
// is given in cents (which is equal to 1200 times the equivalent input error).
// The 3nd order function is generally preferred. The 2nd order function should
// be acceptable in many situations, since the 3.0 cent error is below the
// typical threshold of human pitch perception, which is about 10 cents. The 4th
// order function has less tuning error than the MIDI Tuning Standard, which has
// a tuning error of 0.0061 cents (100 / 2^14 cents).
//
// Worst-case error, in cents:
//   2: 3.0
//   3: 0.13
//   4: 0.0047
//   5: 0.00057
//   6: 0.00029
void ufxr_exp2_2(int n, float *restrict outs, const float *restrict xs);
void ufxr_exp2_3(int n, float *restrict outs, const float *restrict xs);
void ufxr_exp2_4(int n, float *restrict outs, const float *restrict xs);
void ufxr_exp2_5(int n, float *restrict outs, const float *restrict xs);
void ufxr_exp2_6(int n, float *restrict outs, const float *restrict xs);

// Generate oscillator phase from frequency input.
void ufxr_osc(int n, float *restrict outs, const float *restrict xs);

// Compute triangle waveform from phase. Period is 1. Output has same sign as
// sin(2 pi x).
void ufxr_tri(int n, float *restrict outs, const float *restrict xs);

// Compute out = sin(2 pi x). Available with complexity 2 to 6.
//
// The 2nd order version is two parabolas, one for the positive and one for the
// negative half of the wave. This is extremely fast and pure enough to use as
// a sine tone or for phase modulation synthesis.
//
// The higher-order versions are chosen to fix f(0) = 0 and minimize the maximum
// error. For calculating filter coefficients, the 4th order version should be
// accurate enough to use for filters tuned to specific notes, but its exact
// accuracy for this purpose has not yet been measured.
void ufxr_sin1_2(int n, float *restrict outs, const float *restrict xs);
void ufxr_sin1_3(int n, float *restrict outs, const float *restrict xs);
void ufxr_sin1_4(int n, float *restrict outs, const float *restrict xs);
void ufxr_sin1_5(int n, float *restrict outs, const float *restrict xs);
void ufxr_sin1_6(int n, float *restrict outs, const float *restrict xs);
