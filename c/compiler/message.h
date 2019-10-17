// message.h - Diagnostic messages.
#pragma once

#include "c/compiler/source.h"

// Severity levels for diagnostic messages.
typedef enum {
    MSG_ERROR,
    MSG_WARNING,
    MSG_NOTE,
} ufxr_msglevel;

// Diagnostic message handler.
//
// The message handling function will be called for every diagnostic message
// emitted by the parser or compiler. To abort compilation, this function should
// return ERR_CANCELED.
struct ufxr_msghandler {
    void *ctx;
    int (*message)(void *ctx, struct ufxr_srcspan loc, ufxr_msglevel level,
                   int msgidx, const char *msg);
};
