// util.h - Utility functions.
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdnoreturn.h>

// Print an error message and exit the program. If ecode is not zero, append the
// error code to the message.
noreturn void die(int ecode, const char *msg);

// Print an error message and exit the program. If ecode is not zero, append the
// error code to the message.
noreturn void dief(int ecode, const char *fmt, ...)
    __attribute__((format(printf, 2, 3)));

// Print an out of memory error message and exit the program.
noreturn void die_nomem(void);

// Print an error message for an error returned by a function for writing
// output.
noreturn void die_output(void);

// Convert string to integer or abort.
int xatoi(const char *s);

// Allocate memory. Aborts on failure. Returns NULL if size is zero.
void *xmalloc(size_t size) __attribute__((malloc, alloc_size(1)));

// Reallocate memory. Aborts on failure. Frees input and returns NULL if size is
// zero.
void *xrealloc(void *ptr, size_t size)
    __attribute__((warn_unused_result, alloc_size(2)));

// Return a double-quoted version of a string. The result may is allocated as
// if by malloc.
char *quote_bytes(const char *data, size_t len);

// Return a double-quoted version of a string. The result may is allocated as
// if by malloc.
char *quote_str(const char *data);

// Convert a bool to a string.
const char *bool_str(bool x);

// Checked version of fputs which aborts on error.
void xputs(FILE *fp, const char *s);

// Checked version of fprintf which aborts on error.
void xprintf(FILE *fp, const char *fmt, ...)
    __attribute__((format(printf, 2, 3)));

// Checked version of fwrite which aborts on error.
void xwrite(FILE *fp, const char *p, size_t size);

// As sprintf, but abort if the result does not fit in the buffer.
void xsprintf(char *restrict buf, size_t size, const char *fmt, ...)
    __attribute__((format(printf, 3, 4)));

struct data {
    void *ptr;
    size_t size;
    size_t alloc;
};

// Read a file in its entirety. Aborts on error.
void read_file(struct data *restrict data, const char *name);

struct strings {
    char **strings;
    size_t count;
    size_t alloc;
};

// Append a string to a list of strings.
void strings_push(struct strings *restrict strings, char *string);

// Split a string into lines. Newlines are stripped and each line is
// null-terminated. Modifies the input.
void split_lines(struct strings *restrict lines, struct data *restrict data);

// Split row of comma-delimited fields into a list of fields. Modifies the
// input. Always returns at least one field.
void split_csv(struct strings *restrict fields, char *row);

// C comment with notice that a file is automatically generated. Ends with line
// break.
extern const char kNotice[];
