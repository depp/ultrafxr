// sexpr.h - S-Expression parsing.
#pragma once

#include <stdint.h>

// Token types.
typedef enum {
    STOK_END,   // End of input.
    STOK_ERROR, // Invalid character.
    STOK_COMMENT,
    STOK_SYMBOL,
    STOK_NUMBER,
    STOK_PAREN_OPEN,
    STOK_PAREN_CLOSE,
} ufxr_stokentype;

// A token in an s-expression.
struct ufxr_stoken {
    ufxr_stokentype type;
    const char *text;   // Pointer to token text.
    uint32_t length;    // Token length, in bytes.
    uint32_t sourcepos; // Token source offset, in bytes.
};

// State of a token stream.
struct ufxr_stoken_state {
    const char *pos;
    const char *start;
    const char *end;
};

// Return the next token in the stream.
struct ufxr_stoken ufxr_stoken_next(struct ufxr_stoken_state *restrict state);
