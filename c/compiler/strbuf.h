// strbuf.h - String buffers for building strings.
#pragma once

#include "c/compiler/source.h"

#include <stddef.h>
#include <stdint.h>

// A buffer for building strings.
//
// These fields are either all NULL or all point to, or past, the same object.
// In these cases, start <= end, and end <= alloc.
struct ufxr_strbuf {
    char *start; // Pointer to buffer start.
    char *end;   // Pointer past data written to buffer.
    char *alloc; // Pointer past allocated buffer size.
};

// Append a character to a string buffer. Returns an error code.
int ufxr_strbuf_putc(struct ufxr_strbuf *restrict buf, unsigned char c);

// Append an array of characters to a string buffer. Returns an error code.
int ufxr_strbuf_putmem(struct ufxr_strbuf *restrict buf, const void *ptr,
                       size_t len);

// Append a nul-terminated string to a string buffer. Returns an error code.
int ufxr_strbuf_puts(struct ufxr_strbuf *restrict buf, const char *str);

// Append a 64-bit unsigned integer, in decimal, to the buffer.
int ufxr_strbuf_putu64(struct ufxr_strbuf *restrict buf, uint64_t val);

// String formatting parameter types.
typedef enum {
    FPARAM_END, // End of format parameters, if count is not specified.
    FPARAM_U64, // A uint64_t value in decimal format.
} ufxr_fmtparamtype;

// A parameter for string interpolation.
struct ufxr_fmtparam {
    ufxr_fmtparamtype type;
    union {
        // FPARAM_U64.
        uint64_t u64;
    } value;
};

// Expand a format string and append it to the buffer. The format string may
// contain parameter references, $1..$9, which are replaced by the given
// parameters supplied here (with $1 replaced with params[0], $2 replaced with
// params[1], etc). Returns an error code on failure.
//
// If there are any errors in the format string or parameters, short error
// messages are embedded in the output. This is not considered a failure and the
// function may still return 0. The errors are:
//
// - $(missing): Missing parameter, e.g. parameter is $2 but only 1 parameter
//   was given.
//
// - $(badformat): Invalid format string.
//
// - $(badtype): Invalid parameter type.
int ufxr_strbuf_fmtmem(struct ufxr_strbuf *restrict buf, const char *msg,
                       size_t msglen, int paramcount,
                       const struct ufxr_fmtparam *restrict params);
