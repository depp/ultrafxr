#include "c/compiler/argparse.h"

#include <errno.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char USAGE[] =
    "Error message compiler.\n"
    "\n"
    "Usage: error_gen [option...] <input>\n"
    "\n"
    "Options:\n"
    "  -dump               Dump parsed messages to stdout.\n"
    "  -help               Print command help.\n"
    "  -include=<file.h>   Include file to include from generated sources.\n"
    "  -msg-func=<name>    Use <name> as message lookup function name.\n"
    "  -msg-out=<file.c>   Write message lookup function to <file.h>.\n"
    "  -name-func=<name>   Use <name> as name lookup function name.\n"
    "  -name-out=<file.c>  Write name lookup function to <file.h>.\n";

// Print an error message for bad program usage and exit the program.
static void die_usage(const char *msg) __attribute__((noreturn));
static void die_usage(const char *msg) {
    fprintf(stderr, "Error: %s\n", msg);
    exit(2);
}

// Print a formatted error for bad program usage and exit the program.
static void die_usagef(const char *msg, ...)
    __attribute__((format(printf, 1, 2), noreturn));
static void die_usagef(const char *msg, ...) {
    fputs("Error: ", stderr);
    va_list ap;
    va_start(ap, msg);
    vfprintf(stderr, msg, ap);
    va_end(ap);
    fputc('\n', stderr);
    exit(2);
}

static const char ERR_INPUT[] = "could not read input";

// Print an error message and exit the program. If ecode is not zero, append the
// error code to the message.
static void die(int ecode, const char *msg) __attribute__((noreturn));
static void die(int ecode, const char *msg) {
    if (ecode != 0) {
        fprintf(stderr, "Error: %s: %s\n", msg, strerror(ecode));
    } else {
        fprintf(stderr, "Error: %s\n", msg);
    }
    exit(1);
}

static void die_nomem(void) __attribute__((noreturn));
static void die_nomem(void) {
    fputs("Error: out of memory\n", stderr);
    exit(1);
}

// Print an error message for the given input line number and exit the program.
static void die_input(int lineno, const char *msg) __attribute__((noreturn));
static void die_input(int lineno, const char *msg) {
    fprintf(stderr, "Error: line %d: %s\n", lineno, msg);
    exit(1);
}

// Print an error message for an error returned by a function for writing
// output.
static void die_output(void) __attribute__((noreturn));
static void die_output(void) {
    die(errno, "could not write output");
}

// Return true if the range has the given prefix.
static bool has_prefix(const char *start, const char *end, const char *prefix) {
    size_t n = strlen(prefix);
    return n <= (size_t)(end - start) && memcmp(start, prefix, n) == 0;
}

// Return true if the given character is whitespace.
static bool is_space(unsigned char c) {
    // space, \t, \n, \v, \f, \r
    return c == 32 || (9 <= c && c <= 13);
}

// Return true if the given character is legal in a C identifier.
static bool is_ident(unsigned char c) {
    return ('A' <= c && c <= 'Z') || ('a' <= c && c <= 'z') || c == '_' ||
           ('0' <= c && c <= '9');
}

// Return the start pointer which excludes all leading whitespace.
static const char *trim_start(const char *start, const char *end) {
    const char *pos = start;
    while (pos != end && is_space(*pos)) {
        pos++;
    }
    return pos;
}

// Return the end pointer which excludes all trailing whitespace.
static const char *trim_end(const char *start, const char *end) {
    const char *pos = end;
    while (pos != start && is_space(*(pos - 1))) {
        pos--;
    }
    return pos;
}

// Return a copy of a string and add a nul delimiter.
static char *copystr(const char *start, const char *end) {
    size_t size = end - start;
    char *text = malloc(size + 1);
    if (text == NULL) {
        die_nomem();
    }
    memcpy(text, start, size);
    text[size] = '\0';
    return text;
}

// Arrays of error names and descriptions.
struct errs {
    char **names;
    char **texts;
    size_t count;
    size_t alloc;
};

