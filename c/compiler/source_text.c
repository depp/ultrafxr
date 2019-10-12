// source_text.c - Tools for showing source locations.
#include "c/compiler/source.h"

#include "c/compiler/error.h"

#include <stdlib.h>

static int ufxr_srctext_addbreak(struct ufxr_srctext *restrict st,
                                 uint32_t pos) {
    if (st->breakcount >= st->breakalloc) {
        uint32_t nalloc = st->breakalloc == 0 ? 16 : st->breakalloc * 2;
        size_t size = nalloc * sizeof(uint32_t);
        if (size == 0) {
            return ERR_NOMEM;
        }
        uint32_t *breaks = realloc(st->breaks, size);
        if (breaks == NULL) {
            return ERR_NOMEM;
        }
        st->breaks = breaks;
        st->breakalloc = nalloc;
    }
    st->breaks[st->breakcount] = pos;
    st->breakcount++;
    return 0;
}

int ufxr_srctext_settext(struct ufxr_srctext *restrict st, const char *text,
                         size_t textlen) {
    if (textlen > (uint32_t)-1) {
        return ERR_LARGETEXT;
    }
    st->text = text;
    st->breakcount = 0;
    int r = ufxr_srctext_addbreak(st, 0);
    if (r != 0) {
        return r;
    }
    uint32_t end = textlen;
    for (uint32_t pos = 0; pos < end;) {
        unsigned char c = text[pos];
        pos++;
        if (c == '\n') {
            r = ufxr_srctext_addbreak(st, pos);
            if (r != 0) {
                return r;
            }
        } else if (c == '\r') {
            if (pos < end && text[pos] == '\n') {
                pos++;
            }
            r = ufxr_srctext_addbreak(st, pos);
            if (r != 0) {
                return r;
            }
        }
    }
    if (st->breaks[st->breakcount - 1] != end) {
        return ufxr_srctext_addbreak(st, end);
    }
    return 0;
}

struct ufxr_line ufxr_srctext_getline(struct ufxr_srctext *restrict st,
                                      int lineno) {
    if (lineno <= 0 || st->breakcount <= (uint32_t)lineno) {
        return (struct ufxr_line){};
    }
    uint32_t start = st->breaks[lineno - 1];
    uint32_t end = st->breaks[lineno];
    const char *text = st->text + start;
    uint32_t length = end - start;
    while (length > 0 &&
           (text[length - 1] == '\n' || text[length - 1] == '\r')) {
        length--;
    }
    return (struct ufxr_line){
        .text = text,
        .length = length,
    };
}

struct ufxr_srcpos ufxr_srctext_getpos(struct ufxr_srctext *restrict st,
                                       uint32_t offset) {
    if (st->breakcount == 0) {
        return (struct ufxr_srcpos){};
    }
    uint32_t left = 0, right = st->breakcount - 1;
    while (right - left > 1) {
        uint32_t middle = left + (right - left) / 2;
        uint32_t val = st->breaks[middle];
        if (val < offset) {
            left = middle;
        } else if (val > offset) {
            right = middle;
        } else {
            left = middle;
            break;
        }
    }
    return (struct ufxr_srcpos){
        .lineno = (int)left + 1,
        .colno = (int)(offset - st->breaks[left]),
    };
}
