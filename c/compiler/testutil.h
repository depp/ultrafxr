#pragma once

#include <stddef.h>

// Allocate memory as from malloc(), and abort the program on failure.
void *temp_alloc(size_t size);

// Free all objects created by temp_alloc;
void temp_free(void);

// Enclose a nul-terminated string in double quotes and escape characters inside
// as in a C string.
const char *quote_str(const char *text);

// Enclose a byte array in double quotes and escape characters inside as in a C
// string.
const char *quote_mem(const char *text, size_t len);
