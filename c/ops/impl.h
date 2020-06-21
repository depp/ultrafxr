// c/ops/impl.h - Definitions for operator implementations.
#pragma once
#include "c/ops/ops.h"

#include <stddef.h>
#include <stdint.h>
#include <stdnoreturn.h>

#if !USE_SCALAR

#define USE_SSE2 __SSE2__
#define USE_SSE4_1 __SSE4_1__

#endif

#if NDEBUG

#define CHECK_SIZE_FAIL_(n) __builtin_unreachable()
#define CHECK_ALIGN_(p) p = __builtin_assume_aligned(p, UFXR_ALIGN)

#else

noreturn void ufxr_check_size_fail(size_t n);
noreturn void ufxr_check_align_fail(const void *ptr);
#define CHECK_SIZE_FAIL_(n) ufxr_check_size_fail(n)
#define CHECK_ALIGN_(p)                         \
    if (((uintptr_t)p & (UFXR_ALIGN - 1)) != 0) \
        ufxr_check_align_fail(p);               \
    p = __builtin_assume_aligned(p, UFXR_ALIGN)

#endif

#define CHECK_SIZE_(n)            \
    if ((n) & (UFXR_QUANTUM - 1)) \
    CHECK_SIZE_FAIL_(n)
#define CHECK2(n, x1, x2) \
    do {                  \
        CHECK_SIZE_(n);   \
        CHECK_ALIGN_(x1); \
        CHECK_ALIGN_(x2); \
    } while (0)
