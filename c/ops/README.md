# Operators

The `//c/ops` library provides low-level implementations of signal processing operators. These functions operate on arrays, and the size of the arrays must be a multiple of ARRAY_QUANTUM, which is currently 4. This requirement greatly simplifies the implementation of SIMD versions of these functions.

These functions will use SIMD if an appropriate implementation exists.
