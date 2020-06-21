#if !NDEBUG

#include "c/ops/impl.h"

#include <stdio.h>
#include <stdlib.h>

void ufxr_check_size_fail(size_t n) {
    fprintf(stderr, "Error: invalid UFXR array size: %zu\n", n);
    abort();
}

noreturn void ufxr_check_align_fail(const void *ptr) {
    fprintf(stderr, "Error: invalid URXR array alignment: %p\n", ptr);
    abort();
}

#endif
