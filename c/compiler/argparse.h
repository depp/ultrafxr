// argparse.h - Argument parsing.
#pragma once

typedef enum {
    // Bare argument - no parameter (val = NULL).
    ARG_BARE,
    // String argument - parameter stored in val.
    ARG_STRING,
} ufxr_argtype;

struct ufxr_argdef {
    int id;
    const char *name;
    ufxr_argtype type;
};

struct ufxr_argparser {
    char **argptr;
    char **argend;
    char *name;
    char *val;
};

// Special argument IDs. All special IDs are negative, so non-negative IDs can
// be used for options.
enum {
    // No more arguments remain to parse.
    ARG_END = -1,
    // Positional argument encountered. If it takes a parameter, the parameter
    // value is stored in val.
    ARG_POSITIONAL = -2,
    // Error - unknown option. Option name is stored in 'name'.
    ARG_UNKNOWN = -3,
    // Error - option requires a parameter. Option name is stored in 'name'.
    ARG_NEEDS_PARAM = -4,
    // Error - option does not take a parameter. Option name is stored in
    // 'name', option parameter is stored in 'val'.
    ARG_UNEXPECTED_PARAM = -5,
};

// Initialize an argument parser with the given arguments, not including the
// program name.
void ufxr_argparser_init(struct ufxr_argparser *restrict ap, int argc,
                         char **argv);

// Parse the next argument. If the next argument is an option, store its value,
// if any, in val, and return its id. If the next argument is positional, return
// ARG_POSITIONAL and store the positional arg in val. If there are no remaining
// arguments, return ARG_END. Other return values indicate errors.
int ufxr_argparser_next(struct ufxr_argparser *restrict ap,
                        const struct ufxr_argdef *restrict defs);

// Convert an error code to an error message. If successful, the result is a
// pointer to a string allocated by malloc, which must be freed by the caller.
// If out of memory, returns NULL.
char *ufxr_argparser_err(struct ufxr_argparser *restrict ap, int code);
