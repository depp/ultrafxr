#include "c/compiler/error.h"
#include "c/compiler/message_code.h"
#include "c/compiler/strbuf.h"
#include "c/compiler/testutil.h"

#include <stdbool.h>
#include <stdio.h>
#include <string.h>

struct ctx {
    bool success;
    int msgidx;
    ufxr_msglevel level;
    int pos;
    int max;
    struct ufxr_srcspan locs[2];
    const char *msgs[2];
};

static int message(void *ctx, struct ufxr_srcspan loc, ufxr_msglevel level,
                   int msgidx, const char *msg) {
    struct ctx *restrict c = ctx;
    if (c->pos == c->max) {
        fprintf(stderr, "unexpected message: %s\n", quote_str(msg));
        c->success = false;
        return 0;
    }
    struct ufxr_srcspan eloc = c->locs[c->pos];
    const char *emsg = c->msgs[c->pos];
    int emsgidx = c->pos == 0 ? c->msgidx : 0;
    ufxr_msglevel elevel = c->pos == 0 ? c->level : MSG_NOTE;
    if (eloc.start != loc.start || eloc.end != loc.end ||
        strcmp(emsg, msg) != 0 || emsgidx != msgidx || elevel != level) {
        fprintf(stderr,
                "incorrect message:\n"
                "  loc = %u:%u, expect %u:%u\n"
                "  level = %d, expect %d\n"
                "  msgidx = %d, expect %d\n"
                "  msg = %s, expect %s\n",
                loc.start, loc.end, eloc.start, eloc.end, (int)level,
                (int)elevel, msgidx, emsgidx, quote_str(msg), quote_str(emsg));
        c->success = false;
    }
    c->pos++;
    return 0;
}

static bool test_result(const struct ctx *restrict ctx, int r) {
    bool success = true;
    if (r != 0) {
        fprintf(stderr, "msgwrite: %s\n", ufxr_errtext(r));
        success = false;
    }
    if (ctx->pos != ctx->max) {
        fprintf(stderr, "got %d messages, expected %d\n", ctx->pos, ctx->max);
        success = false;
    }
    if (!ctx->success) {
        success = false;
    }
    return success;
}

static bool test1(void) {
    struct ctx ctx = {
        .success = true,
        .msgidx = MSG_FILE_LONG,
        .level = MSG_ERROR,
        .max = 1,
        .locs = {{1, 2}},
        .msgs = {"Source file is too large: file is 100 bytes long, "
                 "but the maximum length is 99 bytes."},
    };
    struct ufxr_msghandler hdl = {
        .ctx = &ctx,
        .message = message,
    };
    int r = ufxr_msgwrite(
        &hdl, MSG_ERROR, MSG_FILE_LONG,
        (const struct ufxr_fmtparam[]){
            {.type = FPARAM_SRCSPAN, .value = {.srcspan = {1, 2}}},
            {.type = FPARAM_U64, .value = {.u64 = 100}},
            {.type = FPARAM_U64, .value = {.u64 = 99}},
            {.type = FPARAM_END},
        });
    return test_result(&ctx, r);
}

static bool test2(void) {
    struct ctx ctx = (struct ctx){
        .success = true,
        .msgidx = MSG_UNCLOSED_PAREN,
        .level = MSG_WARNING,
        .max = 2,
        .locs = {{3, 4}, {5, 6}},
        .msgs = {"Missing closing paren ')'.",
                 "To match opening paren '(' here."},
    };
    struct ufxr_msghandler hdl = {
        .ctx = &ctx,
        .message = message,
    };
    int r = ufxr_msgwrite(
        &hdl, MSG_WARNING, MSG_UNCLOSED_PAREN,
        (const struct ufxr_fmtparam[]){
            {.type = FPARAM_SRCSPAN, .value = {.srcspan = {3, 4}}},
            {.type = FPARAM_SRCSPAN, .value = {.srcspan = {5, 6}}},
            {.type = FPARAM_END},
        });
    return test_result(&ctx, r);
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;
    bool success = true;
    if (!test1()) {
        success = false;
    }
    if (!test2()) {
        success = false;
    }
    if (!success) {
        fputs("FAILED\n", stderr);
        return 1;
    }
    return 0;
}
