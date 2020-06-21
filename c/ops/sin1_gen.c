// sin4_gen.c - Generate sin4 functions.
#include "c/util/util.h"

#include <errno.h>
#include <stdlib.h>
#include <unistd.h>

static void emit(int order, char **coeffs) {
    char fname[30];
    xsprintf(fname, sizeof(fname), "sin1_%d.c", order);
    FILE *fp = fopen(fname, "wb");
    if (fp == NULL) {
        goto error;
    }

    // Note that coeffs are scaled for period = 4.

    const char *args =
        "(int n, float *restrict outs, const float *restrict xs)";

    xputs(fp, kNotice);
    xputs(fp,
          "#include \"c/ops/ops.h\"\n"
          "#include <assert.h>\n");

    xputs(fp,
          "\n"
          "// Scalar version.\n"
          "#if !HAVE_FUNC\n");
    xprintf(fp, "void ufxr_sin1_%d%s {\n", order, args);
    xputs(fp, "    assert((n % UFXR_QUANTUM) == 0);\n");
    for (int i = 0; i < order - 1; i++) {
        xprintf(fp, "    const float c%d = %d.0f * %sf;\n", i, 1 << (2 + 4 * i),
                coeffs[i]);
    }
    xputs(fp,
          "    for (int i = 0; i < n; i++) {\n"
          "        float x = xs[i];\n"
          "        x -= (float)(int)x;\n"
          "        float t1 = 0.5f - x;\n"
          "        float t2 = -0.5f - x;\n"
          "        if (t1 < x)\n"
          "            x = t1;\n"
          "        if (t2 > x)\n"
          "            x = t2;\n"
          "        float x2 = x * x;\n");
    xprintf(fp, "        float y = c%d;\n", order - 2);
    for (int i = order - 3; i >= 0; i--) {
        xprintf(fp, "        y = y * x2 + c%d;\n", i);
    }
    xputs(fp,
          "        outs[i] = x * y;\n"
          "    }\n"
          "}\n"
          "#endif\n");

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
        fputs("Usage: sin1_gen <max-order> <sin4.csv> <out-dir>\n", stderr);
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
        // CSV file uses different idea of what "order" means.
        order++;
        if (order < 3 || order > max_order) {
            continue;
        }
        if (fields.count != (size_t)order) {
            dief(0, "line %d: found %zu fields, expected %zu", lineno,
                 fields.count, (size_t)order);
        }
        emit(order, fields.strings + 1);
    }

    return 0;
}
