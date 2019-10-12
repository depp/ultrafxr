#include "error.h"
#include "testutil.h"

#include <stdbool.h>
#include <stdio.h>
#include <string.h>

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    bool success = true;
    const char *got, *expect;

    got = ufxr_errname(ERR_NOMEM);
    expect = "NOMEM";
    if (strcmp(got, expect)) {
        fprintf(stderr, "ufxr_errname(ERR_NOMEM) = %s, expect %s\n",
                quote_str(got), quote_str(expect));
        success = false;
    }
    temp_free();

    got = ufxr_errtext(ERR_NOMEM);
    expect = "Out of memory.";
    if (strcmp(got, expect)) {
        fprintf(stderr, "ufxr_errtext(ERR_NOMEM) = %s, expect %s\n",
                quote_str(got), quote_str(expect));
        success = false;
    }
    temp_free();

    return success ? 0 : 1;
}
