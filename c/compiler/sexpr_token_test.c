#include "sexpr.h"
#include "testutil.h"

#include <stdbool.h>
#include <stdio.h>
#include <string.h>

static const char *STOKENTYPE_NAME[] = {
    // clang-format off
    "end",
    "error",
    "comment",
    "symbol",
    "number",
    "paren_open",
    "paren_close",
    // clang-format on
};

// Return the name of the given token type.
static const char *stokentype_name(ufxr_stokentype type) {
    size_t n = sizeof(STOKENTYPE_NAME) / sizeof(*STOKENTYPE_NAME);
    if (0 <= type && type < n) {
        return STOKENTYPE_NAME[type];
    }
    char buf[32];
    snprintf(buf, sizeof(buf), "unknown(%d)", (int)type);
    size_t len = strlen(buf);
    void *ptr = temp_alloc(len + 1);
    memcpy(ptr, buf, len + 1);
    return ptr;
}

struct testcase {
    const char *text;
    size_t textlen;
    size_t textpos;
    ufxr_stokentype type;
    uint32_t tokstart;
    uint32_t toklen;
};

static bool run_test(const struct testcase *restrict t) {
    struct ufxr_stoken_state state = {
        .pos = t->text + t->textpos,
        .start = t->text,
        .end = t->text + t->textlen,
    };
    struct ufxr_stoken tok = ufxr_stoken_next(&state);
    if (tok.type == t->type && tok.text == t->text + t->tokstart &&
        tok.length == t->toklen && tok.sourcepos == t->tokstart) {
        return true;
    }
    fprintf(stderr, "failed:\n");
    fprintf(stderr, "  input: %s\n", quote_mem(t->text, t->textlen));
    fprintf(stderr, "  type: %s, expect %s\n", stokentype_name(tok.type),
            stokentype_name(t->type));
    fprintf(stderr, "  text: %p, expect %p\n", tok.text, t->text + t->tokstart);
    fprintf(stderr, "  length: %u, expect %u\n", tok.length, t->toklen);
    fprintf(stderr, "  sourcepos: %u, expect %u\n", tok.sourcepos, t->tokstart);
    return false;
}

struct simplecase {
    const char *text;
    ufxr_stokentype type;
};

static const struct simplecase SIMPLE_CASES[] = {
    // clang-format off
    {";comment\n", STOK_COMMENT},
    {";\n", STOK_COMMENT},
    {"symbol ", STOK_SYMBOL},
    {"ABCXYZ ", STOK_SYMBOL},
    {"ZYXCBA ", STOK_SYMBOL},
    {"abcxyz ", STOK_SYMBOL},
    {"zyxcba ", STOK_SYMBOL},
    {"a0123456789 ", STOK_SYMBOL},
    {"s;", STOK_SYMBOL},
    {"s\n", STOK_SYMBOL},
    {"s(", STOK_SYMBOL},
    {"s)", STOK_SYMBOL},
    {". ", STOK_SYMBOL},
    {"- ", STOK_SYMBOL},
    {"+ ", STOK_SYMBOL},
    {"-. ", STOK_SYMBOL},
    {"+. ", STOK_SYMBOL},
    {"0 ", STOK_NUMBER},
    {"987 ", STOK_NUMBER},
    {"5.0abc@@&* ", STOK_NUMBER},
    {"+0 ", STOK_NUMBER},
    {"+555 ", STOK_NUMBER},
    {"-9 ", STOK_NUMBER},
    {".00 ", STOK_NUMBER},
    {".99 ", STOK_NUMBER},
    {".67 ", STOK_NUMBER},
    {"-.0 ", STOK_NUMBER},
    {"+.9 ", STOK_NUMBER},
    {"(a", STOK_PAREN_OPEN},
    {")a", STOK_PAREN_CLOSE},
    {"\x01 ", STOK_ERROR},
    {"\x7f ", STOK_ERROR},
    {"\x80 ", STOK_ERROR},
    {"\xff ", STOK_ERROR},
    // clang-format on
};

// Run through simple tokenization test cases.
static bool simple_tests(void) {
    size_t n = sizeof(SIMPLE_CASES) / sizeof(*SIMPLE_CASES);
    bool success = true;
    for (size_t i = 0; i < n; i++) {
        const struct simplecase *t = &SIMPLE_CASES[i];
        // Test the token by itself, with nothing before or after.
        size_t len = strlen(t->text);
        char *buf = temp_alloc(len - 1);
        memcpy(buf, t->text, len - 1);
        bool test_success = run_test(&(struct testcase){
            .text = buf,
            .textlen = len - 1,
            .textpos = 0,
            .type = t->type,
            .tokstart = 0,
            .toklen = len - 1,
        });
        if (!test_success) {
            success = false;
        }
        temp_free();
        // Test the token with text before and after.
        buf = temp_alloc(len + 2);
        buf[0] = '^';
        buf[1] = ' ';
        memcpy(buf + 2, t->text, len);
        test_success = run_test(&(struct testcase){
            .text = buf,
            .textlen = len + 2,
            .textpos = 1,
            .type = t->type,
            .tokstart = 2,
            .toklen = len - 1,
        });
        if (!test_success) {
            success = false;
        }
        temp_free();
    }
    return success;
}

// Test all punctuation that can appear in symbols.
static bool symbol_tests(void) {
    bool success = true;
    static const char SYM[] = "-!$%&*+./:<=>?@^_~";
    for (size_t i = 0; i < sizeof(SYM) - 1; i++) {
        char c = SYM[i];
        char buf[2] = {c, c};
        bool test_success = run_test(&(struct testcase){
            .text = buf,
            .textlen = 2,
            .textpos = 0,
            .type = STOK_SYMBOL,
            .tokstart = 0,
            .toklen = 2,
        });
        if (!test_success) {
            success = false;
        }
    }
    return success;
}

static struct testcase SPECIAL_TESTS[] = {
    // clang-format off
    {"", 0, 0, STOK_END, 0, 0},
    {"   ", 3, 0, STOK_END, 3, 0},
    {"", 1, 0, STOK_ERROR, 0, 1},
    // clang-format on
};

// Other, special test cases.
static bool special_tests(void) {
    bool success = true;
    size_t n = sizeof(SPECIAL_TESTS) / sizeof(*SPECIAL_TESTS);
    for (size_t i = 0; i < n; i++) {
        bool test_success = run_test(&SPECIAL_TESTS[i]);
        if (!test_success) {
            success = false;
        }
    }
    return success;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;
    bool success = true;
    if (!simple_tests()) {
        success = false;
    }
    if (!symbol_tests()) {
        success = false;
    }
    if (!special_tests()) {
        success = false;
    }
    if (!success) {
        fputs("FAILED\n", stderr);
        return 1;
    }
    return 0;
}
