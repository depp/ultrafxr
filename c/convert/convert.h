// c/ops/convert.h - Sample type conversions.

// These functions convert aligned floating-point data to and from various
// unaligned types.

// Quantize to unsigned 8-bit integer.
void ufxr_to_u8(int n, void *restrict out, const float *restrict xs);

// Quantize to signed 16-bit little-endian integer.
void ufxr_to_les16(int n, void *restrict out, const float *restrict xs);

// Quantize to signed 24-bit little-endian integer.
void ufxr_to_les24(int n, void *restrict out, const float *restrict xs);

// Convert to 32-bit little-endian float.
void ufxr_to_lef32(int n, void *restrict out, const float *restrict xs);
