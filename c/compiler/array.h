// array.h - Dynamic arrays.
#pragma once

#include <stddef.h>
#include <stdint.h>

// Expand an array, increasing the capacity. Modifies the value of alloc and
// returns a non-NULL new value for array, if successful. Otherwise, returns
// NULL, and the original array is unmodified.
//
// Essentially, this is a wrapper around realloc.
void *ufxr_array_expand(void *array, uint32_t *restrict alloc, size_t elem_size,
                        uint32_t init_alloc);
