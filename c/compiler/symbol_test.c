#include "c/compiler/symbol.h"
#include "c/compiler/testutil.h"

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct item {
    int32_t index;
    const char *text;
};

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;
    static struct item ITEMS[] = {
        {1, "abcdefghijklmnopqrstuvwxyz"},
        {2, "sym1"},
        {3, "sym2"},
        {2, "sym1"},
        {2, "SYM1"},
        {4, "A"},
        {4, "a"},
        {5, "symbol3"},
        {6, "SYMBOL4"},
        {7, "symbol5"},
        {8, "sym6"},
        {9, "sym7"},
        {10, "sym8"},
        {11, "sym9"},
        {2, "syM1"},
    };
    bool success = true;
    struct ufxr_symtab tab = {};
    size_t n = sizeof(ITEMS) / sizeof(*ITEMS);
    for (size_t i = 0; i < n; i++) {
        const char *text = ITEMS[i].text;
        size_t len = strlen(text);
        int32_t result = ufxr_symtab_add(&tab, text, len);
        if (result < 0) {
            fprintf(stderr, "%zu: add(%s): error %d\n", i, quote_mem(text, len),
                    result);
            temp_free();
            success = false;
            break;
        }
        int32_t expect = ITEMS[i].index;
        if (result != expect) {
            fprintf(stderr, "%zu: add(%s): got %d, expect %d\n", i,
                    quote_mem(text, len), result, expect);
            temp_free();
            success = false;
        }
    }
    free(tab.arr);
    if (!success) {
        fputs("FAILED\n", stderr);
        return 1;
    }
    return 0;
}
