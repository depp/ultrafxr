// strbuf.h - String buffers for building strings.
#include "c/compiler/strbuf.h"

#include "c/compiler/error.h"

#include <stdlib.h>
#include <string.h>

// Round a size_t value up to the smallest power of two at least as large as the
// input. Return 0 if this is not possible because the input is too large.
static size_t round_up_pow2_size(size_t x) {
    x -= 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    // If we write x>>32, we invoke undefined behavior on 32-bit systems.
    x |= (x >> 16) >> 16;
    x += 1;
    return x;
}

static int ufxr_strbuf_reserve(struct ufxr_strbuf *restrict buf, size_t amt) {
    size_t rem = buf->alloc - buf->end;
    if (amt <= rem) {
        return 0;
    }
    size_t size = buf->end - buf->start;
    if (amt > (size_t)-1 - size) {
        return ERR_NOMEM;
    }
    size_t new_alloc = round_up_pow2_size(size + amt);
    if (new_alloc == 0) {
        return ERR_NOMEM;
    }
    char *new_buf = realloc(buf->start, new_alloc);
    if (new_buf == NULL) {
        return ERR_NOMEM;
    }
    *buf = (struct ufxr_strbuf){
        .start = new_buf,
        .end = new_buf + size,
        .alloc = new_buf + new_alloc,
    };
    return 0;
}

int ufxr_strbuf_putc(struct ufxr_strbuf *restrict buf, unsigned char c) {
    int r = ufxr_strbuf_reserve(buf, 1);
    if (r != 0) {
        return r;
    }
    *buf->end++ = c;
    return 0;
}

int ufxr_strbuf_putmem(struct ufxr_strbuf *restrict buf, const void *ptr,
                       size_t len) {
    int r = ufxr_strbuf_reserve(buf, len);
    if (r != 0) {
        return r;
    }
    memcpy(buf->end, ptr, len);
    buf->end += len;
    return 0;
}

int ufxr_strbuf_puts(struct ufxr_strbuf *restrict buf, const char *str) {
    return ufxr_strbuf_putmem(buf, str, strlen(str));
}

int ufxr_strbuf_putu64(struct ufxr_strbuf *restrict buf, uint64_t val) {
    char tmp[20];
    char *ptr = tmp + sizeof(tmp);
    uint64_t x = val;
    for (int i = 0; i < 5; i++) {
        uint32_t q = x % 10000;
        x /= 10000;
        uint32_t h0 = q / 100;
        uint32_t h1 = q % 100;
        ptr -= 4;
        ptr[3] = '0' + (h1 % 10);
        ptr[2] = '0' + (h1 / 10);
        ptr[1] = '0' + (h0 % 10);
        ptr[0] = '0' + (h0 / 10);
    }
    ptr = tmp + 1;
    while (ptr != tmp + sizeof(tmp) - 1 && *ptr == '0') {
        ptr++;
    }
    return ufxr_strbuf_putmem(buf, ptr, tmp + sizeof(tmp) - ptr);
}

int ufxr_strbuf_fmtmem(struct ufxr_strbuf *restrict buf, const char *msg,
                       size_t msglen, int paramcount,
                       const struct ufxr_fmtparam *restrict params) {
    const char *pos = msg;
    const char *end = msg + msglen;
    while (pos != end) {
        const char *arg = memchr(pos, '$', end - pos);
        if (arg == NULL) {
            return ufxr_strbuf_putmem(buf, pos, end - pos);
        }
        if (pos != arg) {
            int r = ufxr_strbuf_putmem(buf, pos, arg - pos);
            if (r != 0) {
                return r;
            }
        }
        if (arg + 1 == end) {
            // Bad format string: $ at end of string.
            return ufxr_strbuf_puts(buf, "$(badformat)");
        }
        unsigned c = *(arg + 1);
        pos = arg + 2;
        if ('1' <= c && c <= '9') {
            int idx = c - '1';
            if (idx >= paramcount) {
                // Missing value for a $param.
                int r = ufxr_strbuf_puts(buf, "$(missing)");
                if (r != 0) {
                    return r;
                }
            } else {
                int r;
                switch (params[idx].type) {
                default:
                    r = ufxr_strbuf_puts(buf, "$(badtype)");
                    break;
                case FPARAM_U64:
                    r = ufxr_strbuf_putu64(buf, params[idx].value.u64);
                    break;
                }
                if (r != 0) {
                    return r;
                }
            }
        } else if (c == '$') {
            int r = ufxr_strbuf_putc(buf, c);
            if (r != 0) {
                return r;
            }
        } else {
            int r = ufxr_strbuf_puts(buf, "$(badformat)");
            if (r != 0) {
                return r;
            }
        }
    }
    return 0;
}
