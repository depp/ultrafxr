// sexpr_token.c - S-Expression tokenization.
#include "sexpr.h"

#include <stdbool.h>

// Return true if the given character is whitespace.
static bool is_space(unsigned char c) {
    // space, \t, \n, \v, \f, \r
    return c == 32 || (9 <= c && c <= 13);
}

// Return the end of the symbol.
static const char *ufxr_stoken_symbol(const char *start, const char *end) {
    for (const char *pos = start; pos != end; pos++) {
        unsigned char c = *pos;
        switch (c) {
            // clang-format off
            // Lowercase alpha.
        case 'a': case 'b': case 'c': case 'd': case 'e': case 'f': case 'g':
        case 'h': case 'i': case 'j': case 'k': case 'l': case 'm': case 'n':
        case 'o': case 'p': case 'q': case 'r': case 's': case 't': case 'u':
        case 'v': case 'w': case 'x': case 'y': case 'z':
            // Uppercase alpha.
        case 'A': case 'B': case 'C': case 'D': case 'E': case 'F': case 'G':
        case 'H': case 'I': case 'J': case 'K': case 'L': case 'M': case 'N':
        case 'O': case 'P': case 'Q': case 'R': case 'S': case 'T': case 'U':
        case 'V': case 'W': case 'X': case 'Y': case 'Z':
            // Punctuation.
        case '-': case '!': case '$': case '%': case '&': case '*': case '+':
        case '.': case '/': case ':': case '<': case '=': case '>': case '?':
        case '@': case '^': case '_': case '~':
            // Digits.
        case '0': case '1': case '2': case '3': case '4':
        case '5': case '6': case '7': case '8': case '9':
            // clang-format on
            break;

        default:
            return pos;
        }
    }
    return end;
}

// Return the end of the line.
static const char *ufxr_stoken_line(const char *start, const char *end) {
    for (const char *pos = start; pos != end; pos++) {
        unsigned char c = *pos;
        if (c == '\n' || c == '\r') {
            return pos;
        }
    }
    return end;
}

struct ufxr_stoken ufxr_stoken_next(struct ufxr_stoken_state *restrict state) {
    while (state->pos != state->end && is_space(*state->pos)) {
        state->pos++;
    }
    struct ufxr_stoken tok;
    tok.text = state->pos;
    tok.sourcepos = state->pos - state->start;
    if (state->pos == state->end) {
        tok.type = STOK_END;
    } else {
        unsigned char c = *state->pos++;
        switch (c) {
            // clang-format off
            // Lowercase alpha.
        case 'a': case 'b': case 'c': case 'd': case 'e': case 'f': case 'g':
        case 'h': case 'i': case 'j': case 'k': case 'l': case 'm': case 'n':
        case 'o': case 'p': case 'q': case 'r': case 's': case 't': case 'u':
        case 'v': case 'w': case 'x': case 'y': case 'z':
            // Uppercase alpha.
        case 'A': case 'B': case 'C': case 'D': case 'E': case 'F': case 'G':
        case 'H': case 'I': case 'J': case 'K': case 'L': case 'M': case 'N':
        case 'O': case 'P': case 'Q': case 'R': case 'S': case 'T': case 'U':
        case 'V': case 'W': case 'X': case 'Y': case 'Z':
            // Punctuation.
        case '!': case '$': case '%': case '&': case '*': case '/': case ':':
        case '<': case '=': case '>': case '?': case '@': case '^': case '_':
        case '~':
            // clang-format on
            tok.type = STOK_SYMBOL;
            state->pos = ufxr_stoken_symbol(state->pos, state->end);
            break;

        case ';':
            tok.type = STOK_COMMENT;
            state->pos = ufxr_stoken_line(state->pos, state->end);
            break;

        case '-':
        case '+':
            tok.type = STOK_SYMBOL;
            state->pos = ufxr_stoken_symbol(state->pos, state->end);
            if (state->pos - tok.text >= 2) {
                c = *(tok.text + 1);
                if ('0' <= c && c <= '9') {
                    tok.type = STOK_NUMBER;
                } else if (c == '.') {
                    if (state->pos - tok.text >= 3) {
                        c = *(tok.text + 2);
                        if ('0' <= c && c <= '9') {
                            tok.type = STOK_NUMBER;
                        }
                    }
                }
            }
            break;

        case '.':
            tok.type = STOK_SYMBOL;
            state->pos = ufxr_stoken_symbol(state->pos, state->end);
            if (state->pos - tok.text >= 2) {
                c = *(tok.text + 1);
                if ('0' <= c && c <= '9') {
                    tok.type = STOK_NUMBER;
                }
            }
            break;

            // clang-format off
        case '0': case '1': case '2': case '3': case '4':
        case '5': case '6': case '7': case '8': case '9':
            // clang-format on
            tok.type = STOK_NUMBER;
            state->pos = ufxr_stoken_symbol(state->pos, state->end);
            break;

        case '(':
            tok.type = STOK_PAREN_OPEN;
            break;

        case ')':
            tok.type = STOK_PAREN_CLOSE;
            break;

        default:
            tok.type = STOK_ERROR;
            break;
        }
    }
    tok.length = state->pos - tok.text;
    return tok;
}
