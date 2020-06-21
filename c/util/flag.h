// flag.h - Command-line flag parsing.
#pragma once

#include <stdbool.h>
#include <stdnoreturn.h>

// Define a flag with a string value.
void flag_string(const char **value, const char *name, const char *doc);

// Define a flag with an integer value.
void flag_int(int *value, const char *name, const char *doc);

// Define a flag with a boolean value.
void flag_bool(bool *value, const char *name, const char *doc);

// Parse the command-line flags. Modifies the input to contain only positional
// args. Removes argv[0]. Returns the new value of argc.
int flag_parse(int argc, char **argv);

// Print an error message for incorrect usage and exit the program.
noreturn void die_usage(const char *msg);

// Print an error message for incorrect usage and exit the program.
noreturn void die_usagef(const char *fmt, ...)
    __attribute__((format(printf, 1, 2)));