// Read the given input file and extract all of the error codes.
static struct errs read_input(const char *filename) {
    FILE *fp = fopen(filename, "r");
    if (fp == NULL) {
        die(errno, "could not open input");
    }
    char line[100];
    int lineno = 0;
    char *text = NULL;
    // Scan for start of enums.
    while (true) {
        if (fgets(line, sizeof(line), fp) == NULL) {
            if (ferror(fp)) {
                die(errno, ERR_INPUT);
            }
            die_input(lineno, "could not find start of error codes");
        }
        lineno++;
        if (memcmp(line, "enum {", 6) == 0) {
            break;
        }
    }
    struct errs errs = {};
    // Read error descriptions and names.
    while (true) {
        if (fgets(line, sizeof(line), fp) == NULL) {
            if (ferror(fp)) {
                die(errno, ERR_INPUT);
            }
            die_input(lineno, "could not find end of error codes");
        }
        lineno++;
        const char *start = line;
        const char *end = line + strlen(line);
        start = trim_start(start, end);
        end = trim_end(start, end);
        if (start == end) {
        } else if (has_prefix(start, end, "//")) {
            // Error description.
            start = trim_start(start + 2, end);
            if (start != end) {
                if (text != NULL) {
                    die_input(lineno, "multiple description for same error");
                }
                for (const char *ptr = start; ptr != end; ptr++) {
                    unsigned char c = *ptr;
                    if (c < 32 || c > 126) {
                        die_input(lineno,
                                  "description contains non-ASCII text");
                    }
                }
                text = copystr(start, end);
            }
        } else if (has_prefix(start, end, "ERR_")) {
            // Error name.
            if (text == NULL) {
                die_input(lineno, "error code has no description");
            }
            start += 4;
            end = start;
            while (is_ident(*end)) {
                end++;
            }
            if (start == end) {
                die_input(lineno, "invalid error code name");
            }
            char *name = copystr(start, end);
            if (errs.count >= errs.alloc) {
                errs.alloc = errs.alloc ? errs.alloc * 2 : 8;
                errs.names =
                    realloc(errs.names, errs.alloc * sizeof(errs.names));
                if (errs.names == NULL) {
                    die_nomem();
                }
                errs.texts =
                    realloc(errs.texts, errs.alloc * sizeof(errs.texts));
                if (errs.texts == NULL) {
                    die_nomem();
                }
            }
            errs.names[errs.count] = name;
            errs.texts[errs.count] = text;
            errs.count++;
            text = NULL;
        } else if (has_prefix(start, end, "};")) {
            // End of errors.
            if (text != NULL) {
                die_input(lineno, "expected error code name");
            }
            if (errs.count == 0) {
                die_input(lineno, "no error codes found");
            }
            break;
        } else {
            die_input(lineno, "could not parse error");
        }
    }
    fclose(fp);
    if (errs.count == 0) {
        die(errno, "no messages found in input");
    }
    return errs;
}

static void cputs(FILE *fp, const char *s) {
    if (fputs(s, fp) < 0) {
        die_output();
    }
}

static void cprintf(FILE *fp, const char *fmt, ...)
    __attribute__((format(printf, 2, 3)));
static void cprintf(FILE *fp, const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    if (vfprintf(fp, fmt, ap) < 0) {
        die_output();
    }
    va_end(ap);
}

static void cwrite(FILE *fp, const char *p, size_t size) {
    size_t r = fwrite(p, 1, size, fp);
    if (r != size) {
        die_output();
    }
}

static const char HEXDIGIT[16] = "0123456789abcdef";

// Write a character in a C string to the given buffer, and return the advanced
// pointer.
static char *write_char(char *buf, unsigned char c) {
    char *out = buf;
    if (32 <= c && c <= 126) {
        if (c == '\\' || c == '"') {
            *out++ = '\\';
        }
        *out++ = c;
        return out;
    }
    *out++ = '\\';
    unsigned char e;
    switch (c) {
    case '\n':
        e = 'n';
        break;
    case '\r':
        e = 'r';
        break;
    case '\t':
        e = 't';
        break;
    default:
        out[0] = 'x';
        out[1] = HEXDIGIT[c >> 4];
        out[2] = HEXDIGIT[c & 15];
        return out + 3;
    }
    *out++ = e;
    return out;
}

// Write a C string to the given buffer, and return the advanced pointer.
static char *write_str(char *buf, const char *restrict str) {
    char *out = buf;
    for (const char *ptr = str; *ptr != '\0'; ptr++) {
        out = write_char(out, *ptr);
    }
    return out;
}

// Dump the parsed errors to standard output.
static void dump_errs(const struct errs *restrict errs) {
    size_t maxmsg = 0;
    for (size_t i = 0; i < errs->count; i++) {
        size_t len = strlen(errs->texts[i]);
        if (len > maxmsg) {
            maxmsg = len;
        }
    }
    char *buf = malloc(maxmsg * 4 + 1);
    if (buf == NULL) {
        die_nomem();
    }
    for (size_t i = 0; i < errs->count; i++) {
        cprintf(stdout, "%zu %s \"", i, errs->names[i]);
        char *out = write_str(buf, errs->texts[i]);
        *out = '\0';
        cwrite(stdout, buf, out - buf);
        cputs(stdout, "\"\n");
    }
    free(buf);
}

