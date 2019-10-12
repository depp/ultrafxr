#include "c/compiler/error.h"
#include "c/compiler/source.h"
#include "c/compiler/testutil.h"

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct testcase {
    const char *name;
    const char *text;
    const struct ufxr_srcpos *restrict posarr;
    const char *const *restrict lines;
};

const struct testcase TEST_CASES[] = {
    {
        .name = "simple",
        .text = "abc\n\ndef\n",
        .posarr =
            (const struct ufxr_srcpos[]){
                // clang-format off
                {1, 0}, {1, 1}, {1, 2}, {1, 3},
                {2, 0},
                {3, 0}, {3, 1}, {3, 2}, {3, 3}, {3, 4},
                // clang-format off
            },
        .lines = (const char *const[]){"abc", "", "def", 0},
    },
    {
        .name = "missing_break",
        .text = "line",
        .posarr =
            (const struct ufxr_srcpos[]){
                // clang-format off
                {1, 0}, {1, 1}, {1, 2}, {1, 3}, {1, 4},
                // clang-format on
            },
        .lines = (const char *const[]){"line", 0},
    },
    {
        .name = "crlf",
        .text = "a\r\nb\r\n",
        .posarr =
            (const struct ufxr_srcpos[]){
                // clang-format off
                {1, 0}, {1, 1}, {1, 2},
                {2, 0}, {2, 1}, {2, 2}, {2, 3},
                // clang-format on
            },
        .lines = (const char *const[]){"a", "b", 0},
    },
    {
        .name = "cr",
        .text = "a\rb\r",
        .posarr =
            (const struct ufxr_srcpos[]){
                // clang-format off
                {1, 0}, {1, 1},
                {2, 0}, {2, 1}, {2, 2},
                // clang-format on
            },
        .lines = (const char *const[]){"a", "b", 0},
    },
};

static bool run_test(const struct testcase *restrict t) {
    struct ufxr_srctext text = {};
    size_t len = strlen(t->text);
    int r = ufxr_srctext_settext(&text, t->text, len);
    if (r) {
        fprintf(stderr, "%s: settext: %s\n", t->name, ufxr_errtext(r));
        return false;
    }
    bool success = true;
    for (uint32_t i = 0; i <= len; i++) {
        struct ufxr_srcpos pos = ufxr_srctext_getpos(&text, i);
        struct ufxr_srcpos expect = t->posarr[i];
        if (pos.lineno != expect.lineno || pos.colno != expect.colno) {
            fprintf(stderr, "%s: getpos(%u): got (%d:%d), expect (%d:%d)\n",
                    t->name, i, pos.lineno, pos.colno, expect.lineno,
                    expect.colno);
            success = false;
        }
    }
    int lineno = 0;
    struct ufxr_line line = ufxr_srctext_getline(&text, lineno);
    if (line.text != NULL) {
        fprintf(stderr, "%s: getline(%d): got %s, expect NULL\n", t->name,
                lineno, quote_mem(line.text, line.length));
        success = false;
    }
    for (lineno = 1; t->lines[lineno - 1]; lineno++) {
        const char *expect = t->lines[lineno - 1];
        size_t expectlen = strlen(expect);
        line = ufxr_srctext_getline(&text, lineno);
        if (line.length != expectlen ||
            memcmp(line.text, expect, expectlen) != 0) {
            fprintf(stderr, "%s: getline(%d): got %s, expect %s\n", t->name,
                    lineno, quote_mem(line.text, line.length),
                    quote_mem(expect, expectlen));
            temp_free();
            success = false;
        }
    }
    line = ufxr_srctext_getline(&text, lineno);
    if (line.text != NULL) {
        fprintf(stderr, "%s: getline(%d): got %s, expect NULL\n", t->name,
                lineno, quote_mem(line.text, line.length));
        success = false;
    }
    free(text.breaks);
    return success;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;
    bool success = true;
    for (size_t i = 0; i < sizeof(TEST_CASES) / sizeof(*TEST_CASES); i++) {
        bool test_pass = run_test(&TEST_CASES[i]);
        if (!test_pass) {
            success = false;
        }
    }
    if (!success) {
        fputs("FAILED\n", stderr);
        return 1;
    }
    return 0;
}
