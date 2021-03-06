// die.c - Functions to abort the program.
#include "c/util/util.h"

#include <errno.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>

enum {
    kStatusError = 1,
    kStatusUsage = 64,
};

void die(int ecode, const char *msg) {
    if (ecode != 0) {
        fprintf(stderr, "Error: %s: %s\n", msg, strerror(ecode));
    } else {
        fprintf(stderr, "Error: %s\n", msg);
    }
    exit(kStatusError);
}

void dief(int ecode, const char *fmt, ...) {
    fputs("Error: ", stderr);
    {
        va_list ap;
        va_start(ap, fmt);
        vfprintf(stderr, fmt, ap);
        va_end(ap);
    }
    if (ecode != 0) {
        fputc(' ', stderr);
        fputs(strerror(ecode), stderr);
    }
    fputc('\n', stderr);
    exit(kStatusError);
}

void die_nomem(void) {
    fputs("Error: out of memory\n", stderr);
    exit(kStatusError);
}

void die_output(void) {
    die(errno, "could not write output");
}

void die_usage(const char *msg) {
    fputs("Error: ", stderr);
    fputs(msg, stderr);
    fputc('\n', stderr);
    exit(kStatusUsage);
}

void die_usagef(const char *fmt, ...) {
    fputs("Error: ", stderr);
    va_list ap;
    va_start(ap, fmt);
    vfprintf(stderr, fmt, ap);
    va_end(ap);
    fputc('\n', stderr);
    exit(kStatusUsage);
}
