#include "error.h"

#include <stdbool.h>
#include <stdio.h>
#include <string.h>

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    bool success = true;

    const char *name = ufxr_errname(ERR_NOMEM);
    if (strcmp(name, "NOMEM") != 0) {
        fputs("ufxr_errname(ERR_NOMEM) != \"NOMEM\"\n", stderr);
        success = false;
    }
    const char *text = ufxr_errtext(ERR_NOMEM);
    if (strcmp(text, "Out of memory.") != 0) {
        fputs("ufxr_errtext(ERR_NOMEM) != \"Out of memory.\"\n", stderr);
        success = false;
    }

    return success ? 0 : 1;
}
