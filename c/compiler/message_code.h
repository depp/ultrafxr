// message_code.h - Diagnostic message code definitions.
#pragma once

#include "c/compiler/message.h"

struct ufxr_fmtparam;

// Diagnostic messages for errors in input programs. The format here is fixed:
// the comments and enums are parsed to generate tables containing the strings.
// Text in comments are folded into one line, with series of blank lines
// converted to line breaks.
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

// Write a message to the given handler.
//
// The parameter list must start with a number of ufxr_srcspan parameters equal
// to the number of lines in the message. The remaining parameters are used as
// format parameters in the message. The parameter list is terminated with an
// FPARAM_END parameter.
//
// This will invoke the handler one time for each line in the message. The first
// invocation will use the given level and message ID, any following invocations
// will use the NOTE level and an ID of 0.
int ufxr_msgwrite(struct ufxr_msghandler *restrict mh, ufxr_msglevel level,
                  int msgidx, const struct ufxr_fmtparam *restrict params);
