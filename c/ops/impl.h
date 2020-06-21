// c/ops/impl.h - Definitions for operator implementations.
#pragma once
#include "c/ops/ops.h"

#if !USE_SCALAR

#define USE_SSE2 __SSE2__
#define USE_SSE4_1 __SSE4_1__

#endif
