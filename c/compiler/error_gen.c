#include <errno.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char ERR_INPUT[] = "could not read input";
static const char ERR_NOMEM[] = "out of memory";

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

// Print an error message for the given input line number and exit the program.
static void dieline(int lineno, const char *msg) __attribute__((noreturn));
static void dieline(int lineno, const char *msg) {
    fprintf(stderr, "Error: line %d: %s\n", lineno, msg);
    exit(1);
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
        die(errno, ERR_NOMEM);
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
            dieline(lineno, "could not find start of error codes");
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
            dieline(lineno, "could not find end of error codes");
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
                    dieline(lineno, "multiple description for same error");
                }
                for (const char *ptr = start; ptr != end; ptr++) {
                    unsigned char c = *ptr;
                    if (c < 32 || c > 126) {
                        dieline(lineno, "description contains non-ASCII text");
                    }
                }
                text = copystr(start, end);
            }
        } else if (has_prefix(start, end, "ERR_")) {
            // Error name.
            if (text == NULL) {
                dieline(lineno, "error code has no description");
            }
            start += 4;
            end = start;
            while (is_ident(*end)) {
                end++;
            }
            if (start == end) {
                dieline(lineno, "invalid error code name");
            }
            char *name = copystr(start, end);
            if (errs.count >= errs.alloc) {
                errs.alloc = errs.alloc ? errs.alloc * 2 : 8;
                errs.names =
                    realloc(errs.names, errs.alloc * sizeof(errs.names));
                if (errs.names == NULL) {
                    die(errno, ERR_NOMEM);
                }
                errs.texts =
                    realloc(errs.texts, errs.alloc * sizeof(errs.texts));
                if (errs.texts == NULL) {
                    die(errno, ERR_NOMEM);
                }
            }
            errs.names[errs.count] = name;
            errs.texts[errs.count] = text;
            errs.count++;
            text = NULL;
        } else if (has_prefix(start, end, "};")) {
            // End of errors.
            if (text != NULL) {
                dieline(lineno, "expected error code name");
            }
            if (errs.count == 0) {
                dieline(lineno, "no error codes found");
            }
            break;
        } else {
            dieline(lineno, "could not parse error");
        }
    }
    fclose(fp);
    return errs;
}

static void diewrite(void) __attribute__((noreturn));
static void diewrite(void) {
    die(errno, "could not write output");
}

static void cputs(FILE *fp, const char *s) {
    if (fputs(s, fp) < 0) {
        diewrite();
    }
}

static void cprintf(FILE *fp, const char *fmt, ...)
    __attribute__((format(printf, 2, 3)));
static void cprintf(FILE *fp, const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    if (vfprintf(fp, fmt, ap) < 0) {
        diewrite();
    }
    va_end(ap);
}

static void cwrite(FILE *fp, const char *p, size_t size) {
    size_t r = fwrite(p, 1, size, fp);
    if (r != size) {
        diewrite();
    }
}

// Write a string array function to the given output file. If the output file is
// NULL, write to stdout.
static void write_array(const char *filename, char **arr, size_t count,
                        const char *funcname) {
    if (count == 0) {
        die(errno, "zero array size");
    }
    // Calculate string offsets.
    size_t *offsets = malloc(sizeof(*offsets) * count);
    if (offsets == NULL) {
        die(errno, ERR_NOMEM);
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
    if (filename == NULL) {
        fp = stdout;
    } else {
        fp = fopen(filename, "w");
        if (fp == NULL) {
            die(errno, "could not open output");
        }
    }
    cputs(fp, "/* This file is automatically generated. */\n");
    cputs(fp, "#include \"c/compiler/error.h\"\n");
    cputs(fp, "#include <stdint.h>\n");
    cprintf(fp, "#define ERR_COUNT %zu\n", count);
    cputs(fp, "static char ERR_TEXT[] =\n");
    char *buf = malloc(9 + maxsize * 2);
    if (buf == NULL) {
        die(errno, ERR_NOMEM);
    }
    for (size_t i = 0; i < count; i++) {
        char *out = buf;
        memcpy(out, "    \"", 5);
        out += 5;
        for (const char *ptr = arr[i]; *ptr; ptr++) {
            unsigned char c = *ptr;
            if (c == '\\' || c == '"') {
                *out++ = '\\';
            }
            *out++ = c;
        }
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
    if (filename != NULL) {
        if (fclose(fp) < 0) {
            diewrite();
        }
    }
}

int main(int argc, char **argv) {
    const char *input, *outname, *outtext;
    switch (argc) {
    case 2:
        input = argv[1];
        outname = NULL;
        outtext = NULL;
        break;
    case 4:
        input = argv[1];
        outname = argv[2];
        outtext = argv[3];
        break;
    default:
        die(0, "invalid usage");
    }
    struct errs errs = read_input(input);
    write_array(outname, errs.names, errs.count, "ufxr_errname");
    write_array(outtext, errs.texts, errs.count, "ufxr_errtext");
    return 0;
}
