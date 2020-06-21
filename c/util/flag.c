// flag.c - Command-line flag parsing.
#include "c/util/flag.h"

#include "c/util/util.h"

#include <errno.h>
#include <limits.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>

typedef enum {
    kFlagString,
    kFlagInteger,
    kFlagBoolean,
} flag_type;

#define FLAG_TYPE_COUNT ((int)kFlagBoolean + 1)

struct flag {
    flag_type type;
    void *value;
    const char *name;
    const char *doc;
};

static struct flag *flags;
static size_t flag_count;
static size_t flag_alloc;

static void check_name(const char *name) {
    for (size_t i = 0; i < flag_count; i++) {
        if (!strcmp(name, flags[i].name)) {
            dief(0, "duplicate flag name %s", quote_str(name));
        }
    }
}

static struct flag *flag_new(void) {
    if (flag_count >= flag_alloc) {
        size_t alloc = flag_alloc == 0 ? 4 : flag_alloc * 2;
        if (alloc == 0) {
            die_nomem();
        }
        flags = xrealloc(flags, alloc * sizeof(*flags));
        flag_alloc = alloc;
    }
    struct flag *f = &flags[flag_count];
    flag_count++;
    return f;
}

void flag_string(const char **value, const char *name, const char *doc) {
    check_name(name);
    *flag_new() = (struct flag){
        .type = kFlagString,
        .value = value,
        .name = name,
        .doc = doc,
    };
}

void flag_int(int *value, const char *name, const char *doc) {
    check_name(name);
    *flag_new() = (struct flag){
        .type = kFlagInteger,
        .value = value,
        .name = name,
        .doc = doc,
    };
}

void flag_bool(bool *value, const char *name, const char *doc) {
    check_name(name);
    *flag_new() = (struct flag){
        .type = kFlagBoolean,
        .value = value,
        .name = name,
        .doc = doc,
    };
}

static void parse_string(struct flag *fp, char *arg) {
    char **value = fp->value;
    *value = arg;
}

static void parse_integer(struct flag *fp, char *arg) {
    int *value = fp->value;
    char *end;
    errno = 0;
    long x = strtol(arg, &end, 10);
    if (*arg == '\0' || *end != '\0') {
        dief(0, "invalid value for -%s: got %s, expected an integer", fp->name,
             quote_str(arg));
    }
    int ecode = errno;
    if (x < INT_MIN || x > INT_MAX || ecode == ERANGE) {
        dief(0, "value for -%s is too large: %s", fp->name, quote_str(arg));
    }
    *value = x;
}

static void parse_boolean(struct flag *fp) {
    bool *value = fp->value;
    *value = true;
}

static const bool kFlagNeedsArg[FLAG_TYPE_COUNT] = {
    [kFlagString] = true,
    [kFlagInteger] = true,
};

int flag_parse(int argc, char **argv) {
    (void)argc;
    char **outp = argv, **inp = argv + 1;
    struct flag *fs = flags, *fe = fs + flag_count;
    while (1) {
        char *arg = *inp;
        if (arg == NULL) {
            break;
        }
        inp++;
        if (arg[0] != '-') {
            *outp++ = arg;
            continue;
        }
        char *name;
        if (arg[1] == '-') {
            if (arg[2] == '\0') {
                break;
            }
            name = &arg[2];
        } else {
            name = &arg[1];
        }
        char *eq = strchr(name, '=');
        char *value;
        if (eq == name) {
            dief(0, "invalid flag %s", quote_str(arg));
        } else if (eq == NULL) {
            value = NULL;
        } else {
            value = eq + 1;
            *eq = '\0';
        }
        struct flag *fp = fs;
        while (fp != fe && strcmp(name, fp->name)) {
            fp++;
        }
        if (fp == fe) {
            dief(0, "unknown flag %s", quote_str(arg));
        }
        if (kFlagNeedsArg[fp->type]) {
            if (value == NULL) {
                value = *inp;
                if (value == NULL) {
                    dief(0, "flag -%s requires argument", fp->name);
                }
                inp++;
            }
        } else {
            if (value != NULL) {
                dief(0, "flag -%s does not take an argument", fp->name);
            }
        }
        switch (fp->type) {
        case kFlagString:
            parse_string(fp, value);
            break;
        case kFlagInteger:
            parse_integer(fp, value);
            break;
        case kFlagBoolean:
            parse_boolean(fp);
            break;
        }
    }
    while (1) {
        char *arg = *inp;
        if (arg == NULL) {
            break;
        }
        inp++;
        *outp++ = arg;
    }
    *outp = NULL;
    return outp - argv;
}
