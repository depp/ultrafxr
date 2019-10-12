// source.h - Source locations and diagnostic handling.
#pragma once

#include <stddef.h>
#include <stdint.h>

// A structure for displaying source code locations to the user. This translates
// byte offsets to line numbers and extracts lines from the text.
struct ufxr_srctext {
    const char *text;
    uint32_t *breaks;
    uint32_t breakcount;
    uint32_t breakalloc;
};

// Set the source text that this structure uses.
int ufxr_srctext_settext(struct ufxr_srctext *restrict st, const char *text,
                         size_t textlen);

// A line in a source file. This does not include the line break.
struct ufxr_line {
    const char *text;
    uint32_t length;
};

// Return the contents of the given line.
struct ufxr_line ufxr_srctext_getline(struct ufxr_srctext *restrict st,
                                      int lineno);

// A translated location in a source file.
struct ufxr_srcpos {
    int lineno; // Line number, starting from 1.
    int colno;  // Column byte offset.
};

// Translate a byte offset to a location in the source file.
struct ufxr_srcpos ufxr_srctext_getpos(struct ufxr_srctext *restrict st,
                                       uint32_t offset);
