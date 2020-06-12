// c/math/math.h - Math functions.
#pragma once

// Compute out = 2^x. Array size must be multiple of 16.
void ufxr_exp2_2(int n, float *restrict outs, const float *restrict xs);
void ufxr_exp2_3(int n, float *restrict outs, const float *restrict xs);
void ufxr_exp2_4(int n, float *restrict outs, const float *restrict xs);
void ufxr_exp2_5(int n, float *restrict outs, const float *restrict xs);
void ufxr_exp2_6(int n, float *restrict outs, const float *restrict xs);
