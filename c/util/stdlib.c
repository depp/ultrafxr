// stdlib.c - Versions of stdlib.h functions which abort on error.
#include "c/util/util.h"

#include <errno.h>
#include <limits.h>
#include <stdlib.h>

int xatoi(const char *s) {
    errno = 0;
    char *end;
    long x = strtol(s, &end, 10);
    if (*s == '\0' || *end != '\0') {
        dief(0, "invalid integer: %s", quote_str(s));
    }
    int ecode = errno;
    if (x > INT_MAX || x < INT_MIN || ecode == ERANGE) {
        dief(0, "integer out of range: %s", quote_str(s));
    }
    return x;
}

float xatof(const char *s) {
    errno = 0;
    char *end;
    float x = strtof(s, &end);
    if (*s == '\0' || *end != '\0') {
        dief(0, "invalid numebr: %s", quote_str(s));
    }
    int ecode = errno;
    if (ecode == ERANGE) {
        dief(0, "number out of range: %s", quote_str(s));
    }
    return x;
}

double xatod(const char *s) {
    errno = 0;
    char *end;
    double x = strtod(s, &end);
    if (*s == '\0' || *end != '\0') {
        dief(0, "invalid numebr: %s", quote_str(s));
    }
    int ecode = errno;
    if (ecode == ERANGE) {
        dief(0, "number out of range: %s", quote_str(s));
    }
    return x;
}

void *xmalloc(size_t size) {
    if (size == 0) {
        return NULL;
    }
    void *ptr = malloc(size);
    if (ptr == NULL) {
        die_nomem();
    }
    return ptr;
}

void *xrealloc(void *ptr, size_t size) {
    if (size == 0) {
        free(ptr);
        return NULL;
    }
    void *new_ptr = realloc(ptr, size);
    if (new_ptr == NULL) {
        die_nomem();
    }
    return new_ptr;
}
