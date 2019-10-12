// hash.h - Simple hash function.
#pragma once

#include <stddef.h>
#include <stdint.h>

// Compute the hash of a bytestring.
uint32_t ufxr_hash(const void *data, size_t size) __attribute__((pure));
