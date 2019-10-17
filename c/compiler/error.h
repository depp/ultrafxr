// error.h - Error handling.
#pragma once

// Error codes returned by functions in this library.
enum {
    // No error.
    ERR_OK,
    // Operation canceled.
    ERR_CANCELED,
    // Out of memory.
    ERR_NOMEM,
    // Source text too large.
    ERR_LARGETEXT,
    // Invalid argument.
    ERR_INVAL_ARG,
};

// Return the name of the error code.
//
// ufxr_errname(ERR_LARGETEXT) = "LARGETEXT".
const char *ufxr_errname(int code);

// Return the textual description of the error code. Human readable.
const char *ufxr_errtext(int code);
