// message_code.h - Diagnostic message code definitions.
#pragma once

// Diagnostic messages for errors in input programs. The format here is fixed:
// the comments and enums are parsed to generate tables containing the strings.
enum {
    // Source file is too large: file is $1 bytes long, but the maximum length
    // is $2 bytes.
    MSG_FILE_LONG = 1,

    // Symbol is too long: symbol is $1 bytes long, but the maximum length is $2
    // bytes.
    MSG_SYMBOL_LONG = 2,

    // Missing closing paren ')'.
    //
    // To match opening paren '(' here.
    MSG_UNCLOSED_PAREN = 3,

    // Extra closing paren ')'.
    MSG_EXTRA_PAREN = 4,
};

// Return the text of the corresponding diagnostic message, or NULL if no such
// message exists.
const char *ufxr_msgtext(int code);
