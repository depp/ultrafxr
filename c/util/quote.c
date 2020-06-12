// quote.c - String quoting.
#include "c/util/util.h"

#include <string.h>

static const char HEX_DIGIT[16] = {
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
};

char *quote_bytes(const char *data, size_t len) {
    // Worst-case scenario: 4 bytes per character.
    char *buf = xmalloc(len * 4 + 3);
    char *ptr = buf;
    *ptr = '"';
    ptr++;
    for (size_t i = 0; i < len; i++) {
        unsigned c = (unsigned char)data[i];
        if (32 <= c && c <= 126) {
            if (c == '"' || c == '\\')
                *ptr++ = '\\';
            *ptr++ = c;
        } else {
            *ptr++ = '\\';
            switch (c) {
            case '\n':
                *ptr++ = 'n';
                break;
            case '\r':
                *ptr++ = 'r';
                break;
            case '\t':
                *ptr++ = 't';
                break;
            default:
                *ptr++ = 'x';
                *ptr++ = HEX_DIGIT[c >> 4];
                *ptr++ = HEX_DIGIT[c & 15];
                break;
            }
        }
    }
    *ptr++ = '"';
    *ptr = '\0';
    return buf;
}

char *quote_str(const char *data) {
    return quote_bytes(data, strlen(data));
}

const char *bool_str(bool x) {
    return x ? "true" : "false";
}
