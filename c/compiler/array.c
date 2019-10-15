// array.c - Dynamic arrays.
#include "c/compiler/array.h"

#include <stdlib.h>

// Expand an array, increasing the capacity.
void *ufxr_array_expand(void *array, uint32_t *restrict alloc, size_t elem_size,
                        uint32_t init_alloc) {
    uint32_t new_alloc;
    if (*alloc == 0) {
        new_alloc = init_alloc;
    } else if (__builtin_umul_overflow(*alloc, 2, &new_alloc)) {
        return NULL;
    }
    void *new_array;
    if (sizeof(size_t) == sizeof(unsigned)) {
        // 32-bit.
        unsigned new_size;
        if (__builtin_umul_overflow(new_alloc, elem_size, &new_size)) {
            return NULL;
        }
        new_array = realloc(array, new_size);
    } else {
        // 64-bit.
        unsigned long long new_size;
        if (__builtin_umulll_overflow(new_alloc, elem_size, &new_size)) {
            return NULL;
        }
        new_array = realloc(array, new_size);
    }
    if (new_array == NULL) {
        return NULL;
    }
    *alloc = new_alloc;
    return new_array;
}
