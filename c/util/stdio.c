// stdio.c - Versions of stdio.h functions which abort on error.
#include "c/util/util.h"

#include <stdarg.h>

void xputs(FILE *fp, const char *s) {
    if (fputs(s, fp) < 0) {
        die_output();
    }
}

void xprintf(FILE *fp, const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    if (vfprintf(fp, fmt, ap) < 0) {
        die_output();
    }
    va_end(ap);
}

void xwrite(FILE *fp, const char *p, size_t size) {
    size_t r = fwrite(p, 1, size, fp);
    if (r != size) {
        die_output();
    }
}

void xsprintf(char *restrict buf, size_t size, const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    int r = vsnprintf(buf, size, fmt, ap);
    va_end(ap);
    if ((size_t)r >= size) {
        die(0, "xsprintf buffer too small");
    }
}
