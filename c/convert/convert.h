// c/ops/convert.h - Sample type conversions.

// These functions convert aligned floating-point data to and from various
// unaligned types.

// Quantize to signed 16-bit little-endian integer without dithering.
void ufxr_to_les16(int n, void *restrict out, const float *restrict xs);
