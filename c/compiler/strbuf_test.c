#include "c/compiler/strbuf.h"
#include "c/compiler/error.h"
#include "c/compiler/testutil.h"

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static bool test_buf(const char *testname, struct ufxr_strbuf *restrict buf,
                     const char *expect) {
    size_t len = strlen(expect);
    if ((size_t)(buf->end - buf->start) != len ||
        memcmp(buf->start, expect, len) != 0) {
        fprintf(stderr, "%s: got %s, expect %s\n", testname,
                quote_mem(buf->start, buf->end - buf->start),
                quote_mem(expect, len));
        temp_free();
        return false;
    }
    return true;
}

static bool test_putc(void) {
    bool success = true;
    struct ufxr_strbuf buf = {};
    static const char str[] =
        "hello 0123456789 "
        "abcdefghijklmnopqrstuvwxyz "
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const int len = strlen(str);
    for (int i = 0; i < len; i++) {
        int r = ufxr_strbuf_putc(&buf, str[i]);
        if (r != 0) {
            fprintf(stderr, "putc('%c'): %s\n", str[i], ufxr_errtext(r));
            success = false;
            break;
        }
    }
    if (success && !test_buf("putc", &buf, str)) {
        success = false;
    }
    free(buf.start);
    return success;
}

static bool test_puts(void) {
    bool success = true;
    struct ufxr_strbuf buf = {};
    const int len = 5;
    static const char *strs[] = {
        "",
        "abcdef",
        "123",
        "q",
        "0123456789abcdefghijklmnopqrstuvwxyz"
        "0123456789abcdefghijklmnopqrstuvwxyz",
    };
    static const char expect[] =
        "abcdef"
        "123"
        "q"
        "0123456789abcdefghijklmnopqrstuvwxyz"
        "0123456789abcdefghijklmnopqrstuvwxyz";
    for (int i = 0; i < len; i++) {
        int r = ufxr_strbuf_puts(&buf, strs[i]);
        if (r != 0) {
            fprintf(stderr, "puts(\"%s\"): %s\n", strs[i], ufxr_errtext(r));
            success = false;
            break;
        }
    }
    if (success && !test_buf("puts", &buf, expect)) {
        success = false;
    }
    free(buf.start);
    return success;
}

struct u64case {
    uint64_t val;
    char str[20];
};

#define T(x) \
    { x, #x }
static const struct u64case U64_CASES[] = {
    T(0),    T(1),     T(9),         T(10),         T(123),
    T(4321), T(98765), T(987654321), T(1234567890), T(9223372036854775807),
};

static bool test_putu64(void) {
    bool success = true;
    struct ufxr_strbuf buf = {};
    const int n = sizeof(U64_CASES) / sizeof(*U64_CASES);
    for (size_t i = 0; i < n; i++) {
        buf.end = buf.start;
        int r = ufxr_strbuf_putu64(&buf, U64_CASES[i].val);
        if (r != 0) {
            fprintf(stderr, "putu64(%llu): %s\n", U64_CASES[i].val,
                    ufxr_errtext(r));
            success = false;
        } else if (!test_buf("putu64", &buf, U64_CASES[i].str)) {
            success = false;
        }
    }
    free(buf.start);
    return success;
}

struct fmtcase {
    const char *msg;
    const char *out;
    int paramcount;
    const struct ufxr_fmtparam *params;
};

static const struct fmtcase FMT_CASES[] = {
    {
        "hello, world",
        "hello, world",
        0,
        NULL,
    },
    {
        "$1",
        "99",
        1,
        (const struct ufxr_fmtparam[]){
            {.type = FPARAM_U64, .value = {.u64 = 99}},
        },
    },
    {
        "Parameter is $2, parameter is $1",
        "Parameter is 100, parameter is 42",
        2,
        (const struct ufxr_fmtparam[]){
            {.type = FPARAM_U64, .value = {.u64 = 42}},
            {.type = FPARAM_U64, .value = {.u64 = 100}},
        },
    },
    {
        "fmt $1",
        "fmt $(missing)",
        0,
        NULL,
    },
    {
        "fmt $",
        "fmt $(badformat)",
        0,
        NULL,
    },
    {
        "inval $q",
        "inval $(badformat)",
        0,
        NULL,
    },
    {
        "p $1 q",
        "p $(badtype) q",
        1,
        (const struct ufxr_fmtparam[]){
            {.type = -1},
        },
    },
};

static bool test_format(void) {
    bool success = true;
    struct ufxr_strbuf buf = {};
    size_t n = sizeof(FMT_CASES) / sizeof(*FMT_CASES);
    for (size_t i = 0; i < n; i++) {
        buf.end = buf.start;
        int r =
            ufxr_strbuf_fmtmem(&buf, FMT_CASES[i].msg, strlen(FMT_CASES[i].msg),
                               FMT_CASES[i].paramcount, FMT_CASES[i].params);
        if (r != 0) {
            fprintf(stderr, "fmtmem %zu: %s", i, ufxr_errtext(r));
            success = false;
        } else if (!test_buf("fmtmem", &buf, FMT_CASES[i].out)) {
            success = false;
        }
    }
    free(buf.start);
    return success;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;
    bool success = true;
    if (!test_putc()) {
        success = false;
    }
    if (!test_puts()) {
        success = false;
    }
    if (!test_putu64()) {
        success = false;
    }
    if (!test_format()) {
        success = false;
    }
    if (!success) {
        fputs("FAILED\n", stderr);
        return 1;
    }
    return 0;
}
