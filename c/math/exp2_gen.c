// exp2_gen.c - Generate exp2 functions.
#include "c/util/util.h"

#include <errno.h>
#include <stdlib.h>
#include <unistd.h>

static void emit(int order, char **coeffs) {
    char fname[30];
    xsprintf(fname, sizeof(fname), "exp2_%d.c", order);
    FILE *fp = fopen(fname, "wb");
    if (fp == NULL) {
        goto error;
    }

    (void)coeffs;

    xputs(fp, kNotice);
    xputs(fp, "#include \"c/math/math.h\"\n");
    xputs(fp, "#include <math.h>\n");
    xprintf(fp, "void ufxr_exp2_%d", order);
    xputs(fp, "(int n, float *restrict outs, const float *restrict xs) {\n");
    for (int i = 0; i <= order; i++) {
        xprintf(fp, "    const float c%d = %sf;\n", i, coeffs[i]);
    }
    xputs(fp, "    for (int i = 0; i < n; i++) {\n");
    xputs(fp, "        float x = xs[i];\n");
    xputs(fp, "        float ival = rintf(x);\n");
    xputs(fp, "        float frac = x - ival;\n");
    xprintf(fp, "        float y = c%d;\n", order);
    for (int i = order - 1; i >= 0; i--) {
        xprintf(fp, "        y = y * frac + c%d;\n", i);
    }
    xputs(fp, "        outs[i] = scalbnf(y, (int)ival);\n");
    xputs(fp, "    }\n");
    xputs(fp, "}\n");

    int r = fclose(fp);
    if (r != 0) {
        goto error;
    }
    return;
error:;
    int ecode = errno;
    dief(ecode, "could not write %s", quote_str(fname));
}

int main(int argc, char **argv) {
    if (argc != 4) {
        fputs("Usage: exp2_gen <max-order> <exp2.csv> <out-dir>\n", stderr);
        exit(64);
    }
    int max_order = xatoi(argv[1]);
    const char *inpath = argv[2];
    const char *outdir = argv[3];

    struct data data = {0};
    read_file(&data, inpath);
    int r = chdir(outdir);
    if (r != 0) {
        die(errno, "chdir");
    }
    struct strings lines = {0};
    split_lines(&lines, &data);
    struct strings fields = {0};
    for (size_t lineidx = 0; lineidx < lines.count; lineidx++) {
        int lineno = lineidx + 1;
        char *line = lines.strings[lineidx];
        if (*line == '\0') {
            continue;
        }
        split_csv(&fields, line);
        char *ostr = fields.strings[0], *end;
        long order = strtol(ostr, &end, 10);
        if (*ostr == '\0' || *end != '\0' || order < 0) {
            dief(0, "line %d: invalid order: %s", lineno, quote_str(ostr));
        }
        if (fields.count != (size_t)order + 2) {
            dief(0, "line %d: found %zu fields, expected %zu", lineno,
                 fields.count, (size_t)order + 2);
        }
        if (order <= max_order) {
            emit(order, fields.strings + 1);
        }
    }

    return 0;
}
