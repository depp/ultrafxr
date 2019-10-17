#include "c/compiler/message_code.h"

#include "c/compiler/error.h"
#include "c/compiler/message.h"
#include "c/compiler/strbuf.h"

#include <assert.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

enum {
    MAXLINES = 2,
};

// Split a message into its component lines.
static int ufxr_msgsplit(struct ufxr_line *restrict lines, const char *text) {
    const char *ptr = text;
    for (int i = 0; i < MAXLINES; i++) {
        const char *bptr = strchr(ptr, '\n');
        if (bptr == NULL) {
            lines[i] = (struct ufxr_line){
                .text = ptr,
                .length = strlen(ptr),
            };
            return i + 1;
        }
        lines[i] = (struct ufxr_line){
            .text = ptr,
            .length = bptr - ptr,
        };
        ptr = bptr + 1;
    }
    return MAXLINES;
}

int ufxr_msgwrite(struct ufxr_msghandler *restrict mh, ufxr_msglevel level,
                  int msgidx, const struct ufxr_fmtparam *restrict params) {
    const char *text = ufxr_msgtext(msgidx);
    if (text == NULL) {
        return ERR_INVAL_ARG;
    }
    bool canceled = false;
    struct ufxr_line msgs[MAXLINES];
    int numlines = ufxr_msgsplit(msgs, text);
    for (int i = 0; i < numlines; i++) {
        if (params[i].type != FPARAM_SRCSPAN) {
            return ERR_INVAL_ARG;
        }
    }
    int pcount = 0;
    while (params[numlines + pcount].type != FPARAM_END) {
        pcount++;
    }
    struct ufxr_strbuf buf = {};
    for (int i = 0; i < numlines; i++) {
        buf.end = buf.start;
        int r = ufxr_strbuf_fmtmem(&buf, msgs[i].text, msgs[i].length, pcount,
                                   params + numlines);
        if (r != 0) {
            return r;
        }
        r = ufxr_strbuf_putc(&buf, 0);
        if (r != 0) {
            return r;
        }
        r = mh->message(mh->ctx, params[i].value.srcspan,
                        i == 0 ? level : MSG_NOTE, i == 0 ? msgidx : 0,
                        buf.start);
        if (r != 0) {
            // If canceled, we continue writing the rest of this message.
            if (r == ERR_CANCELED) {
                canceled = true;
            } else {
                return r;
            }
        }
    }
    free(buf.start);
    return canceled ? ERR_CANCELED : 0;
}