// Write a string array function to the given output file. If the output file is
// NULL, write to stdout.
static void write_array(const char *filename, char **arr, size_t count,
                        const char *funcname, const char *include) {
    if (count == 0) {
        die(errno, "zero array size");
    }
    // Calculate string offsets.
    size_t *offsets = malloc(sizeof(*offsets) * count);
    if (offsets == NULL) {
        die_nomem();
    }
    size_t offset = 0;
    size_t maxsize = 0;
    for (size_t i = 0; i < count; i++) {
        size_t size = strlen(arr[i]);
        if (size > maxsize) {
            maxsize = size;
        }
        offsets[i] = offset;
        offset += size + 1;
    }
    // Write output.
    FILE *fp;
    if (strcmp(filename, "-") == 0) {
        fp = stdout;
    } else {
        fp = fopen(filename, "w");
        if (fp == NULL) {
            die(errno, "could not open output");
        }
    }
    cputs(fp, "/* This file is automatically generated. */\n");
    if (include != NULL) {
        cprintf(fp, "#include \"%s\"\n", include);
    }
    cputs(fp, "#include <stdint.h>\n");
    cprintf(fp, "#define ERR_COUNT %zu\n", count);
    cputs(fp, "static char ERR_TEXT[] =\n");
    char *buf = malloc(9 + maxsize * 4);
    if (buf == NULL) {
        die_nomem();
    }
    for (size_t i = 0; i < count; i++) {
        char *out = buf;
        memcpy(out, "    \"", 5);
        out += 5;
        out = write_str(out, arr[i]);
        if (i + 1 == count) {
            memcpy(out, "\";\n", 3);
            out += 3;
        } else {
            memcpy(out, "\\0\"\n", 4);
            out += 4;
        }
        cwrite(fp, buf, out - buf);
    }
    free(buf);
    const char *atype;
    size_t maxoffset = offsets[count - 1];
    if (maxoffset > 0xffff) {
        atype = "uint32_t";
    } else if (maxoffset > 0xff) {
        atype = "uint16_t";
    } else {
        atype = "uint8_t";
    }
    cprintf(fp, "static const %s ERR_OFFSET[] = {\n", atype);
    for (size_t i = 0; i < count; i++) {
        cprintf(fp, "    %zu,\n", offsets[i]);
    }
    cputs(fp, "};\n");
    cprintf(fp, "const char *%s(int err) {\n", funcname);
    cputs(fp, "    if (err < 0 || ERR_COUNT <= err) {\n");
    cputs(fp, "        return 0;\n");
    cputs(fp, "    }\n");
    cputs(fp, "    return ERR_TEXT + ERR_OFFSET[err];\n");
    cputs(fp, "}\n");
    if (fp != stdout) {
        if (fclose(fp) < 0) {
            die_output();
        }
    }
}

struct args {
    bool dump;
    const char *input;
    const char *include;
    const char *msg_func;
    const char *msg_out;
    const char *name_func;
    const char *name_out;
};

enum {
    OPT_DUMP,
    OPT_HELP,
    OPT_INCLUDE,
    OPT_MSG_FUNC,
    OPT_MSG_OUT,
    OPT_NAME_FUNC,
    OPT_NAME_OUT,
};

static const struct ufxr_argdef ARG_DEFS[] = {
    {OPT_DUMP, "dump", ARG_BARE},
    {OPT_HELP, "help", ARG_BARE},
    {OPT_INCLUDE, "include", ARG_STRING},
    {OPT_MSG_FUNC, "msg-func", ARG_STRING},
    {OPT_MSG_OUT, "msg-out", ARG_STRING},
    {OPT_NAME_FUNC, "name-func", ARG_STRING},
    {OPT_NAME_OUT, "name-out", ARG_STRING},
    {},
};

static void parse_args(struct args *args, int argc, char **argv) {
    struct ufxr_argparser ap;
    ufxr_argparser_init(&ap, argc, argv);
    while (true) {
        int r = ufxr_argparser_next(&ap, ARG_DEFS);
        if (r < 0) {
            char *msg;
            switch (r) {
            case ARG_END:
                if (args->input == NULL) {
                    die_usage("input file not specified");
                }
                if (args->msg_out != NULL && args->msg_func == NULL) {
                    die_usage("-msg-out requires -msg-func to be specified");
                }
                if (args->name_out != NULL && args->name_func == NULL) {
                    die_usage("-name-out requires -name-func to be specified");
                }
                return;
            case ARG_POSITIONAL:
                if (args->input != NULL) {
                    die_usagef("unexpected argument '%s'", ap.val);
                }
                args->input = ap.val;
                break;
            default:
                msg = ufxr_argparser_err(&ap, r);
                if (msg == NULL) {
                    die_nomem();
                }
                die_usage(msg);
            }
        } else {
            switch (r) {
            case OPT_DUMP:
                args->dump = true;
                break;
            case OPT_HELP:
                cputs(stdout, USAGE);
                exit(0);
                break;
            case OPT_INCLUDE:
                args->include = ap.val;
                break;
            case OPT_MSG_FUNC:
                args->msg_func = ap.val;
                break;
            case OPT_MSG_OUT:
                args->msg_out = ap.val;
                break;
            case OPT_NAME_FUNC:
                args->name_func = ap.val;
                break;
            case OPT_NAME_OUT:
                args->name_out = ap.val;
                break;
            }
        }
    }
}

int main(int argc, char **argv) {
    struct args args = {};
    parse_args(&args, argc - 1, argv + 1);
    struct errs errs = read_input(args.input);
    if (args.dump) {
        dump_errs(&errs);
    }
    if (args.name_out != NULL) {
        write_array(args.name_out, errs.names, errs.count, args.name_func,
                    args.include);
    }
    if (args.msg_out != NULL) {
        write_array(args.msg_out, errs.texts, errs.count, args.msg_func,
                    args.include);
    }
    return 0;
}
