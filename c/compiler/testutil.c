#include "testutil.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void *xalloc(size_t size) {
    if (size == 0) {
        return NULL;
    }
    void *ptr = malloc(size);
    if (ptr == NULL) {
        fprintf(stderr, "error: malloc failed on %zu bytes\n", size);
        abort();
    }
    return ptr;
}

static void *xrealloc(void *ptr, size_t size) {
    if (size == 0) {
        free(ptr);
        return NULL;
    }
    void *out = realloc(ptr, size);
    if (out == NULL) {
        fprintf(stderr, "error: realloc failed on %zu bytes\n", size);
        abort();
    }
    return out;
}

static struct {
    void **arr;
    size_t count;
    size_t alloc;
} temp_objs;

void *temp_alloc(size_t size) {
    if (size == 0) {
        return NULL;
    }
    if (temp_objs.count >= temp_objs.alloc) {
        size_t nalloc = temp_objs.alloc == 0 ? 4 : temp_objs.alloc * 2;
        void **narr = xrealloc(temp_objs.arr, sizeof(*narr) * nalloc);
        for (void **pos = narr + temp_objs.alloc, **end = narr + nalloc;
             pos != end; pos++) {
            *pos = NULL;
        }
        temp_objs.arr = narr;
        temp_objs.alloc = nalloc;
    }
    void *ptr = xalloc(size);
    temp_objs.arr[temp_objs.count] = ptr;
    temp_objs.count++;
    return ptr;
}

void temp_free(void) {
    for (void **pos = temp_objs.arr, **end = pos + temp_objs.count; pos != end;
         pos++) {
        free(*pos);
        *pos = NULL;
    }
}

const char *quote_str(const char *text) {
    return quote_mem(text, strlen(text));
}

static const char XDIGIT[16] = {'0', '1', '2', '3', '4', '5', '6', '7',
                                '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'};

static void put_hex(char *ptr, unsigned char c) {
    ptr[0] = '\\';
    ptr[1] = 'x';
    ptr[2] = XDIGIT[c >> 4];
    ptr[3] = XDIGIT[c & 15];
}

const char *quote_mem(const char *text, size_t len) {
    if (len == 0) {
        return "\"\"";
    }
    char *buf = temp_alloc(len * 4 + 3);
    char *out = buf;
    *out++ = '"';
    for (size_t i = 0; i < len; i++) {
        unsigned char c = text[i];
        if (c < 32 || c > 126) {
            unsigned char e = 0;
            switch (c) {
            case '\n':
                e = 'n';
                break;
            case '\r':
                e = 'r';
                break;
            case '\t':
                e = 't';
                break;
            }
            if (e == 0) {
                put_hex(out, c);
                out += 4;
            } else {
                out[0] = '\\';
                out[1] = e;
                out += 2;
            }
        } else if (c == '"' || c == '\\') {
            out[0] = '\\';
            out[1] = c;
            out += 2;
        } else {
            *out = c;
            out++;
        }
    }
    *out++ = '"';
    *out = '\0';
    return buf;
}
