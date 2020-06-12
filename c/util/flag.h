// flag.h - Command-line flag parsing.
#pragma once

#include <stdbool.h>

// Define a flag with a string value.
void flag_string(const char **value, const char *name, const char *doc);

// Define a flag with an integer value.
void flag_int(int *value, const char *name, const char *doc);

// Define a flag with a boolean value.
void flag_bool(bool *value, const char *name, const char *doc);

// Parse the command-line flags. Modifies the input to contain only positional
// args. Removes argv[0]. Returns the new value of argc.
int flag_parse(char **argv);
