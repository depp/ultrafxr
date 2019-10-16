// argparse.c - Argument parsing.
#include "c/compiler/argparse.h"

#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

void ufxr_argparser_init(struct ufxr_argparser *restrict ap, int argc,
                         char **argv) {
    *ap = (struct ufxr_argparser){
        .argptr = argv,
        .argend = argv + argc,
    };
}

static bool TAKES_PARAM[] = {
    [ARG_BARE] = false,
    [ARG_STRING] = true,
};

int ufxr_argparser_next(struct ufxr_argparser *restrict ap,
                        const struct ufxr_argdef *restrict defs) {
    if (ap->argptr == ap->argend) {
        ap->name = NULL;
        ap->val = NULL;
        return ARG_END;
    }
    char *arg = *ap->argptr++;
    if (arg[0] != '-') {
        ap->name = NULL;
        ap->val = arg;
        return ARG_POSITIONAL;
    }
    char *name = arg + 1;
    if (*name == '-') {
        name++;
    }
    ap->name = name;
    char *equal = strchr(name, '=');
    if (equal != NULL) {
        *equal = '\0';
        ap->val = equal + 1;
    } else {
        ap->val = NULL;
    }
    for (int i = 0; defs[i].name != NULL; i++) {
        if (strcmp(name, defs[i].name) == 0) {
            assert(ARG_BARE <= defs[i].type && defs[i].type <= ARG_STRING);
            if (TAKES_PARAM[defs[i].type]) {
                if (ap->val == NULL) {
                    if (ap->argptr == ap->argend) {
                        return ARG_NEEDS_PARAM;
                    }
                    ap->val = *ap->argptr++;
                }
            } else {
                if (ap->val != NULL) {
                    return ARG_UNEXPECTED_PARAM;
                }
            }
            return defs[i].id;
        }
    }
    return ARG_UNKNOWN;
}

char *ufxr_argparser_err(struct ufxr_argparser *restrict ap, int code) {
    char *str;
    int r;
    switch (code) {
    case ARG_UNKNOWN:
        r = asprintf(&str, "unknown option -%s", ap->name);
        break;
    case ARG_NEEDS_PARAM:
        r = asprintf(&str, "option -%s requires a parameter", ap->name);
        break;
    case ARG_UNEXPECTED_PARAM:
        r = asprintf(&str, "option -%s does not take a parameter", ap->name);
        break;
    default:
        r = asprintf(&str, "unknown error code %d", code);
        break;
    }
    if (r < 0) {
        return NULL;
    }
    return str;
}
